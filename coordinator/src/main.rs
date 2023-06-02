use actix_web::{App, HttpServer};

mod api_handlers;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(api_handlers::get_root))
        .bind(("127.0.0.1", 5000))?
        .run()
        .await
}
