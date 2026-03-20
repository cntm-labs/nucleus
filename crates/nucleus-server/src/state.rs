use std::sync::Arc;

use nucleus_auth::jwt::SigningKeyPair;
use nucleus_auth::service::AuthService;
use nucleus_core::clock::{Clock, SystemClock};
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
}

impl AppState {
    pub fn new(
        db: PgPool,
        redis: ConnectionManager,
        master_key: [u8; 32],
        auth_service: Arc<AuthService>,
        session_service: Arc<SessionService>,
        signing_key: Arc<SigningKeyPair>,
        user_service: Arc<UserService>,
        org_service: Arc<OrgService>,
        allowed_origins: Vec<String>,
    ) -> Self {
        Self {
            db,
            redis,
            master_key,
            clock: Arc::new(SystemClock),
            auth_service,
            session_service,
            signing_key,
            user_service,
            org_service,
            allowed_origins,
        }
    }
}
