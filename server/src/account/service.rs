//! AccountService — orchestrates account auth using existing primitives.

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::password::PasswordService;
use crate::core::crypto;
use crate::core::error::AppError;
use crate::core::notification::NotificationService;
use crate::core::types::ProjectId;
use crate::db::repos::audit_repo::{AuditRepository, NewAuditLog};
use crate::db::repos::verification_token_repo::VerificationTokenRepository;

use crate::account::repo::{Account, AccountRepository, NewAccount};

const VERIFICATION_TOKEN_TTL_HOURS: i64 = 24;
const VERIFICATION_TOKEN_TYPE: &str = "account_email_verification";

pub struct AccountService {
    repo: Arc<dyn AccountRepository>,
    verification_token_repo: Arc<dyn VerificationTokenRepository>,
    notification: Arc<dyn NotificationService>,
    audit: Arc<dyn AuditRepository>,
    base_url: String,
}

impl AccountService {
    pub fn new(
        repo: Arc<dyn AccountRepository>,
        verification_token_repo: Arc<dyn VerificationTokenRepository>,
        notification: Arc<dyn NotificationService>,
        audit: Arc<dyn AuditRepository>,
        base_url: String,
    ) -> Self {
        Self {
            repo,
            verification_token_repo,
            notification,
            audit,
            base_url,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::repo::tests::MockAccountRepo;
    use crate::core::error::AccountError;
    use crate::core::notification::tests::MockNotificationService;
    use crate::db::repos::audit_repo::tests::MockAuditRepo;
    use crate::db::repos::verification_token_repo::tests::MockVerificationTokenRepo;

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
        let svc = AccountService::new(
            repo.clone() as Arc<dyn AccountRepository>,
            token_repo,
            notif.clone() as Arc<dyn NotificationService>,
            audit,
            "https://dashboard.example.test".to_string(),
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
}
