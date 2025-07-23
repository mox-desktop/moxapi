mod idle;
mod notify;

use actix_cors::Cors;
use actix_web::{
    App, Error, HttpResponse, HttpServer,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    get,
    middleware::{self, DefaultHeaders},
    post, web,
};
use clap::Parser;
use env_logger::Builder;
use futures_util::future::{LocalBoxFuture, Ready, ok};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{env, rc::Rc, sync::Arc};
use tokio::sync::RwLock;

struct State {
    idle: Arc<RwLock<idle::Idle>>,
    notify: notify::NotificationManager,
}

#[derive(Serialize)]
struct Status {
    active: bool,
    active_time: u32,
    inhibited: bool,
}

#[get("/status")]
async fn get_status(data: web::Data<State>) -> Result<HttpResponse, actix_web::Error> {
    let idle = data.idle.read().await;
    let status = Status {
        active: idle.get_active().await.unwrap(),
        active_time: idle.get_active_time().await.unwrap(),
        inhibited: idle.get_inhibited(),
    };

    Ok(HttpResponse::Ok().json(status))
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
        .await
        .unwrap();

    Ok(HttpResponse::Ok().finish())
}

pub struct AuthMiddleware {
    key: String,
}

impl AuthMiddleware {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            key: self.key.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    key: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = self.key.clone();
        let srv = self.service.clone();
        Box::pin(async move {
            let authorized = req
                .headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .map(|v| v == key)
                .unwrap_or(false);
            if !authorized {
                return Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .body("Unauthorized")
                        .map_into_boxed_body(),
                ));
            }
            let res = srv.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, action = clap::ArgAction::Count)]
    quiet: u8,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut log_level = LevelFilter::Info;

    (0..cli.verbose).for_each(|_| {
        log_level = match log_level {
            LevelFilter::Error => LevelFilter::Warn,
            LevelFilter::Warn => LevelFilter::Info,
            LevelFilter::Info => LevelFilter::Debug,
            LevelFilter::Debug => LevelFilter::Trace,
            _ => log_level,
        };
    });

    (0..cli.quiet).for_each(|_| {
        log_level = match log_level {
            LevelFilter::Warn => LevelFilter::Error,
            LevelFilter::Info => LevelFilter::Warn,
            LevelFilter::Debug => LevelFilter::Info,
            LevelFilter::Trace => LevelFilter::Debug,
            _ => log_level,
        };
    });

    Builder::new().filter(Some("daemon"), log_level).init();

    let state = web::Data::new(State {
        idle: Arc::new(RwLock::new(idle::Idle::new().await.unwrap())),
        notify: notify::NotificationManager::new().await.unwrap(),
    });

    let auth_key = match env::var("AUTH_KEY_FILE") {
        Ok(auth_key_file) => std::fs::read_to_string(&auth_key_file)
            .unwrap_or_else(|_| panic!("Failed to read {auth_key_file}")),
        Err(_) => env::var("AUTH_KEY").expect("AUTH_KEY_FILE or AUTH_KEY env var must be set"),
    };

    HttpServer::new(move || {
        App::new()
            .wrap(AuthMiddleware::new(auth_key.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .expose_headers(vec![actix_web::http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .app_data(state.clone())
            .app_data(web::PayloadConfig::new(1024 * 1024))
            .app_data(web::JsonConfig::default().limit(1024 * 1024))
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-XSS-Protection", "1; mode=block")),
            )
            .wrap(middleware::Logger::default())
            .service(post_idle_lock)
            .service(post_idle_unlock)
            .service(post_simulate_user_activity)
            .service(post_idle_inhibit)
            .service(post_idle_uninhibit)
            .service(get_notify_capabilities)
            .service(post_notify)
            .service(get_status)
    })
    .bind(("0.0.0.0", 8000))?
    .workers(2)
    .run()
    .await
}
