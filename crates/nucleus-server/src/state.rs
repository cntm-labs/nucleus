use std::sync::Arc;

use nucleus_core::clock::{Clock, SystemClock};
use redis::aio::ConnectionManager;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub master_key: [u8; 32],
    pub clock: Arc<dyn Clock>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        redis: ConnectionManager,
        master_key: [u8; 32],
    ) -> Self {
        Self {
            db,
            redis,
            master_key,
            clock: Arc::new(SystemClock),
        }
    }
}
