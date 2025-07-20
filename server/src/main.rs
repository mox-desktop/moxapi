mod idle;
mod notify;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    App, HttpResponse, HttpServer, get,
    middleware::{self, DefaultHeaders},
    post, web,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

struct State {
    idle: Arc<RwLock<idle::Idle>>,
    notify: notify::NotificationManager,
}

#[post("/idle/inhibit")]
async fn post_idle_inhibit(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    if data.idle.write().await.inhibit().await.is_ok() {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to idle inhibit"
        })))
    }
}

#[post("/idle/uninhibit")]
async fn post_idle_uninhibit(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    if data.idle.write().await.uninhibit().await.is_ok() {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to idle uninhibit"
        })))
    }
}

#[post("/idle/lock")]
async fn post_idle_lock(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    if data.idle.read().await.lock().await.is_ok() {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to lock"
        })))
    }
}

#[post("/idle/unlock")]
async fn post_idle_unlock(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    if data.idle.read().await.unlock().await.is_ok() {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to unlock"
        })))
    }
}

#[post("/idle/simulate_user_activity")]
async fn post_simulate_user_activity(
    data: web::Data<State>,
) -> Result<HttpResponse, actix_web::Error> {
    if data
        .idle
        .read()
        .await
        .simulate_user_activity()
        .await
        .is_ok()
    {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to unlock"
        })))
    }
}

#[get("/notify/capabilities")]
async fn get_notify_capabilities(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    match data.notify.get_capabilities().await {
        Ok(capabilities) => Ok(HttpResponse::Ok().json(capabilities)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Error: {e}"))),
    }
}

#[derive(Deserialize)]
struct NotificationRequest {
    summary: Box<str>,
    body: Box<str>,
    timeout: i32,
    id: u32,
}

#[post("/notify")]
async fn post_notify(
    data: web::Data<State>,
    req_body: web::Json<NotificationRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let builder = data.notify.builder().await;
    builder
        .with_summary(&req_body.summary)
        .with_body(&req_body.body)
        .with_timeout(req_body.timeout)
        .with_id(req_body.id)
        .send()
        .await;

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = web::Data::new(State {
        idle: Arc::new(RwLock::new(idle::Idle::new().await.unwrap())),
        notify: notify::NotificationManager::new().await.unwrap(),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods(vec!["POST", "GET"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        let governor_conf = GovernorConfigBuilder::default()
            .seconds_per_request(2)
            .burst_size(5)
            .finish()
            .unwrap();

        App::new()
            .app_data(state.clone())
            .app_data(web::PayloadConfig::new(1024 * 1024))
            .app_data(web::JsonConfig::default().limit(1024 * 1024))
            .wrap(Governor::new(&governor_conf))
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add((
                        "Strict-Transport-Security",
                        "max-age=31536000; includeSubDomains; preload",
                    ))
                    .add(("Content-Security-Policy", "default-src 'self'"))
                    .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
                    .add((
                        "Permissions-Policy",
                        "geolocation=(), microphone=(), camera=()",
                    )),
            )
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(post_idle_lock)
            .service(post_idle_unlock)
            .service(post_simulate_user_activity)
            .service(post_idle_inhibit)
            .service(post_idle_uninhibit)
            .service(get_notify_capabilities)
            .service(post_notify)
    })
    .bind(("0.0.0.0", 8000))?
    .workers(2)
    .run()
    .await
}
