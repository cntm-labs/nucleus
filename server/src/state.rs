use std::sync::Arc;

use crate::auth::jwt::SigningKeyPair;
use crate::auth::service::AuthService;
use crate::core::clock::Clock;
use crate::core::notification::NotificationService;
use crate::db::repos::api_key_repo::ApiKeyRepository;
use crate::db::repos::audit_repo::AuditRepository;
use crate::db::repos::credential_repo::CredentialRepository;
use crate::db::repos::mfa_enrollment_repo::MfaEnrollmentRepository;
use crate::db::repos::project_repo::ProjectRepository;
use crate::db::repos::signing_key_repo::SigningKeyRepository;
use crate::db::repos::user_repo::UserRepository;
use crate::db::repos::verification_token_repo::VerificationTokenRepository;
use crate::identity::user::UserService;
use crate::org::organization::OrgService;
use crate::session::SessionService;
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
    pub mfa_repo: Arc<dyn MfaEnrollmentRepository>,
    pub project_repo: Arc<dyn ProjectRepository>,
    pub api_key_repo: Arc<dyn ApiKeyRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub signing_key_repo: Arc<dyn SigningKeyRepository>,
    pub org_service: Arc<OrgService>,
    pub notification_service: Arc<dyn NotificationService>,
    pub allowed_origins: Vec<String>,
    pub issuer_url: String,
    pub rp_name: String,
    pub rp_id: String,
}

impl AppState {
    pub fn magic_link_state(&self) -> crate::auth::handlers::magic_link::MagicLinkState {
        crate::auth::handlers::magic_link::MagicLinkState {
            token_repo: self.token_repo.clone(),
            user_repo: self.user_repo.clone(),
            session_service: self.session_service.clone(),
            auth_service: self.auth_service.clone(),
            notification_service: self.notification_service.clone(),
            base_url: self.issuer_url.clone(),
        }
    }

    pub fn password_reset_state(
        &self,
    ) -> crate::auth::handlers::password_reset::PasswordResetState {
        crate::auth::handlers::password_reset::PasswordResetState {
            token_repo: self.token_repo.clone(),
            user_repo: self.user_repo.clone(),
            credential_repo: self.credential_repo.clone(),
            session_service: self.session_service.clone(),
            notification_service: self.notification_service.clone(),
            base_url: self.issuer_url.clone(),
        }
    }

    pub fn otp_state(&self) -> crate::auth::handlers::otp::OtpState {
        crate::auth::handlers::otp::OtpState {
            redis: self.redis.clone(),
            user_repo: self.user_repo.clone(),
            session_service: self.session_service.clone(),
            auth_service: self.auth_service.clone(),
            notification_service: self.notification_service.clone(),
        }
    }

    pub fn dashboard_state(&self) -> crate::api::handlers::dashboard::DashboardState {
        crate::api::handlers::dashboard::DashboardState {
            project_repo: self.project_repo.clone(),
            api_key_repo: self.api_key_repo.clone(),
            audit_repo: self.audit_repo.clone(),
            signing_key_repo: self.signing_key_repo.clone(),
            master_key: self.master_key,
        }
    }

    pub fn mfa_state(&self) -> crate::auth::handlers::mfa::MfaState {
        crate::auth::handlers::mfa::MfaState {
            mfa_repo: self.mfa_repo.clone(),
            user_repo: self.user_repo.clone(),
            redis: self.redis.clone(),
            session_service: self.session_service.clone(),
            auth_service: self.auth_service.clone(),
            master_key: self.master_key,
        }
    }
}
