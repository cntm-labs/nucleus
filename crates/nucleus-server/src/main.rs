mod config;
mod middleware;
mod router;
mod state;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use nucleus_db::pool::create_pg_pool;
use nucleus_db::redis::create_redis_pool;
use nucleus_migrate::run_migrations;

use crate::config::Config;
use crate::router::create_router;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
        )
        .json()
        .init();

    tracing::info!("Starting Nucleus server...");

    // Connect to databases
    let db = create_pg_pool(&config.database_url).await?;
    tracing::info!("Connected to PostgreSQL");

    // Run migrations
    run_migrations(&db).await?;
    tracing::info!("Database migrations complete");

    // Connect to Redis
    let redis = create_redis_pool(&config.redis_url).await?;
    tracing::info!("Connected to Redis");

    // Build application state
    let state = Arc::new(AppState::new(db, redis, config.master_encryption_key));

    // Build router
    let app = create_router(state);

    // Start server
    let bind_addr = config.bind_addr();
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Nucleus server listening on {}", bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Nucleus server stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
    tracing::info!("Shutdown signal received");
}
