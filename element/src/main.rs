use std::{env, sync::Arc};

use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

mod element;

#[tokio::main]
async fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        eprintln!("error: invalid number of arguments");
        return;
    }

    let mut cfg = Config::new();
    cfg.user = Some("root".to_string());
    cfg.dbname = Some(args[1].clone());
    cfg.host = Some("localhost".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = Arc::new(Mutex::new(
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap(),
    ));

    element::Element::new().await.run(pool).await;
}
