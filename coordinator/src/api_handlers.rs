use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::app_state::AppState;

use core_2pc::{convert_args, Command, Post, Transaction};

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

#[post("/post")]
async fn create_post(state: Data<AppState>, post: Json<Post>) -> impl Responder {
    let state = state.clone();

    let coordinator = state.as_ref().coordinator.clone();

    let client = state.as_ref().pool.clone().lock().unwrap().clone();
    let mut client = client.get().await.unwrap();

    let transaction = client.transaction().await.unwrap();

    let tx = Transaction::new(Command {
        query: format!("INSERT INTO post (body) VALUES ($1)"),
        args: vec![post.body.clone()],
    });

    let params = convert_args(tx.command.args.iter());

    if let Ok(_) = transaction
        .execute(tx.command.query.as_str(), &params)
        .await
    {
        coordinator.lock().unwrap().execute_transaction(tx).unwrap();
        transaction.commit().await.unwrap();
    } else {
        transaction.rollback().await.unwrap();

        return HttpResponse::InternalServerError().body(format!(
            "An error occured, we could not execute commands on db"
        ));
    }

    HttpResponse::Ok().body("ok")
}
