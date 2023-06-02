use deadpool_postgres::Pool;
use std::sync::{Arc, Mutex};

use crate::coordinator;

pub struct AppState {
    pub coordinator: Arc<Mutex<coordinator::Coordinator>>,
    pub pool: Arc<Mutex<Pool>>,
}
