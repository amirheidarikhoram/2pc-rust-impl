use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Responder,
};

use crate::app_state::AppState;

use core_2pc::{Post, Transaction};

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

    let _coordinator = state.as_ref().coordinator.clone();

    let client = state.as_ref().pool.clone().lock().unwrap().clone();
    let mut client = client.get().await.unwrap();

    let transaction = client.transaction().await.unwrap();

    /*  TODO: there are 2 ways to send parameters to other nodes:
            1. send parameters in the command
            2. serialize the parameters and send them as a separate message

        I use the first method here to avoid any complexity.
    */

    let tx = Transaction::new(format!("INSERT INTO post (body) VALUES ($1)"));

    if let Ok(_) = transaction
        .execute(tx.command.as_str(), &[&post.body.as_str()])
        .await
    {
        // TODO: execute transaction
        // coordinator.lock().unwrap().execute_transaction(tx).unwrap();
    } else {
        transaction.rollback().await.unwrap();

        return HttpResponse::InternalServerError().body(format!(
            "An error occured, we could not execute commands on db"
        ));
    }

    HttpResponse::Ok().body("ok")
}
