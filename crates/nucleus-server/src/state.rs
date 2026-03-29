use std::sync::Arc;

use nucleus_auth::jwt::SigningKeyPair;
use nucleus_auth::service::AuthService;
use nucleus_core::clock::Clock;
use nucleus_db::repos::credential_repo::CredentialRepository;
use nucleus_db::repos::user_repo::UserRepository;
use nucleus_db::repos::verification_token_repo::VerificationTokenRepository;
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
    pub user_repo: Arc<dyn UserRepository>,
    pub credential_repo: Arc<dyn CredentialRepository>,
    pub token_repo: Arc<dyn VerificationTokenRepository>,
    pub org_service: Arc<OrgService>,
    pub allowed_origins: Vec<String>,
    pub issuer_url: String,
    pub rp_name: String,
    pub rp_id: String,
}

impl AppState {
    pub fn magic_link_state(&self) -> nucleus_auth::handlers::magic_link::MagicLinkState {
        nucleus_auth::handlers::magic_link::MagicLinkState {
            token_repo: self.token_repo.clone(),
            user_repo: self.user_repo.clone(),
            session_service: self.session_service.clone(),
            auth_service: self.auth_service.clone(),
        }
    }

    pub fn password_reset_state(
        &self,
    ) -> nucleus_auth::handlers::password_reset::PasswordResetState {
        nucleus_auth::handlers::password_reset::PasswordResetState {
            token_repo: self.token_repo.clone(),
            user_repo: self.user_repo.clone(),
            credential_repo: self.credential_repo.clone(),
            session_service: self.session_service.clone(),
        }
    }

    pub fn otp_state(&self) -> nucleus_auth::handlers::otp::OtpState {
        nucleus_auth::handlers::otp::OtpState {
            redis: self.redis.clone(),
            user_repo: self.user_repo.clone(),
            session_service: self.session_service.clone(),
            auth_service: self.auth_service.clone(),
        }
    }
}
