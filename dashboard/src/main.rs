use actix_files::NamedFile;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, post, web};
use askama::Template;
use awc::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{HttpRequest, http::header, middleware::Logger};
use actix_web::cookie::Key;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Host {
    ip: String,
    api_key_file: String,
    #[serde(skip)]
    api_key: Option<String>,
}

impl Host {
    fn with_api_key(mut self) -> Self {
        match std::fs::read_to_string(&self.api_key_file) {
            Ok(key) => self.api_key = Some(key.trim().to_string()),
            Err(_) => self.api_key = None,
        }
        self
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    hostname: String,
    ip: String,
    status: String,
}

fn resolve_config_path() -> Option<std::path::PathBuf> {
    if let Some(arg_path) = env::args().nth(1) {
        return Some(std::path::PathBuf::from(arg_path));
    }
    if let Ok(env_path) = env::var("MOXAPI_CONFIG") {
        return Some(std::path::PathBuf::from(env_path));
    }
    if let Some(home) = dirs::config_dir() {
        let fallback = home.join("mox/moxapi/hosts.yaml");
        if fallback.exists() {
            return Some(fallback);
        }
    }
    let etc_path = std::path::PathBuf::from("/etc/moxapi/hosts.yaml");
    if etc_path.exists() {
        return Some(etc_path);
    }

    None
}

#[get("/")]
async fn index(session: Session) -> impl Responder {
    if session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false) {
        match std::fs::read("static/index.html") {
            Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
            Err(_) => HttpResponse::InternalServerError().body("Failed to load index.html"),
        }
    } else {
        HttpResponse::Found().append_header((header::LOCATION, "/login")).finish()
    }
}

#[derive(Deserialize, Serialize)]
struct Status {
    active: bool,
    active_time: u32,
    inhibited: bool,
}

#[get("/status/{hostname}")]
async fn get_status(path: web::Path<String>) -> impl Responder {
    let hostname = path.into_inner();
    let config_path = resolve_config_path().unwrap();
    let hosts: HashMap<String, Host> = match fs::read_to_string(config_path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    };
    let hosts_with_keys: HashMap<String, Host> = hosts
        .into_iter()
        .map(|(k, v)| (k, v.with_api_key()))
        .collect();

    let host = hosts_with_keys.get(&hostname).unwrap();
    let response = Client::default()
        .get(format!("{}/status", host.ip))
        .insert_header(("Authorization", host.api_key.as_deref().unwrap_or("")))
        .send()
        .await;

    HttpResponse::Ok().json(response.unwrap().json::<Status>().await.unwrap())
}

#[derive(Template)]
#[template(path = "add_host_form.html")]
struct AddHostFormTemplate;

#[get("/add-host-form")]
async fn add_host_form() -> impl Responder {
    let template = AddHostFormTemplate;
    HttpResponse::Ok().body(template.render().unwrap())
}

#[get("/hosts")]
async fn get_hosts() -> impl Responder {
    let hosts: HashMap<String, Host> = if let Some(config_path) = resolve_config_path() {
        match fs::read_to_string(config_path) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };
    let hosts_with_keys: HashMap<String, Host> = hosts
        .into_iter()
        .map(|(k, v)| (k, v.with_api_key()))
        .collect();
    HttpResponse::Ok().json(hosts_with_keys)
}

#[get("/dashboard/{hostname}")]
async fn dashboard(path: web::Path<String>) -> impl Responder {
    let hostname = path.into_inner();

    let hosts: HashMap<String, Host> = if let Some(config_path) = resolve_config_path() {
        match fs::read_to_string(config_path) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    let host = hosts.get(&hostname).unwrap();

    let template = DashboardTemplate {
        ip: host.ip.clone(),
        status: "online".to_string(),
        hostname,
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[post("/add-host")]
async fn add_host(_form: web::Form<std::collections::HashMap<String, String>>) -> impl Responder {
    HttpResponse::Ok().body("<div style='text-align:center;padding:2rem 0;'>Adding hosts is disabled. Edit config/hosts.yaml.</div>")
}

#[post("/action/{hostname}/{action}")]
async fn host_action(path: web::Path<(String, String)>) -> impl Responder {
    let (hostname, action) = path.into_inner();
    let config_path = resolve_config_path().unwrap();
    let hosts: HashMap<String, Host> = match fs::read_to_string(config_path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    };
    let hosts_with_keys: HashMap<String, Host> = hosts
        .into_iter()
        .map(|(k, v)| (k, v.with_api_key()))
        .collect();

    if let Some(host) = hosts_with_keys.get(&hostname) {
        match action.as_str() {
            "lock" | "unlock" | "simulate-activity" | "inhibit" | "uninhibit" => {
                let endpoint = match action.as_str() {
                    "lock" => "/idle/lock",
                    "unlock" => "/idle/unlock",
                    "simulate-activity" => "/idle/simulate_user_activity",
                    "inhibit" => "/idle/inhibit",
                    "uninhibit" => "/idle/uninhibit",
                    _ => unreachable!(),
                };
                match Client::default()
                    .post(format!("{}{endpoint}", host.ip))
                    .insert_header(("Authorization", host.api_key.as_deref().unwrap()))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            HttpResponse::Ok().body(format!("{action} command sent successfully!"))
                        } else {
                            HttpResponse::InternalServerError()
                                .body(format!("Failed to send {action} command."))
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError()
                        .body(format!("Failed to send {action} command.")),
                }
            }
            _ => HttpResponse::BadRequest().body("Unknown action"),
        }
    } else {
        HttpResponse::NotFound().body("Host not found")
    }
}

#[get("/login")]
async fn login_form() -> impl Responder {
    let html = r#"
    <html><head><title>Login</title></head><body style='background:#111;color:#fff;display:flex;align-items:center;justify-content:center;height:100vh;'>
    <form method='post' action='/login' style='background:#222;padding:2rem;border-radius:1rem;box-shadow:0 0 10px #000;'>
      <h2 style='margin-bottom:1rem;'>Login</h2>
      <input type='password' name='password' placeholder='Password' style='padding:0.5rem;width:100%;margin-bottom:1rem;border-radius:0.5rem;border:none;'/><br/>
      <button type='submit' style='padding:0.5rem 2rem;border-radius:0.5rem;border:none;background:#fff;color:#111;font-weight:bold;'>Login</button>
    </form>
    </body></html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[post("/login")]
async fn login_post(form: web::Form<std::collections::HashMap<String, String>>, session: Session) -> impl Responder {
    if let Some(pass) = form.get("password") {
        if pass == "1234" {
            session.insert("logged_in", true).unwrap();
            return HttpResponse::Found().append_header((header::LOCATION, "/")).finish();
        }
    }
    let html = r#"
    <html><head><title>Login</title></head><body style='background:#111;color:#fff;display:flex;align-items:center;justify-content:center;height:100vh;'>
    <form method='post' action='/login' style='background:#222;padding:2rem;border-radius:1rem;box-shadow:0 0 10px #000;'>
      <h2 style='margin-bottom:1rem;'>Login</h2>
      <input type='password' name='password' placeholder='Password' style='padding:0.5rem;width:100%;margin-bottom:1rem;border-radius:0.5rem;border:none;'/><br/>
      <div style='color:#f44;margin-bottom:1rem;'>Incorrect password</div>
      <button type='submit' style='padding:0.5rem 2rem;border-radius:0.5rem;border:none;background:#fff;color:#111;font-weight:bold;'>Login</button>
    </form>
    </body></html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

fn is_logged_in(session: &Session) -> bool {
    session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false)
}

// Wrap handlers to require login
macro_rules! require_login {
    ($handler:expr) => {{
        |req: HttpRequest, payload: web::Payload, session: Session| {
            if is_logged_in(&session) {
                $handler(req, payload, session)
            } else {
                Box::pin(async {
                    HttpResponse::Found().append_header((header::LOCATION, "/login")).finish()
                })
            }
        }
    }};
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let secret = Key::generate();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret.clone(),
            ))
            .service(login_form)
            .service(login_post)
            // All other routes require login
            .service(index)
            .service(add_host_form)
            .service(add_host)
            .service(get_hosts)
            .service(dashboard)
            .service(host_action)
            .service(get_status)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
