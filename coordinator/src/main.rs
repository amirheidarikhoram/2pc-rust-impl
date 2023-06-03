use std::sync::{Arc, Mutex};

use actix_web::{web::Data, App, HttpServer};
use app_state::AppState;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

mod api_handlers;
mod app_state;
mod coordinator;
mod peer;
mod transaction;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut cfg = Config::new();
    cfg.user = Some("amir".to_string());
    cfg.dbname = Some("2pc".to_string());
    cfg.host = Some("localhost".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = Arc::new(Mutex::new(
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap(),
    ));

    let coordinator = Arc::new(Mutex::new(coordinator::Coordinator::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                coordinator: coordinator.clone(),
                pool: Arc::clone(&pool),
            }))
            .service(api_handlers::get_root)
            .service(api_handlers::get_posts)
            .service(api_handlers::create_post)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
