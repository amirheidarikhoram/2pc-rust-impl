use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn get_root() -> impl Responder {
    HttpResponse::Ok().body("hello")
}
