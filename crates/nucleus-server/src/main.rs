mod config;
mod handlers;
mod middleware;
mod router;
mod state;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use nucleus_auth::jwt::JwtService;
use nucleus_auth::service::AuthService;
use nucleus_core::clock::SystemClock;
use nucleus_db::pool::create_pg_pool;
use nucleus_db::redis::create_redis_pool;
use nucleus_db::repos::audit_repo::PgAuditRepository;
use nucleus_db::repos::credential_repo::PgCredentialRepository;
use nucleus_db::repos::session_repo::RedisSessionRepository;
use nucleus_db::repos::org_repo::PgOrgRepository;
use nucleus_db::repos::user_repo::PgUserRepository;
use nucleus_identity::user::UserService;
use nucleus_migrate::run_migrations;
use nucleus_org::organization::OrgService;
use nucleus_session::session::SessionService;

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

    // Generate signing key pair for JWT
    let signing_key = Arc::new(
        JwtService::generate_key_pair("nucleus-key-1")
            .expect("Failed to generate signing key"),
    );

    // Create session repository and service
    let session_repo = Arc::new(RedisSessionRepository::new(redis.clone()));
    let clock: Arc<dyn nucleus_core::clock::Clock> = Arc::new(SystemClock);
    let session_service = Arc::new(SessionService::new(session_repo, clock));

    // Create auth service repositories
    let user_repo = Arc::new(PgUserRepository::new(db.clone()));
    let credential_repo = Arc::new(PgCredentialRepository::new(db.clone()));
    let audit_repo = Arc::new(PgAuditRepository::new(db.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        credential_repo,
        audit_repo,
        signing_key.clone(),
        "https://nucleus.local".to_string(),
        300, // 5 min JWT lifetime
    ));

    // Create identity user service
    let user_service = Arc::new(UserService::new(user_repo));

    // Create organization service
    let org_repo = Arc::new(PgOrgRepository::new(db.clone()));
    let org_service = Arc::new(OrgService::new(org_repo));

    // Build application state
    let state = Arc::new(AppState::new(
        db,
        redis,
        config.master_encryption_key,
        auth_service,
        session_service,
        signing_key,
        user_service,
        org_service,
    ));

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
