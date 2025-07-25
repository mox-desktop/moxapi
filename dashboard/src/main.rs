mod config;

use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use actix_web::{http::header, middleware::Logger};
use askama::Template;
use awc::Client;
use chrono_humanize::{Accuracy, Tense};
use futures::future;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    hostname: String,
    ip: String,
    status: &'static str,
    last_seen: String,
}

#[derive(Debug, Clone)]
pub struct HostWithStatus {
    pub hostname: String,
    pub ip: String,
    pub status: String,
}

#[derive(Template)]
#[template(path = "main_dashboard.html")]
pub struct MainDashboardTemplate {
    pub hosts: Vec<HostWithStatus>,
}

#[get("/")]
async fn index(
    session: Session,
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    if session
        .get::<bool>("logged_in")
        .unwrap_or(Some(false))
        .unwrap_or(false)
    {
        let config = data.read().await;

        let futures: Vec<_> = config
            .hosts
            .iter()
            .map(|(hostname, host)| async {
                let response = Client::default()
                    .get(format!("{}/status", host.ip))
                    .insert_header(("Authorization", host.api_key.clone()))
                    .timeout(Duration::from_secs(1))
                    .send()
                    .await;
                let status = match response {
                    Ok(mut res) => res.json::<Status>().await.unwrap_or_default(),
                    Err(_) => {
                        return HostWithStatus {
                            hostname: hostname.clone(),
                            ip: host.ip.clone(),
                            status: "offline".to_string(),
                        };
                    }
                };
                let status = match status.active {
                    true => "idle",
                    false => "online",
                }
                .to_string();

                HostWithStatus {
                    hostname: hostname.clone(),
                    ip: host.ip.clone(),
                    status,
                }
            })
            .collect();

        let hosts: Vec<HostWithStatus> = future::join_all(futures).await;
        let template = MainDashboardTemplate { hosts };
        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(template.render().unwrap()))
    } else {
        Ok(HttpResponse::Found()
            .append_header((header::LOCATION, "/login"))
            .finish())
    }
}

#[derive(Deserialize, Serialize, Default)]
struct Status {
    active: bool,
    active_time: u32,
    inhibited: bool,
}

#[get("/status/{hostname}")]
async fn get_status(
    path: web::Path<String>,
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let hostname = path.into_inner();
    let config = data.read().await;
    let host = config.hosts.get(&hostname).unwrap();
    let response = Client::default()
        .get(format!("{}/status", host.ip))
        .insert_header(("Authorization", host.api_key.clone()))
        .send()
        .await;

    Ok(HttpResponse::Ok().json(response.unwrap().json::<Status>().await.unwrap_or_default()))
}

#[derive(Template)]
#[template(path = "add_host_form.html")]
struct AddHostFormTemplate;

#[get("/add-host-form")]
async fn add_host_form() -> Result<HttpResponse, actix_web::Error> {
    let template = AddHostFormTemplate;
    Ok(HttpResponse::Ok().body(template.render().unwrap()))
}

#[get("/hosts")]
async fn get_hosts(
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let config = data.read().await;
    Ok(HttpResponse::Ok().json(&config.hosts))
}

#[get("/dashboard/{hostname}")]
async fn dashboard(
    path: web::Path<String>,
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let hostname = path.into_inner();
    let config = data.read().await;
    let host = match config.hosts.get(&hostname) {
        Some(h) => h,
        None => return Ok(HttpResponse::NotFound().body("Host not found")),
    };

    let response = Client::default()
        .get(format!("{}/status", host.ip))
        .insert_header(("Authorization", host.api_key.clone()))
        .send()
        .await;

    let mut response = match response {
        Ok(resp) => resp,
        Err(_) => {
            let template = DashboardTemplate {
                ip: host.ip.clone(),
                status: "offline",
                hostname,
                last_seen: "now".to_string(),
            };
            return Ok(HttpResponse::Ok().body(template.render().unwrap()));
        }
    };

    let status: Status = match response.json().await {
        Ok(s) => s,
        Err(_) => {
            let template = DashboardTemplate {
                ip: host.ip.clone(),
                status: "offline",
                hostname,
                last_seen: "now".to_string(),
            };
            return Ok(HttpResponse::Ok().body(template.render().unwrap()));
        }
    };

    let dt = chrono::Local::now() - chrono::Duration::seconds(status.active_time as i64);
    let ht = chrono_humanize::HumanTime::from(dt);

    let status = match status.active {
        true => "idle",
        false => "online",
    };

    let template = DashboardTemplate {
        ip: host.ip.clone(),
        status,
        hostname,
        last_seen: ht.to_text_en(Accuracy::Rough, Tense::Past),
    };
    Ok(HttpResponse::Ok().body(template.render().unwrap()))
}

#[post("/add-host")]
async fn add_host(_form: web::Form<std::collections::HashMap<String, String>>) -> impl Responder {
    HttpResponse::Ok().body("<div style='text-align:center;padding:2rem 0;'>Adding hosts is disabled. Edit config/hosts.yaml.</div>")
}

#[post("/action/{hostname}/{action}")]
async fn host_action(
    path: web::Path<(String, String)>,
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (hostname, action) = path.into_inner();
    let config = data.read().await;

    let host = config
        .hosts
        .get(&hostname)
        .ok_or_else(|| actix_web::error::ErrorNotFound("Host not found"))?;

    let endpoint = match action.as_str() {
        "lock" => "/idle/lock",
        "unlock" => "/idle/unlock",
        "simulate-activity" => "/idle/simulate_user_activity",
        "inhibit" => "/idle/inhibit",
        "uninhibit" => "/idle/uninhibit",
        _ => return Ok(HttpResponse::BadRequest().body("Unknown action")),
    };

    let resp = Client::default()
        .post(format!("{}{endpoint}", host.ip))
        .insert_header(("Authorization", host.api_key.clone()))
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to send {action} command: {e}"
            ))
        })?;

    if resp.status().is_success() {
        Ok(HttpResponse::Ok().body(format!("{action} command sent successfully!")))
    } else {
        Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to send {action} command. Status: {}",
            resp.status()
        )))
    }
}

#[post("/reload-config")]
async fn reload_config(
    data: web::Data<Arc<RwLock<config::Config>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut config = data.write().await;
    *config = config::Config::load().unwrap_or_default();
    Ok(HttpResponse::Ok().body("Config reloaded."))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    incorrect: bool,
}

#[get("/login")]
async fn login_form() -> Result<HttpResponse, actix_web::Error> {
    let template = LoginTemplate { incorrect: false };
    Ok(HttpResponse::Ok().body(template.render().unwrap()))
}

#[post("/login")]
async fn login_post(
    data: web::Data<Arc<RwLock<config::Config>>>,
    form: web::Form<std::collections::HashMap<String, String>>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    if let Some(pass) = form.get("password") {
        let data = data.into_inner();
        let config = data.read().await;

        if pass == &config.password {
            session.insert("logged_in", true).unwrap();
            return Ok(HttpResponse::Found()
                .append_header((header::LOCATION, "/"))
                .finish());
        }
    }

    let template = LoginTemplate { incorrect: true };
    Ok(HttpResponse::Ok().body(template.render().unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let secret = Key::generate();
    let config = Arc::new(RwLock::new(config::Config::load().unwrap_or_default()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .service(login_form)
            .service(login_post)
            .service(index)
            .service(add_host_form)
            .service(add_host)
            .service(get_hosts)
            .service(dashboard)
            .service(host_action)
            .service(get_status)
            .service(reload_config)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
