use std::sync::Arc;

use crate::auth::jwt::{JwtService, SigningKeyPair};
use crate::auth::password::PasswordService;
use crate::core::crypto;
use crate::core::error::{AppError, AuthError, UserError};
#[allow(unused_imports)]
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::core::types::{ProjectId, SessionId, UserId};
#[allow(unused_imports)]
use crate::db::repos::audit_repo::{AuditRepository, NewSignInAttempt, SignInAttempt};
use crate::db::repos::credential_repo::{CredentialRepository, NewCredential};
use crate::db::repos::mfa_enrollment_repo::MfaEnrollmentRepository;
use crate::db::repos::user_repo::{NewUser, User, UserRepository};
use crate::session::{DeviceInfo, SessionService};
#[allow(unused_imports)]
use async_trait::async_trait;
use chrono::Utc;
use redis::aio::ConnectionManager;

pub struct AuthService {
    pub(crate) user_repo: Arc<dyn UserRepository>,
    pub(crate) credential_repo: Arc<dyn CredentialRepository>,
    pub(crate) mfa_repo: Arc<dyn MfaEnrollmentRepository>,
    pub(crate) audit_repo: Arc<dyn AuditRepository>,
    pub(crate) session_service: Arc<SessionService>,
    pub(crate) redis: ConnectionManager,
    pub(crate) signing_key: Arc<SigningKeyPair>,
    pub(crate) issuer: String,
    pub(crate) jwt_lifetime_secs: i64,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        credential_repo: Arc<dyn CredentialRepository>,
        mfa_repo: Arc<dyn MfaEnrollmentRepository>,
        audit_repo: Arc<dyn AuditRepository>,
        session_service: Arc<SessionService>,
        redis: ConnectionManager,
        signing_key: Arc<SigningKeyPair>,
        issuer: String,
        jwt_lifetime_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            credential_repo,
            mfa_repo,
            audit_repo,
            session_service,
            redis,
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
    ) -> Result<(User, String, String), AppError> {
        let email = crate::core::validation::validate_email(email)?;
        crate::core::validation::validate_password(password)?;

        // 1. Check if user already exists
        if self
            .user_repo
            .find_by_email(project_id, &email)
            .await?
            .is_some()
        {
            return Err(AppError::User(UserError::EmailTaken));
        }

        // 2. Create user
        let new_user = NewUser {
            email,
            username: None,
            first_name,
            last_name,
            external_id: None,
            phone: None,
            avatar_url: None,
            metadata: None,
        };
        let user = self.user_repo.create(project_id, &new_user).await?;

        // 3. Create password credential
        let hash = PasswordService::hash(password)?;
        let new_cred = NewCredential {
            user_id: user.id,
            credential_type: "password".to_string(),
            identifier: None,
            secret_hash: Some(hash),
            provider: None,
            provider_data: None,
        };
        self.credential_repo.create(&new_cred).await?;

        // 4. Create session
        let (_session_token, session) = self
            .session_service
            .create_session(&user.id, project_id, DeviceInfo::default(), 3600)
            .await?;

        // 5. Issue JWT
        let jwt = self.issue_jwt(&user, project_id, &session.id)?;

        Ok((user, jwt, session.id.to_string()))
    }

    /// Sign in with email and password.
    pub async fn sign_in(
        &self,
        project_id: &ProjectId,
        identifier: &str,
        password: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, String, String), AppError> {
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

        // 6. Check for active MFA enrollments
        let mfa_methods = self.mfa_repo.list_active_by_user(user.id.0).await?;
        if !mfa_methods.is_empty() {
            self.log_sign_in_attempt(
                project_id,
                Some(&user.id),
                "password",
                "mfa_required",
                None,
                ip.as_deref(),
                user_agent.as_deref(),
            )
            .await;

            // Generate a challenge ID and store the intent in Redis
            let challenge_id = crypto::generate_token();
            let challenge_key = format!("mfa_challenge:{}", challenge_id);

            let challenge_data = serde_json::json!({
                "user_id": user.id,
                "project_id": project_id,
                "created_at": Utc::now()
            });

            let json = serde_json::to_string(&challenge_data)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?;

            let mut conn = self.redis.clone();
            use redis::AsyncCommands;
            conn.set_ex::<_, _, ()>(&challenge_key, &json, 600)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

            return Err(AppError::Auth(AuthError::MfaRequired {
                mfa_id: challenge_id,
            }));
        }

        // 7. Log successful attempt
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

        // 8. Create session
        let (_session_token, session) = self
            .session_service
            .create_session(&user.id, project_id, DeviceInfo::default(), 3600)
            .await?;

        // 9. Build JWT
        let jwt = self.issue_jwt(&user, project_id, &session.id)?;

        Ok((user, jwt, session.id.to_string()))
    }

    /// Refresh a JWT using a valid session.
    pub async fn refresh_token(
        &self,
        session_service: &SessionService,
        session_id: &SessionId,
        project_id: &ProjectId,
    ) -> Result<(String, i64), AppError> {
        // 1. Validate session in Redis
        let session = session_service.validate_session(session_id).await?;

        // 2. Find user
        let user = self
            .user_repo
            .find_by_id(project_id, &session.user_id)
            .await?
            .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;

        // 3. Check not banned
        if user.banned_at.is_some() {
            session_service
                .revoke_session(session_id, &user.id, None, 300)
                .await?;
            return Err(AppError::Auth(AuthError::AccountBanned));
        }

        // 4. Touch session (update last_active)
        session_service.touch_session(session_id).await?;

        // 5. Issue new JWT
        let jwt = self.issue_jwt(&user, project_id, session_id)?;

        Ok((jwt, self.jwt_lifetime_secs))
    }

    /// Sign out — revoke current session.
    pub async fn sign_out(
        &self,
        session_service: &SessionService,
        session_id: &SessionId,
        user_id: &UserId,
        jti: Option<&str>,
    ) -> Result<(), AppError> {
        session_service
            .revoke_session(session_id, user_id, jti, self.jwt_lifetime_secs as u64)
            .await
    }

    /// Sign out from all devices — revoke all sessions.
    pub async fn sign_out_all(
        &self,
        session_service: &SessionService,
        user_id: &UserId,
    ) -> Result<u64, AppError> {
        session_service.revoke_all_sessions(user_id).await
    }

    /// Expose user repo for OAuth handler.
    pub fn user_repo(&self) -> &dyn UserRepository {
        self.user_repo.as_ref()
    }

    /// Expose credential repo for OAuth handler.
    pub fn credential_repo(&self) -> &dyn CredentialRepository {
        self.credential_repo.as_ref()
    }

    /// Issue a JWT for a given user (public for OAuth handler use).
    pub fn issue_jwt_for_user(
        &self,
        user: &User,
        project_id: &ProjectId,
        session_id: &SessionId,
    ) -> Result<String, AppError> {
        self.issue_jwt(user, project_id, session_id)
    }

    fn issue_jwt(
        &self,
        user: &User,
        project_id: &ProjectId,
        session_id: &SessionId,
    ) -> Result<String, AppError> {
        let claims = JwtService::build_claims(
            &user.id,
            project_id,
            session_id,
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
        let _ = self.audit_repo.create_sign_in_attempt(&attempt).await;
    }
}

#[cfg(test)]
#[allow(unused)]
#[allow(dead_code)]
pub(crate) mod tests {
    use super::*;
    use crate::core::pagination::{PaginatedResponse, PaginationParams};
    use crate::db::repos::audit_repo::{AuditLog, NewAuditLog, SignInAttempt};
    use crate::db::repos::credential_repo::Credential;
    use crate::db::repos::user_repo::UpdateUser;
    use std::sync::Mutex;
    use uuid::Uuid;

    // -- Mock UserRepository --------------------------------------------------

    pub(crate) struct MockUserRepo {
        pub(crate) users: Mutex<Vec<User>>,
    }

    impl MockUserRepo {
        pub(crate) fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn with_user(user: User) -> Self {
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

        async fn ban(&self, _project_id: &ProjectId, _user_id: &UserId) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn unban(&self, _project_id: &ProjectId, _user_id: &UserId) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    // -- Mock CredentialRepository --------------------------------------------

    pub(crate) struct MockCredentialRepo {
        pub(crate) credentials: Mutex<Vec<Credential>>,
    }

    impl MockCredentialRepo {
        pub(crate) fn new() -> Self {
            Self {
                credentials: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn with_credential(cred: Credential) -> Self {
            Self {
                credentials: Mutex::new(vec![cred]),
            }
        }
    }

    #[async_trait]
    impl CredentialRepository for MockCredentialRepo {
        async fn create(&self, new: &NewCredential) -> Result<Credential, AppError> {
            let cred = Credential {
                id: crate::core::types::CredentialId::new(),
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
            credential_type: &str,
            provider: &str,
            identifier: &str,
        ) -> Result<Option<Credential>, AppError> {
            let creds = self.credentials.lock().unwrap();
            Ok(creds
                .iter()
                .find(|c| {
                    c.credential_type == credential_type
                        && c.provider.as_deref() == Some(provider)
                        && c.identifier.as_deref() == Some(identifier)
                })
                .cloned())
        }

        async fn update_secret(
            &self,
            _id: &crate::core::types::CredentialId,
            _new_secret_hash: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn delete(&self, _id: &crate::core::types::CredentialId) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    // -- Mock AuditRepository -------------------------------------------------

    pub(crate) struct MockAuditRepo {
        attempts: Mutex<Vec<SignInAttempt>>,
    }

    impl MockAuditRepo {
        pub(crate) fn new() -> Self {
            Self {
                attempts: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn attempts(&self) -> Vec<SignInAttempt> {
            self.attempts.lock().unwrap().clone()
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
            let attempt = SignInAttempt {
                id: Uuid::new_v4(),
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
            };
            self.attempts.lock().unwrap().push(attempt.clone());
            Ok(attempt)
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

    // ── Mock MfaEnrollmentRepository ─────────────────────────────────

    pub(crate) struct MockMfaRepo {
        pub(crate) enrollments: Mutex<Vec<crate::db::repos::mfa_enrollment_repo::MfaEnrollment>>,
    }

    impl MockMfaRepo {
        pub(crate) fn new() -> Self {
            Self {
                enrollments: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl MfaEnrollmentRepository for MockMfaRepo {
        async fn create(
            &self,
            user_id: uuid::Uuid,
            mfa_type: &str,
            secret_enc: Option<&str>,
            backup_codes_enc: Option<&str>,
        ) -> Result<crate::db::repos::mfa_enrollment_repo::MfaEnrollment, AppError> {
            let enrollment = crate::db::repos::mfa_enrollment_repo::MfaEnrollment {
                id: uuid::Uuid::new_v4(),
                user_id,
                mfa_type: mfa_type.to_string(),
                secret_enc: secret_enc.map(|s| s.to_string()),
                phone: None,
                backup_codes_enc: backup_codes_enc.map(|s| s.to_string()),
                verified: false,
                last_used_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.enrollments.lock().unwrap().push(enrollment.clone());
            Ok(enrollment)
        }

        async fn find_active_by_user(
            &self,
            user_id: uuid::Uuid,
            mfa_type: &str,
        ) -> Result<Option<crate::db::repos::mfa_enrollment_repo::MfaEnrollment>, AppError>
        {
            let enrollments = self.enrollments.lock().unwrap();
            Ok(enrollments
                .iter()
                .find(|e| e.user_id == user_id && e.mfa_type == mfa_type && e.verified)
                .cloned())
        }

        async fn list_active_by_user(
            &self,
            user_id: uuid::Uuid,
        ) -> Result<Vec<crate::db::repos::mfa_enrollment_repo::MfaEnrollment>, AppError> {
            let enrollments = self.enrollments.lock().unwrap();
            Ok(enrollments
                .iter()
                .filter(|e| e.user_id == user_id && e.verified)
                .cloned()
                .collect())
        }

        async fn mark_verified(&self, id: uuid::Uuid) -> Result<(), AppError> {
            let mut enrollments = self.enrollments.lock().unwrap();
            if let Some(e) = enrollments.iter_mut().find(|e| e.id == id) {
                e.verified = true;
            }
            Ok(())
        }

        async fn update_backup_codes(
            &self,
            id: uuid::Uuid,
            backup_codes_enc: &str,
        ) -> Result<(), AppError> {
            let mut enrollments = self.enrollments.lock().unwrap();
            if let Some(e) = enrollments.iter_mut().find(|e| e.id == id) {
                e.backup_codes_enc = Some(backup_codes_enc.to_string());
            }
            Ok(())
        }

        async fn update_last_used(&self, id: uuid::Uuid) -> Result<(), AppError> {
            let mut enrollments = self.enrollments.lock().unwrap();
            if let Some(e) = enrollments.iter_mut().find(|e| e.id == id) {
                e.last_used_at = Some(Utc::now());
            }
            Ok(())
        }

        async fn delete(&self, id: uuid::Uuid) -> Result<(), AppError> {
            let mut enrollments = self.enrollments.lock().unwrap();
            enrollments.retain(|e| e.id != id);
            Ok(())
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    pub(crate) fn test_signing_key() -> Arc<SigningKeyPair> {
        Arc::new(JwtService::generate_key_pair("test-kid").unwrap())
    }

    pub(crate) fn make_service(
        user_repo: Arc<dyn UserRepository>,
        cred_repo: Arc<dyn CredentialRepository>,
        audit_repo: Arc<dyn AuditRepository>,
    ) -> AuthService {
        let mfa_repo = Arc::new(MockMfaRepo::new());

        let redis = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(async {
                let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
                ConnectionManager::new(client).await.unwrap()
            }),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
                    ConnectionManager::new(client).await.unwrap()
                })
            }
        };

        let session_repo = Arc::new(crate::db::repos::RedisSessionRepository::new(redis.clone()));
        let session_service = Arc::new(SessionService::new(
            session_repo,
            Arc::new(crate::core::clock::SystemClock),
        ));

        AuthService::new(
            user_repo,
            cred_repo,
            mfa_repo,
            audit_repo,
            session_service,
            redis,
            test_signing_key(),
            "https://nucleus.test".to_string(),
            3600,
        )
    }

    pub(crate) fn make_test_user(project_id: &ProjectId) -> User {
        User {
            id: UserId::new(),
            project_id: *project_id,
            external_id: None,
            email: "test@example.com".to_string(),
            email_verified: true,
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

    pub(crate) fn make_password_credential(user_id: &UserId, password: &str) -> Credential {
        let hash = PasswordService::hash(password).unwrap();
        Credential {
            id: crate::core::types::CredentialId::new(),
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
}
