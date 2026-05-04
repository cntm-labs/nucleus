//! AccountService — orchestrates account auth using existing primitives.

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::jwt::{JwtService, SigningKeyPair};
use crate::auth::password::PasswordService;
use crate::core::crypto;
use crate::core::error::{AccountError, AppError};
use crate::core::notification::NotificationService;
use crate::core::types::ProjectId;
use crate::db::repos::audit_repo::{AuditRepository, NewAuditLog};
use crate::db::repos::session_repo::Session;
use crate::db::repos::verification_token_repo::VerificationTokenRepository;
use crate::session::{DeviceInfo, SessionService};

use crate::account::repo::{Account, AccountRepository, NewAccount};

const VERIFICATION_TOKEN_TTL_HOURS: i64 = 24;
const VERIFICATION_TOKEN_TYPE: &str = "account_email_verification";

pub struct AccountService {
    repo: Arc<dyn AccountRepository>,
    verification_token_repo: Arc<dyn VerificationTokenRepository>,
    notification: Arc<dyn NotificationService>,
    audit: Arc<dyn AuditRepository>,
    session_service: Arc<SessionService>,
    jwt_signing_key: Arc<SigningKeyPair>,
    issuer: String,
    base_url: String,
    jwt_lifetime_secs: i64,
    session_ttl_secs: u64,
}

impl AccountService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repo: Arc<dyn AccountRepository>,
        verification_token_repo: Arc<dyn VerificationTokenRepository>,
        notification: Arc<dyn NotificationService>,
        audit: Arc<dyn AuditRepository>,
        session_service: Arc<SessionService>,
        jwt_signing_key: Arc<SigningKeyPair>,
        issuer: String,
        base_url: String,
        jwt_lifetime_secs: i64,
        session_ttl_secs: u64,
    ) -> Self {
        Self {
            repo,
            verification_token_repo,
            notification,
            audit,
            session_service,
            jwt_signing_key,
            issuer,
            base_url,
            jwt_lifetime_secs,
            session_ttl_secs,
        }
    }

    /// Sign up a new account. Account starts with `email_verified = false` and a
    /// verification email is dispatched. The caller does NOT receive a JWT — the
    /// account must verify its email and call sign_in.
    pub async fn sign_up_account(
        &self,
        email: &str,
        password: &str,
        name: &str,
        company: Option<&str>,
    ) -> Result<Account, AppError> {
        let email = crate::core::validation::validate_email(email)?;
        let password_hash = PasswordService::hash(password)?;

        let account = self
            .repo
            .create(&NewAccount {
                email: email.clone(),
                password_hash,
                name: name.to_string(),
                company: company.map(|s| s.to_string()),
            })
            .await?;

        let token = crypto::generate_token();
        let token_hash = crypto::generate_token_hash(&token);
        let expires_at = Utc::now() + Duration::hours(VERIFICATION_TOKEN_TTL_HOURS);

        self.verification_token_repo
            .create(
                account.id.0,
                uuid::Uuid::nil(),
                VERIFICATION_TOKEN_TYPE,
                &token_hash,
                None,
                expires_at,
            )
            .await?;

        let verify_url = format!("{}/verify-email?token={}", self.base_url, token);
        let html_body = format!(
            "<p>Welcome to Nucleus, {name}!</p>\
             <p>Click <a href=\"{verify_url}\">here</a> to verify your email and activate your account.</p>\
             <p>This link expires in {VERIFICATION_TOKEN_TTL_HOURS} hours.</p>"
        );
        let text_body = format!(
            "Welcome to Nucleus, {name}!\n\n\
             Click here to verify your email and activate your account:\n{verify_url}\n\n\
             This link expires in {VERIFICATION_TOKEN_TTL_HOURS} hours."
        );
        let _ = self
            .notification
            .send_email(
                &account.email,
                "Verify your Nucleus account",
                &html_body,
                &text_body,
            )
            .await;

        let _ = self
            .audit
            .create_audit_log(&NewAuditLog {
                project_id: ProjectId::from_uuid(uuid::Uuid::nil()),
                actor_type: "account".to_string(),
                actor_id: Some(account.id.0),
                action: "account.signed_up".to_string(),
                target_type: Some("account".to_string()),
                target_id: Some(account.id.0),
                metadata: None,
                ip: None,
                user_agent: None,
            })
            .await;

        Ok(account)
    }

    /// Sign in an account. Returns (Account, jwt, Session) on success.
    /// Anti-enumeration: any failure returns InvalidCredentials except EmailNotVerified.
    pub async fn sign_in_account(
        &self,
        email: &str,
        password: &str,
        device: DeviceInfo,
    ) -> Result<(Account, String, Session), AppError> {
        let email = email.trim().to_lowercase();

        let account = match self.repo.find_by_email(&email).await? {
            Some(a) => a,
            None => return Err(AppError::Account(AccountError::InvalidCredentials)),
        };

        let password_hash = self.repo.get_password_hash(&account.id).await?;
        let ok = PasswordService::verify(password, &password_hash)?;
        if !ok {
            let _ = self
                .audit
                .create_audit_log(&NewAuditLog {
                    project_id: ProjectId::from_uuid(uuid::Uuid::nil()),
                    actor_type: "account".to_string(),
                    actor_id: Some(account.id.0),
                    action: "account.sign_in_failed".to_string(),
                    target_type: Some("account".to_string()),
                    target_id: Some(account.id.0),
                    metadata: Some(serde_json::json!({ "reason": "wrong_password" })),
                    ip: None,
                    user_agent: None,
                })
                .await;
            return Err(AppError::Account(AccountError::InvalidCredentials));
        }

        if !account.email_verified {
            return Err(AppError::Account(AccountError::EmailNotVerified));
        }

        if !account.is_active {
            return Err(AppError::Account(AccountError::InvalidCredentials));
        }

        let (_session_token, session) = self
            .session_service
            .create_account_session(&account.id, device, self.session_ttl_secs)
            .await?;

        let claims = JwtService::build_account_claims(
            &account.id,
            &self.issuer,
            self.jwt_lifetime_secs,
            Some(account.email.clone()),
        );
        let jwt = JwtService::sign(&claims, &self.jwt_signing_key)?;

        let _ = self
            .audit
            .create_audit_log(&NewAuditLog {
                project_id: ProjectId::from_uuid(uuid::Uuid::nil()),
                actor_type: "account".to_string(),
                actor_id: Some(account.id.0),
                action: "account.signed_in".to_string(),
                target_type: Some("account".to_string()),
                target_id: Some(account.id.0),
                metadata: Some(serde_json::json!({ "session_id": session.id.to_string() })),
                ip: None,
                user_agent: None,
            })
            .await;

        Ok((account, jwt, session))
    }

    /// Mark an account's email as verified. Token is single-use.
    pub async fn verify_email(&self, token: &str) -> Result<Account, AppError> {
        let token_hash = crypto::generate_token_hash(token);
        let record = self
            .verification_token_repo
            .find_by_hash(&token_hash, VERIFICATION_TOKEN_TYPE)
            .await?
            .ok_or(AppError::Account(AccountError::TokenInvalid))?;

        if record.expires_at < Utc::now() {
            return Err(AppError::Account(AccountError::TokenExpired));
        }
        if record.used_at.is_some() {
            return Err(AppError::Account(AccountError::TokenInvalid));
        }

        let account_id = crate::core::types::AccountId::from_uuid(record.user_id);
        self.repo.mark_email_verified(&account_id).await?;
        self.verification_token_repo.mark_used(record.id).await?;

        let _ = self
            .audit
            .create_audit_log(&NewAuditLog {
                project_id: ProjectId::from_uuid(uuid::Uuid::nil()),
                actor_type: "account".to_string(),
                actor_id: Some(account_id.0),
                action: "account.email_verified".to_string(),
                target_type: Some("account".to_string()),
                target_id: Some(account_id.0),
                metadata: None,
                ip: None,
                user_agent: None,
            })
            .await;

        self.repo
            .find_by_id(&account_id)
            .await?
            .ok_or(AppError::Account(AccountError::NotFound))
    }

    /// Get the current account profile.
    pub async fn get_me(
        &self,
        account_id: &crate::core::types::AccountId,
    ) -> Result<Account, AppError> {
        self.repo
            .find_by_id(account_id)
            .await?
            .ok_or(AppError::Account(AccountError::NotFound))
    }

    /// Sign out — revoke session + revoke JWT.
    /// SessionService::revoke_session takes &UserId; AccountId.0 == UserId.0 (both Uuid).
    pub async fn sign_out(
        &self,
        session_id: &crate::core::types::SessionId,
        account_id: &crate::core::types::AccountId,
        jti: Option<&str>,
        jwt_ttl_secs: u64,
    ) -> Result<(), AppError> {
        let user_id_proxy = crate::core::types::UserId::from_uuid(account_id.0);
        self.session_service
            .revoke_session(session_id, &user_id_proxy, jti, jwt_ttl_secs)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::repo::tests::MockAccountRepo;
    use crate::core::clock::MockClock;
    use crate::core::error::AccountError;
    use crate::core::notification::tests::MockNotificationService;
    use crate::db::repos::audit_repo::tests::MockAuditRepo;
    use crate::db::repos::verification_token_repo::tests::MockVerificationTokenRepo;
    use crate::session::tests::MockSessionRepo;

    fn make_service() -> (
        AccountService,
        Arc<MockAccountRepo>,
        Arc<MockNotificationService>,
    ) {
        let repo = Arc::new(MockAccountRepo::new());
        let token_repo: Arc<dyn VerificationTokenRepository> =
            Arc::new(MockVerificationTokenRepo::new());
        let notif = Arc::new(MockNotificationService::new());
        let audit: Arc<dyn AuditRepository> = Arc::new(MockAuditRepo::new());
        let session_repo = Arc::new(MockSessionRepo::new());
        let clock = Arc::new(MockClock { now: Utc::now() });
        let session_service = Arc::new(SessionService::new(session_repo, clock));
        let signing_key = Arc::new(JwtService::generate_key_pair("test-kid").unwrap());
        let svc = AccountService::new(
            repo.clone() as Arc<dyn AccountRepository>,
            token_repo,
            notif.clone() as Arc<dyn NotificationService>,
            audit,
            session_service,
            signing_key,
            "https://nucleus.test".to_string(),
            "https://dashboard.example.test".to_string(),
            300,
            3600,
        );
        (svc, repo, notif)
    }

    #[tokio::test]
    async fn sign_up_creates_unverified_account_and_sends_email() {
        let (svc, _repo, notif) = make_service();
        let account = svc
            .sign_up_account(
                "alice@example.test",
                "Strong-Password-123!",
                "Alice",
                Some("Acme"),
            )
            .await
            .unwrap();

        assert_eq!(account.email, "alice@example.test");
        assert!(
            !account.email_verified,
            "fresh account should be unverified"
        );

        let emails = notif.sent_emails();
        assert_eq!(emails.len(), 1, "one verification email expected");
        let (to, subject, html_body, _text_body) = &emails[0];
        assert_eq!(to, "alice@example.test");
        assert!(subject.contains("Verify"));
        assert!(html_body.contains("/verify-email?token="));
    }

    #[tokio::test]
    async fn sign_up_rejects_duplicate_email() {
        let (svc, _repo, _notif) = make_service();
        svc.sign_up_account("dup@example.test", "Strong-Password-123!", "First", None)
            .await
            .unwrap();
        let err = svc
            .sign_up_account("dup@example.test", "Strong-Password-123!", "Second", None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Account(AccountError::EmailTaken)));
    }

    #[tokio::test]
    async fn sign_up_rejects_invalid_email() {
        let (svc, _repo, _notif) = make_service();
        let err = svc
            .sign_up_account("not-an-email", "Strong-Password-123!", "Alice", None)
            .await
            .unwrap_err();
        // validate_email returns a validation error variant
        assert!(
            format!("{err:?}").to_lowercase().contains("validation")
                || format!("{err:?}").to_lowercase().contains("email"),
            "expected validation/email error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn sign_in_succeeds_when_account_verified() {
        let (svc, repo, _notif) = make_service();
        let account = svc
            .sign_up_account("verified@example.test", "Strong-Password-123!", "V", None)
            .await
            .unwrap();
        repo.mark_email_verified(&account.id).await.unwrap();

        let (signed_in, jwt, session) = svc
            .sign_in_account(
                "verified@example.test",
                "Strong-Password-123!",
                DeviceInfo::default(),
            )
            .await
            .unwrap();

        assert_eq!(signed_in.id, account.id);
        assert!(!jwt.is_empty(), "JWT must be issued");
        assert_eq!(
            session.user_id.0, account.id.0,
            "session bound to account_id"
        );
    }

    #[tokio::test]
    async fn sign_in_fails_on_wrong_password() {
        let (svc, repo, _notif) = make_service();
        let account = svc
            .sign_up_account("wp@example.test", "Strong-Password-123!", "WP", None)
            .await
            .unwrap();
        repo.mark_email_verified(&account.id).await.unwrap();

        let err = svc
            .sign_in_account("wp@example.test", "Wrong-Password!", DeviceInfo::default())
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Account(AccountError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn sign_in_fails_on_unverified_account() {
        let (svc, _repo, _notif) = make_service();
        svc.sign_up_account("unverified@example.test", "Strong-Password-123!", "U", None)
            .await
            .unwrap();
        // do NOT mark verified

        let err = svc
            .sign_in_account(
                "unverified@example.test",
                "Strong-Password-123!",
                DeviceInfo::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Account(AccountError::EmailNotVerified)
        ));
    }

    #[tokio::test]
    async fn verify_email_marks_account_verified_and_consumes_token() {
        let (svc, _repo, notif) = make_service();
        let account = svc
            .sign_up_account("verify@example.test", "Strong-Password-123!", "V", None)
            .await
            .unwrap();
        assert!(!account.email_verified, "fresh account starts unverified");

        // Extract token from the verification email body (mock captured the URL).
        let emails = notif.sent_emails();
        let body = &emails[0].2; // (to, subject, html_body, text_body)
        let token = body
            .split("token=")
            .nth(1)
            .expect("verification URL contains token=...")
            .split(|c: char| c == '\"' || c.is_whitespace())
            .next()
            .expect("token segment");

        let updated = svc.verify_email(token).await.unwrap();
        assert!(
            updated.email_verified,
            "after verify_email, account.email_verified must be true"
        );

        // Second use of the same token must fail (single-use).
        let err = svc.verify_email(token).await.unwrap_err();
        assert!(matches!(err, AppError::Account(AccountError::TokenInvalid)));
    }

    #[tokio::test]
    async fn verify_email_fails_on_unknown_token() {
        let (svc, _repo, _notif) = make_service();
        let err = svc.verify_email("does_not_exist").await.unwrap_err();
        assert!(matches!(err, AppError::Account(AccountError::TokenInvalid)));
    }
}
