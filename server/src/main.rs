use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use cntm_nucleus_server::account::repo::PgAccountRepository;
use cntm_nucleus_server::account::service::AccountService;
use cntm_nucleus_server::auth::jwt::{JwtService, SigningKeyPair};
use cntm_nucleus_server::auth::service::AuthService;
use cntm_nucleus_server::core::clock::SystemClock;
use cntm_nucleus_server::core::crypto;
use cntm_nucleus_server::core::notification::NotificationService;
use cntm_nucleus_server::db::pool::create_pg_pool;
use cntm_nucleus_server::db::redis::create_redis_pool;
use cntm_nucleus_server::db::repos::api_key_repo::PgApiKeyRepository;
use cntm_nucleus_server::db::repos::audit_repo::PgAuditRepository;
use cntm_nucleus_server::db::repos::credential_repo::PgCredentialRepository;
use cntm_nucleus_server::db::repos::mfa_enrollment_repo::PgMfaEnrollmentRepository;
use cntm_nucleus_server::db::repos::org_repo::PgOrgRepository;
use cntm_nucleus_server::db::repos::project_repo::PgProjectRepository;
use cntm_nucleus_server::db::repos::session_repo::RedisSessionRepository;
use cntm_nucleus_server::db::repos::signing_key_repo::PgSigningKeyRepository;
use cntm_nucleus_server::db::repos::user_repo::PgUserRepository;
use cntm_nucleus_server::db::repos::verification_token_repo::PgVerificationTokenRepository;
use cntm_nucleus_server::db::repos::SigningKeyRepository;
use cntm_nucleus_server::identity::user::UserService;
use cntm_nucleus_server::middleware::rate_limit::RateLimitConfig;
use cntm_nucleus_server::migrate::run_migrations;
use cntm_nucleus_server::org::organization::OrgService;
use cntm_nucleus_server::session::SessionService;
use cntm_nucleus_server::state::AppState;
use cntm_nucleus_server::{config, services};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
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
    let signing_key_repo_instance = PgSigningKeyRepository::new(db.clone());
    let signing_key_repo = Arc::new(PgSigningKeyRepository::new(db.clone()));
    let default_project_id = Uuid::nil(); // System-level key
    let signing_key = Arc::new(
        match signing_key_repo_instance
            .find_current(&default_project_id)
            .await?
        {
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
                    algorithm: jsonwebtoken::Algorithm::ES256,
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
                signing_key_repo_instance
                    .create(&default_project_id, "ES256", &public_pem, &encrypted)
                    .await?;
                tracing::info!("Generated and persisted new signing key");
                key
            }
        },
    );

    // Create session repository and service
    let session_repo = Arc::new(RedisSessionRepository::new(redis.clone()));
    let clock: Arc<dyn cntm_nucleus_server::core::clock::Clock> = Arc::new(SystemClock);
    let session_service = Arc::new(SessionService::new(session_repo, clock.clone()));

    // Create repositories
    let user_repo = Arc::new(PgUserRepository::new(db.clone()));
    let credential_repo = Arc::new(PgCredentialRepository::new(db.clone()));
    let audit_repo = Arc::new(PgAuditRepository::new(db.clone()));
    let token_repo = Arc::new(PgVerificationTokenRepository::new(db.clone()));
    let mfa_repo = Arc::new(PgMfaEnrollmentRepository::new(db.clone()));
    let project_repo = Arc::new(PgProjectRepository::new(db.clone()));
    let api_key_repo = Arc::new(PgApiKeyRepository::new(db.clone()));
    let audit_repo_dashboard = Arc::new(PgAuditRepository::new(db.clone()));
    let account_repo = Arc::new(PgAccountRepository::new(db.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        credential_repo.clone(),
        mfa_repo.clone(),
        audit_repo,
        session_service.clone(),
        redis.clone(),
        signing_key.clone(),
        config.issuer_url.clone(),
        config.jwt_lifetime_secs,
    ));

    // Create identity user service
    let user_service = Arc::new(UserService::new(user_repo.clone()));

    // Create organization service
    let org_repo = Arc::new(PgOrgRepository::new(db.clone()));
    let org_service = Arc::new(OrgService::new(org_repo));

    // Initialize notification service (email + SMS)
    let email_service: Arc<dyn NotificationService> = match &config.sendgrid_api_key {
        Some(key) => {
            tracing::info!("SendGrid API key configured — using SendGrid for email delivery");
            Arc::new(services::email::SendGridService::new(
                key.clone(),
                config.from_email.clone(),
                config.from_name.clone(),
            ))
        }
        None => {
            tracing::warn!("SENDGRID_API_KEY not set — using log-based email delivery");
            Arc::new(services::email::LogNotificationService)
        }
    };

    // Initialize OAuth providers and state store
    let http_client: Arc<dyn cntm_nucleus_server::auth::oauth::provider::HttpClient> =
        Arc::new(cntm_nucleus_server::auth::oauth::provider::ReqwestHttpClient::new());
    let oauth_state_store =
        Arc::new(cntm_nucleus_server::auth::oauth::state::RedisOAuthStateStore::new(redis.clone()));

    let mut oauth_providers: std::collections::HashMap<
        String,
        Arc<dyn cntm_nucleus_server::auth::oauth::provider::OAuthProvider>,
    > = std::collections::HashMap::new();

    // Google
    if let (Some(client_id), Some(client_secret)) = (
        std::env::var("GOOGLE_CLIENT_ID").ok(),
        std::env::var("GOOGLE_CLIENT_SECRET").ok(),
    ) {
        let google_config = cntm_nucleus_server::auth::oauth::provider::OAuthConfig {
            client_id,
            client_secret,
            redirect_url: format!("{}/api/v1/auth/oauth/callback", config.issuer_url),
            scopes: cntm_nucleus_server::auth::oauth::google::GoogleProvider::default_scopes(),
        };
        oauth_providers.insert(
            "google".to_string(),
            Arc::new(
                cntm_nucleus_server::auth::oauth::google::GoogleProvider::new(
                    google_config,
                    http_client.clone(),
                ),
            ),
        );
        tracing::info!("Google OAuth provider initialized");
    }

    // GitHub
    if let (Some(client_id), Some(client_secret)) = (
        std::env::var("GITHUB_CLIENT_ID").ok(),
        std::env::var("GITHUB_CLIENT_SECRET").ok(),
    ) {
        let github_config = cntm_nucleus_server::auth::oauth::provider::OAuthConfig {
            client_id,
            client_secret,
            redirect_url: format!("{}/api/v1/auth/oauth/callback", config.issuer_url),
            scopes: cntm_nucleus_server::auth::oauth::github::GitHubProvider::default_scopes(),
        };
        oauth_providers.insert(
            "github".to_string(),
            Arc::new(
                cntm_nucleus_server::auth::oauth::github::GitHubProvider::new(
                    github_config,
                    http_client.clone(),
                ),
            ),
        );
        tracing::info!("GitHub OAuth provider initialized");
    }

    let account_service = Arc::new(AccountService::new(
        account_repo,
        token_repo.clone(),
        email_service.clone(),
        audit_repo_dashboard.clone(),
        session_service.clone(),
        signing_key.clone(),
        config.issuer_url.clone(),
        config.issuer_url.clone(), // base_url
        config.jwt_lifetime_secs,
        604800, // Default 7 days session TTL
    ));

    let state = Arc::new(AppState {
        db,
        redis,
        master_key: config.master_encryption_key,
        clock,
        auth_service,
        account_service,
        session_service,
        signing_key,
        user_service,
        user_repo,
        credential_repo,
        token_repo,
        mfa_repo,
        project_repo,
        api_key_repo,
        audit_repo: audit_repo_dashboard,
        signing_key_repo,
        org_service,
        notification_service: email_service,
        oauth_providers,
        oauth_state_store,
        allowed_origins: config.allowed_origins.clone(),
        issuer_url: config.issuer_url.clone(),
        rp_name: "Nucleus".to_string(),
        rp_id: "localhost".to_string(), // TODO: Configurable
    });

    // Build router
    let auth_rate_limit = RateLimitConfig {
        max_requests: config.rate_limit_auth_max,
        window_secs: config.rate_limit_auth_window_secs,
    };
    let api_rate_limit = RateLimitConfig {
        max_requests: config.rate_limit_api_max,
        window_secs: config.rate_limit_api_window_secs,
    };
    let trusted_proxies = config.trusted_proxies.clone();

    let app = cntm_nucleus_server::router::create_router(
        state,
        auth_rate_limit,
        api_rate_limit,
        trusted_proxies,
    )
    .layer(TraceLayer::new_for_http())
    .layer(axum::middleware::from_fn(log_request));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("Nucleus server stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
}

async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let latency = start.elapsed();
    let status = response.status();

    tracing::info!(
        method = %method,
        path = %path,
        status = %status,
        latency = ?latency,
        "request processed"
    );

    response
}
