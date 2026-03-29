mod config;
mod handlers;
mod middleware;
mod router;
mod state;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use nucleus_auth::jwt::{JwtService, SigningKeyPair};
use nucleus_auth::service::AuthService;
use nucleus_core::clock::SystemClock;
use nucleus_core::crypto;
use nucleus_db::pool::create_pg_pool;
use nucleus_db::redis::create_redis_pool;
use nucleus_db::repos::audit_repo::PgAuditRepository;
use nucleus_db::repos::credential_repo::PgCredentialRepository;
use nucleus_db::repos::mfa_enrollment_repo::PgMfaEnrollmentRepository;
use nucleus_db::repos::org_repo::PgOrgRepository;
use nucleus_db::repos::session_repo::RedisSessionRepository;
use nucleus_db::repos::signing_key_repo::{PgSigningKeyRepository, SigningKeyRepository};
use nucleus_db::repos::user_repo::PgUserRepository;
use nucleus_db::repos::verification_token_repo::PgVerificationTokenRepository;
use nucleus_identity::user::UserService;
use nucleus_migrate::run_migrations;
use nucleus_org::organization::OrgService;
use nucleus_session::session::SessionService;
use uuid::Uuid;

use crate::config::Config;
use crate::router::create_router;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
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

    // Load or create signing key pair for JWT
    let signing_key_repo = PgSigningKeyRepository::new(db.clone());
    let default_project_id = Uuid::nil(); // System-level key
    let signing_key = Arc::new(
        match signing_key_repo.find_current(&default_project_id).await? {
            Some(row) => {
                let private_pem = crypto::decrypt(
                    &hex::decode(&row.private_key_enc)
                        .map_err(|e| anyhow::anyhow!("hex decode error: {}", e))?,
                    &config.master_encryption_key,
                )?;
                tracing::info!("Loaded existing signing key {}", row.id);
                SigningKeyPair {
                    kid: row.id.to_string(),
                    private_key_pem: private_pem,
                    public_key_pem: row.public_key.as_bytes().to_vec(),
                    algorithm: jsonwebtoken::Algorithm::RS256,
                }
            }
            None => {
                let key = JwtService::generate_key_pair(&Uuid::new_v4().to_string())?;
                let encrypted = hex::encode(crypto::encrypt(
                    &key.private_key_pem,
                    &config.master_encryption_key,
                )?);
                let public_pem = String::from_utf8(key.public_key_pem.clone())
                    .map_err(|e| anyhow::anyhow!("utf8 error: {}", e))?;
                signing_key_repo
                    .create(&default_project_id, "RS256", &public_pem, &encrypted)
                    .await?;
                tracing::info!("Generated and persisted new signing key");
                key
            }
        },
    );

    // Create session repository and service
    let session_repo = Arc::new(RedisSessionRepository::new(redis.clone()));
    let clock: Arc<dyn nucleus_core::clock::Clock> = Arc::new(SystemClock);
    let session_service = Arc::new(SessionService::new(session_repo, clock));

    // Create repositories
    let user_repo = Arc::new(PgUserRepository::new(db.clone()));
    let credential_repo = Arc::new(PgCredentialRepository::new(db.clone()));
    let audit_repo = Arc::new(PgAuditRepository::new(db.clone()));
    let token_repo = Arc::new(PgVerificationTokenRepository::new(db.clone()));
    let mfa_repo = Arc::new(PgMfaEnrollmentRepository::new(db.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        credential_repo.clone(),
        audit_repo,
        signing_key.clone(),
        config.issuer_url.clone(),
        config.jwt_lifetime_secs,
    ));

    // Create identity user service
    let user_service = Arc::new(UserService::new(user_repo.clone()));

    // Create organization service
    let org_repo = Arc::new(PgOrgRepository::new(db.clone()));
    let org_service = Arc::new(OrgService::new(org_repo));

    // Capture bind address before moving config fields
    let bind_addr = config.bind_addr();

    // Build application state
    let state = Arc::new(AppState {
        db,
        redis,
        master_key: config.master_encryption_key,
        clock: Arc::new(SystemClock),
        auth_service,
        session_service,
        signing_key,
        user_service,
        user_repo,
        credential_repo,
        token_repo,
        mfa_repo,
        org_service,
        allowed_origins: config.allowed_origins,
        issuer_url: config.issuer_url,
        rp_name: config.rp_name,
        rp_id: config.rp_id,
    });

    // Build router with configurable rate limits
    let auth_rate_limit = crate::middleware::rate_limit::RateLimitConfig {
        max_requests: config.rate_limit_auth_max,
        window_secs: config.rate_limit_auth_window_secs,
    };
    let api_rate_limit = crate::middleware::rate_limit::RateLimitConfig {
        max_requests: config.rate_limit_api_max,
        window_secs: config.rate_limit_api_window_secs,
    };
    let app = create_router(state, auth_rate_limit, api_rate_limit);
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
