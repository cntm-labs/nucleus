use std::sync::Arc;

use nucleus_core::error::{AppError, AuthError, UserError};
use nucleus_core::types::*;
use nucleus_db::repos::audit_repo::{AuditRepository, NewSignInAttempt};
use nucleus_db::repos::credential_repo::{CredentialRepository, NewCredential};
use nucleus_db::repos::user_repo::{NewUser, User, UserRepository};

use crate::jwt::{JwtService, SigningKeyPair};
use crate::password::PasswordService;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    credential_repo: Arc<dyn CredentialRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    signing_key: Arc<SigningKeyPair>,
    issuer: String,
    jwt_lifetime_secs: i64,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        credential_repo: Arc<dyn CredentialRepository>,
        audit_repo: Arc<dyn AuditRepository>,
        signing_key: Arc<SigningKeyPair>,
        issuer: String,
        jwt_lifetime_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            credential_repo,
            audit_repo,
            signing_key,
            issuer,
            jwt_lifetime_secs,
        }
    }

    /// Sign up a new user with email and password.
    pub async fn sign_up(
        &self,
        project_id: &ProjectId,
        email: &str,
        password: &str,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<(User, String), AppError> {
        // 1. Validate email
        let email = nucleus_core::validation::validate_email(email)?;

        // 2. Check if email already taken
        if self
            .user_repo
            .find_by_email(project_id, &email)
            .await?
            .is_some()
        {
            return Err(AppError::User(UserError::EmailTaken));
        }

        // 3. Hash password (validates policy internally)
        let password_hash = PasswordService::hash(password)?;

        // 4. Create user
        let new_user = NewUser {
            email: email.clone(),
            first_name,
            last_name,
            username: None,
            external_id: None,
            phone: None,
            avatar_url: None,
            metadata: None,
        };
        let user = self.user_repo.create(project_id, &new_user).await?;

        // 5. Create credential
        let new_credential = NewCredential {
            user_id: user.id,
            credential_type: "password".to_string(),
            identifier: None,
            secret_hash: Some(password_hash),
            provider: None,
            provider_data: None,
        };
        self.credential_repo.create(&new_credential).await?;

        // 6. Build JWT
        let jwt = self.issue_jwt(&user, project_id)?;

        Ok((user, jwt))
    }

    /// Sign in with email and password.
    pub async fn sign_in(
        &self,
        project_id: &ProjectId,
        identifier: &str,
        password: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, String), AppError> {
        // 1. Find user by email
        let user = self
            .user_repo
            .find_by_email(project_id, identifier)
            .await?
            .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;

        // 2. Check if banned
        if user.banned_at.is_some() {
            self.log_sign_in_attempt(
                project_id,
                Some(&user.id),
                "password",
                "blocked",
                Some("account_banned"),
                ip.as_deref(),
                user_agent.as_deref(),
            )
            .await;
            return Err(AppError::Auth(AuthError::AccountBanned));
        }

        // 3. Check if deleted (soft)
        if user.deleted_at.is_some() {
            return Err(AppError::Auth(AuthError::InvalidCredentials));
        }

        // 4. Get password credential
        let credentials = self
            .credential_repo
            .find_by_user_and_type(&user.id, "password")
            .await?;

        let credential = credentials
            .first()
            .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;

        let hash = credential
            .secret_hash
            .as_ref()
            .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;

        // 5. Verify password
        let valid = PasswordService::verify(password, hash)?;
        if !valid {
            self.log_sign_in_attempt(
                project_id,
                Some(&user.id),
                "password",
                "failed",
                Some("invalid_password"),
                ip.as_deref(),
                user_agent.as_deref(),
            )
            .await;
            return Err(AppError::Auth(AuthError::InvalidCredentials));
        }

        // 6. Log successful attempt
        self.log_sign_in_attempt(
            project_id,
            Some(&user.id),
            "password",
            "success",
            None,
            ip.as_deref(),
            user_agent.as_deref(),
        )
        .await;

        // 7. Build JWT
        let jwt = self.issue_jwt(&user, project_id)?;

        Ok((user, jwt))
    }

    fn issue_jwt(&self, user: &User, project_id: &ProjectId) -> Result<String, AppError> {
        let claims = JwtService::build_claims(
            &user.id,
            project_id,
            &self.issuer,
            self.jwt_lifetime_secs,
            Some(user.email.clone()),
            user.first_name.clone(),
            user.last_name.clone(),
            if user.metadata == serde_json::Value::Null {
                None
            } else {
                Some(user.metadata.clone())
            },
        );
        JwtService::sign(&claims, &self.signing_key)
    }

    async fn log_sign_in_attempt(
        &self,
        project_id: &ProjectId,
        user_id: Option<&UserId>,
        method: &str,
        status: &str,
        failure_reason: Option<&str>,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) {
        let attempt = NewSignInAttempt {
            project_id: *project_id,
            user_id: user_id.copied(),
            method: method.to_string(),
            status: status.to_string(),
            failure_reason: failure_reason.map(|s| s.to_string()),
            ip: ip.map(|s| s.to_string()),
            user_agent: user_agent.map(|s| s.to_string()),
            country_code: None,
            city: None,
        };
        // Best effort — don't fail sign-in if audit logging fails
        let _ = self.audit_repo.create_sign_in_attempt(&attempt).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
    use nucleus_db::repos::audit_repo::{AuditLog, NewAuditLog, SignInAttempt};
    use nucleus_db::repos::credential_repo::Credential;
    use nucleus_db::repos::user_repo::UpdateUser;
    use std::sync::Mutex;

    // ── Mock UserRepository ──────────────────────────────────────────

    struct MockUserRepo {
        users: Mutex<Vec<User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
            }
        }

        fn with_user(user: User) -> Self {
            Self {
                users: Mutex::new(vec![user]),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, project_id: &ProjectId, new: &NewUser) -> Result<User, AppError> {
            let user = User {
                id: UserId::new(),
                project_id: *project_id,
                external_id: new.external_id.clone(),
                email: new.email.clone(),
                email_verified: false,
                phone: new.phone.clone(),
                phone_verified: false,
                username: new.username.clone(),
                first_name: new.first_name.clone(),
                last_name: new.last_name.clone(),
                avatar_url: new.avatar_url.clone(),
                metadata: new
                    .metadata
                    .clone()
                    .unwrap_or(serde_json::Value::Object(Default::default())),
                private_metadata: serde_json::Value::Object(Default::default()),
                last_sign_in_at: None,
                banned_at: None,
                deleted_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn find_by_id(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.id == *user_id)
                .cloned())
        }

        async fn find_by_email(
            &self,
            project_id: &ProjectId,
            email: &str,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.email == email)
                .cloned())
        }

        async fn find_by_username(
            &self,
            _project_id: &ProjectId,
            _username: &str,
        ) -> Result<Option<User>, AppError> {
            Ok(None)
        }

        async fn update(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
            _update: &UpdateUser,
        ) -> Result<User, AppError> {
            unimplemented!()
        }

        async fn soft_delete(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _project_id: &ProjectId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<User>, AppError> {
            unimplemented!()
        }

        async fn ban(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn unban(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    // ── Mock CredentialRepository ────────────────────────────────────

    struct MockCredentialRepo {
        credentials: Mutex<Vec<Credential>>,
    }

    impl MockCredentialRepo {
        fn new() -> Self {
            Self {
                credentials: Mutex::new(Vec::new()),
            }
        }

        fn with_credential(cred: Credential) -> Self {
            Self {
                credentials: Mutex::new(vec![cred]),
            }
        }
    }

    #[async_trait]
    impl CredentialRepository for MockCredentialRepo {
        async fn create(&self, new: &NewCredential) -> Result<Credential, AppError> {
            let cred = Credential {
                id: CredentialId::new(),
                user_id: new.user_id,
                credential_type: new.credential_type.clone(),
                identifier: new.identifier.clone(),
                secret_hash: new.secret_hash.clone(),
                provider: new.provider.clone(),
                provider_data: new.provider_data.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.credentials.lock().unwrap().push(cred.clone());
            Ok(cred)
        }

        async fn find_by_user_and_type(
            &self,
            user_id: &UserId,
            credential_type: &str,
        ) -> Result<Vec<Credential>, AppError> {
            let creds = self.credentials.lock().unwrap();
            Ok(creds
                .iter()
                .filter(|c| c.user_id == *user_id && c.credential_type == credential_type)
                .cloned()
                .collect())
        }

        async fn find_by_provider_identifier(
            &self,
            _credential_type: &str,
            _provider: &str,
            _identifier: &str,
        ) -> Result<Option<Credential>, AppError> {
            Ok(None)
        }

        async fn update_secret(
            &self,
            _id: &CredentialId,
            _new_secret_hash: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn delete(&self, _id: &CredentialId) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    // ── Mock AuditRepository ─────────────────────────────────────────

    /// Lightweight record of a sign-in attempt for test assertions.
    #[derive(Debug, Clone)]
    struct RecordedAttempt {
        status: String,
        method: String,
        failure_reason: Option<String>,
        ip: Option<String>,
        user_agent: Option<String>,
    }

    struct MockAuditRepo {
        sign_in_attempts: Mutex<Vec<RecordedAttempt>>,
    }

    impl MockAuditRepo {
        fn new() -> Self {
            Self {
                sign_in_attempts: Mutex::new(Vec::new()),
            }
        }

        fn attempts(&self) -> Vec<RecordedAttempt> {
            self.sign_in_attempts.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl AuditRepository for MockAuditRepo {
        async fn create_audit_log(&self, _log: &NewAuditLog) -> Result<AuditLog, AppError> {
            unimplemented!()
        }

        async fn list_audit_logs(
            &self,
            _project_id: &ProjectId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<AuditLog>, AppError> {
            unimplemented!()
        }

        async fn create_sign_in_attempt(
            &self,
            attempt: &NewSignInAttempt,
        ) -> Result<SignInAttempt, AppError> {
            self.sign_in_attempts.lock().unwrap().push(RecordedAttempt {
                status: attempt.status.clone(),
                method: attempt.method.clone(),
                failure_reason: attempt.failure_reason.clone(),
                ip: attempt.ip.clone(),
                user_agent: attempt.user_agent.clone(),
            });
            Ok(SignInAttempt {
                id: uuid::Uuid::new_v4(),
                project_id: attempt.project_id,
                user_id: attempt.user_id,
                method: attempt.method.clone(),
                status: attempt.status.clone(),
                failure_reason: attempt.failure_reason.clone(),
                ip: attempt.ip.clone(),
                user_agent: attempt.user_agent.clone(),
                country_code: attempt.country_code.clone(),
                city: attempt.city.clone(),
                created_at: Utc::now(),
            })
        }

        async fn list_sign_in_attempts(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<SignInAttempt>, AppError> {
            unimplemented!()
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    use crate::jwt::JwtService;

    fn test_signing_key() -> Arc<SigningKeyPair> {
        Arc::new(JwtService::generate_key_pair("test-kid").unwrap())
    }

    fn make_service(
        user_repo: Arc<dyn UserRepository>,
        cred_repo: Arc<dyn CredentialRepository>,
        audit_repo: Arc<dyn AuditRepository>,
    ) -> AuthService {
        AuthService::new(
            user_repo,
            cred_repo,
            audit_repo,
            test_signing_key(),
            "https://nucleus.test".to_string(),
            3600,
        )
    }

    fn make_test_user(project_id: &ProjectId) -> User {
        User {
            id: UserId::new(),
            project_id: *project_id,
            external_id: None,
            email: "existing@example.com".to_string(),
            email_verified: false,
            phone: None,
            phone_verified: false,
            username: None,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            avatar_url: None,
            metadata: serde_json::Value::Object(Default::default()),
            private_metadata: serde_json::Value::Object(Default::default()),
            last_sign_in_at: None,
            banned_at: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_password_credential(user_id: &UserId, password: &str) -> Credential {
        let hash = PasswordService::hash(password).unwrap();
        Credential {
            id: CredentialId::new(),
            user_id: *user_id,
            credential_type: "password".to_string(),
            identifier: None,
            secret_hash: Some(hash),
            provider: None,
            provider_data: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── Sign-up tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn sign_up_creates_user_and_returns_jwt() {
        let user_repo = Arc::new(MockUserRepo::new());
        let cred_repo = Arc::new(MockCredentialRepo::new());
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo.clone(), cred_repo.clone(), audit_repo);

        let project_id = ProjectId::new();
        let (user, jwt) = service
            .sign_up(
                &project_id,
                "new@example.com",
                "SecurePass123!",
                Some("Alice".to_string()),
                Some("Smith".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.first_name, Some("Alice".to_string()));
        assert_eq!(user.last_name, Some("Smith".to_string()));
        assert!(!jwt.is_empty());

        // Verify credential was created
        let creds = cred_repo.credentials.lock().unwrap();
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].credential_type, "password");
        assert!(creds[0].secret_hash.is_some());
    }

    #[tokio::test]
    async fn sign_up_rejects_duplicate_email() {
        let project_id = ProjectId::new();
        let existing = make_test_user(&project_id);
        let user_repo = Arc::new(MockUserRepo::with_user(existing));
        let cred_repo = Arc::new(MockCredentialRepo::new());
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_up(
                &project_id,
                "existing@example.com",
                "SecurePass123!",
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(AppError::User(UserError::EmailTaken))));
    }

    #[tokio::test]
    async fn sign_up_rejects_weak_password() {
        let user_repo = Arc::new(MockUserRepo::new());
        let cred_repo = Arc::new(MockCredentialRepo::new());
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_up(&ProjectId::new(), "new@example.com", "short", None, None)
            .await;

        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::PasswordTooWeak))
        ));
    }

    #[tokio::test]
    async fn sign_up_rejects_invalid_email() {
        let user_repo = Arc::new(MockUserRepo::new());
        let cred_repo = Arc::new(MockCredentialRepo::new());
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_up(
                &ProjectId::new(),
                "not-an-email",
                "SecurePass123!",
                None,
                None,
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::User(UserError::InvalidEmail))
        ));
    }

    // ── Sign-in tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn sign_in_with_correct_password() {
        let project_id = ProjectId::new();
        let user = make_test_user(&project_id);
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(MockUserRepo::with_user(user.clone()));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let (returned_user, jwt) = service
            .sign_in(
                &project_id,
                "existing@example.com",
                "SecurePass123!",
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(returned_user.id, user.id);
        assert!(!jwt.is_empty());
    }

    #[tokio::test]
    async fn sign_in_with_wrong_password_returns_invalid_credentials() {
        let project_id = ProjectId::new();
        let user = make_test_user(&project_id);
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(MockUserRepo::with_user(user));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_in(
                &project_id,
                "existing@example.com",
                "WrongPassword!",
                None,
                None,
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::InvalidCredentials))
        ));
    }

    #[tokio::test]
    async fn sign_in_with_nonexistent_email_returns_invalid_credentials() {
        let user_repo = Arc::new(MockUserRepo::new());
        let cred_repo = Arc::new(MockCredentialRepo::new());
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_in(
                &ProjectId::new(),
                "nobody@example.com",
                "SecurePass123!",
                None,
                None,
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::InvalidCredentials))
        ));
    }

    #[tokio::test]
    async fn sign_in_banned_user_returns_account_banned() {
        let project_id = ProjectId::new();
        let mut user = make_test_user(&project_id);
        user.banned_at = Some(Utc::now());
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(MockUserRepo::with_user(user));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo);

        let result = service
            .sign_in(
                &project_id,
                "existing@example.com",
                "SecurePass123!",
                None,
                None,
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::AccountBanned))
        ));
    }

    #[tokio::test]
    async fn sign_in_logs_attempt_on_success() {
        let project_id = ProjectId::new();
        let user = make_test_user(&project_id);
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(MockUserRepo::with_user(user));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo.clone());

        service
            .sign_in(
                &project_id,
                "existing@example.com",
                "SecurePass123!",
                Some("1.2.3.4".to_string()),
                Some("TestAgent/1.0".to_string()),
            )
            .await
            .unwrap();

        let attempts = audit_repo.attempts();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, "success");
        assert_eq!(attempts[0].method, "password");
        assert!(attempts[0].failure_reason.is_none());
        assert_eq!(attempts[0].ip, Some("1.2.3.4".to_string()));
        assert_eq!(attempts[0].user_agent, Some("TestAgent/1.0".to_string()));
    }

    #[tokio::test]
    async fn sign_in_logs_attempt_on_failure() {
        let project_id = ProjectId::new();
        let user = make_test_user(&project_id);
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(MockUserRepo::with_user(user));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        let service = make_service(user_repo, cred_repo, audit_repo.clone());

        let _ = service
            .sign_in(
                &project_id,
                "existing@example.com",
                "WrongPassword!",
                Some("5.6.7.8".to_string()),
                None,
            )
            .await;

        let attempts = audit_repo.attempts();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, "failed");
        assert_eq!(
            attempts[0].failure_reason,
            Some("invalid_password".to_string())
        );
        assert_eq!(attempts[0].ip, Some("5.6.7.8".to_string()));
    }
}
