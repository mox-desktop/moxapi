use actix_files::NamedFile;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, post, web};
use askama::Template;
use awc::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

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
#[template(path = "host_panel.html")]
struct HostPanelTemplate {
    hostname: String,
    ip: String,
    status: String,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    hostname: String,
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
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
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

#[get("/host-panel/{hostname}")]
async fn host_panel(path: web::Path<String>) -> impl Responder {
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

    if let Some((k, v)) = hosts_with_keys.get_key_value(&hostname) {
        let template = HostPanelTemplate {
            hostname: k.clone(),
            ip: v.ip.clone(),
            status: "online".to_string(),
        };
        HttpResponse::Ok().body(template.render().unwrap())
    } else {
        HttpResponse::NotFound().body("Host not found")
    }
}

#[get("/dashboard/{hostname}")]
async fn dashboard(path: web::Path<String>) -> impl Responder {
    let hostname = path.into_inner();
    let template = DashboardTemplate { hostname };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[post("/add-host")]
async fn add_host(_form: web::Form<std::collections::HashMap<String, String>>) -> impl Responder {
    // This endpoint is now a no-op for config-based hosts
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
        let ip = &host.ip;
        let api_key = host.api_key.as_deref().unwrap_or("");
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
                let url = format!("{ip}{endpoint}");
                let client = Client::default();
                match client
                    .post(url)
                    .insert_header(("Authorization", api_key))
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(add_host_form)
            .service(add_host)
            .service(get_hosts)
            .service(host_panel)
            .service(dashboard)
            .service(host_action)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
