use std::sync::{Arc};

use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

mod element;
mod peer_transaction;

#[tokio::main]
async fn main() {
    let mut cfg = Config::new();
    cfg.user = Some("amir".to_string());
    // TODO: db_name should be unique for every element
    cfg.dbname = Some("2pc_e1".to_string());
    cfg.host = Some("localhost".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = Arc::new(Mutex::new(
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap(),
    ));

    element::Element::new().await.run(pool).await;
}
