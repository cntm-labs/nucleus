use std::sync::Arc;

use nucleus_auth::jwt::SigningKeyPair;
use nucleus_auth::service::AuthService;
use nucleus_core::clock::Clock;
use nucleus_identity::user::UserService;
use nucleus_org::organization::OrgService;
use nucleus_session::session::SessionService;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub master_key: [u8; 32],
    pub clock: Arc<dyn Clock>,
    pub auth_service: Arc<AuthService>,
    pub session_service: Arc<SessionService>,
    pub signing_key: Arc<SigningKeyPair>,
    pub user_service: Arc<UserService>,
    pub org_service: Arc<OrgService>,
    pub allowed_origins: Vec<String>,
    pub issuer_url: String,
    pub rp_name: String,
    pub rp_id: String,
}
