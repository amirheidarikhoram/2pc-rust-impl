use actix_web::{get, web::Data, HttpResponse, Responder};

use crate::app_state::AppState;

#[get("/")]
async fn get_root() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

#[get("/post")]
async fn get_posts(state: Data<AppState>) -> impl Responder {
    let client = state.as_ref().pool.clone().lock().unwrap().clone();
    let client = client.get().await.unwrap();

    let rows = client
        .query(
            "
        SELECT body FROM post
    ",
            &[],
        )
        .await
        .unwrap();

    let mut posts = Vec::new();

    for row in rows {
        let body: String = row.get(0);
        posts.push(body);
    }

    HttpResponse::Ok().json(posts)
}
