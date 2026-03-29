use std::sync::Arc;

use nucleus_core::clock::Clock;
use nucleus_core::crypto;
use nucleus_core::error::{AppError, AuthError};
use nucleus_core::types::*;
use nucleus_db::repos::session_repo::{NewSession, Session, SessionRepository};

pub struct SessionService {
    repo: Arc<dyn SessionRepository>,
    _clock: Arc<dyn Clock>,
}

impl SessionService {
    pub fn new(repo: Arc<dyn SessionRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            repo,
            _clock: clock,
        }
    }

    /// Create a new session, returns (session_token, session).
    ///
    /// The session_token is a 256-bit random token that the client stores.
    /// We store the hash of this token as the session ID in Redis.
    pub async fn create_session(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        device_info: DeviceInfo,
        ttl_secs: u64,
    ) -> Result<(String, Session), AppError> {
        let session_token = crypto::generate_token();
        let token_hash = crypto::generate_token_hash(&session_token);

        let new_session = NewSession {
            user_id: *user_id,
            project_id: *project_id,
            token_hash,
            device_type: device_info.device_type,
            device_name: device_info.device_name,
            browser: device_info.browser,
            ip: device_info.ip,
            ttl_secs,
        };

        let session = self.repo.create(&new_session).await?;
        Ok((session_token, session))
    }

    /// Validate a session exists and is active.
    pub async fn validate_session(&self, session_id: &SessionId) -> Result<Session, AppError> {
        self.repo
            .find_by_id(session_id)
            .await?
            .ok_or(AppError::Auth(AuthError::SessionExpired))
    }

    /// Validate a session exists and the token matches the stored hash.
    pub async fn validate_session_with_token(
        &self,
        session_id: &SessionId,
        token: &str,
    ) -> Result<Session, AppError> {
        let session = self.validate_session(session_id).await?;
        let token_hash = crypto::generate_token_hash(token);
        if !crypto::constant_time_eq(token_hash.as_bytes(), session.token_hash.as_bytes()) {
            return Err(AppError::Auth(AuthError::SessionInvalid));
        }
        Ok(session)
    }

    /// Revoke a single session and its associated JWT.
    /// SECURITY: Always revoke the JWT when revoking a session to prevent
    /// continued access with a stolen short-lived JWT.
    pub async fn revoke_session(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
        jti: Option<&str>,
        jwt_ttl_secs: u64,
    ) -> Result<(), AppError> {
        self.repo.delete(session_id, user_id).await?;
        if let Some(jti) = jti {
            self.repo.add_to_revocation_list(jti, jwt_ttl_secs).await?;
        }
        Ok(())
    }

    /// Revoke all sessions for a user.
    pub async fn revoke_all_sessions(&self, user_id: &UserId) -> Result<u64, AppError> {
        self.repo.delete_all_for_user(user_id).await
    }

    /// List all active sessions for a user.
    pub async fn list_user_sessions(&self, user_id: &UserId) -> Result<Vec<Session>, AppError> {
        self.repo.list_for_user(user_id).await
    }

    /// Add a JWT ID to the revocation list.
    pub async fn revoke_jwt(&self, jti: &str, ttl_secs: u64) -> Result<(), AppError> {
        self.repo.add_to_revocation_list(jti, ttl_secs).await
    }

    /// Check if a JWT has been revoked.
    pub async fn is_jwt_revoked(&self, jti: &str) -> Result<bool, AppError> {
        self.repo.is_revoked(jti).await
    }

    /// Update last active timestamp for a session.
    pub async fn touch_session(&self, session_id: &SessionId) -> Result<(), AppError> {
        self.repo.update_last_active(session_id).await
    }
}

#[derive(Debug, Default)]
pub struct DeviceInfo {
    pub device_type: Option<String>,
    pub device_name: Option<String>,
    pub browser: Option<String>,
    pub ip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use nucleus_core::clock::Clock;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // -----------------------------------------------------------------------
    // Test Clock
    // -----------------------------------------------------------------------

    struct TestClock {
        now: DateTime<Utc>,
    }

    impl Clock for TestClock {
        fn now(&self) -> DateTime<Utc> {
            self.now
        }
    }

    // -----------------------------------------------------------------------
    // Mock SessionRepository
    // -----------------------------------------------------------------------

    struct MockSessionRepo {
        sessions: Mutex<HashMap<SessionId, Session>>,
        user_sessions: Mutex<HashMap<UserId, Vec<SessionId>>>,
        revoked_jwts: Mutex<HashMap<String, bool>>,
    }

    impl MockSessionRepo {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
                user_sessions: Mutex::new(HashMap::new()),
                revoked_jwts: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepo {
        async fn create(&self, session: &NewSession) -> Result<Session, AppError> {
            let session_id = SessionId::new();
            let now = Utc::now().to_rfc3339();

            let s = Session {
                id: session_id,
                user_id: session.user_id,
                project_id: session.project_id,
                token_hash: session.token_hash.clone(),
                device_type: session.device_type.clone(),
                device_name: session.device_name.clone(),
                browser: session.browser.clone(),
                ip: session.ip.clone(),
                country_code: None,
                created_at: now.clone(),
                last_active_at: now,
            };

            self.sessions.lock().unwrap().insert(session_id, s.clone());
            self.user_sessions
                .lock()
                .unwrap()
                .entry(session.user_id)
                .or_default()
                .push(session_id);

            Ok(s)
        }

        async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<Session>, AppError> {
            Ok(self.sessions.lock().unwrap().get(session_id).cloned())
        }

        async fn update_last_active(&self, session_id: &SessionId) -> Result<(), AppError> {
            if let Some(s) = self.sessions.lock().unwrap().get_mut(session_id) {
                s.last_active_at = Utc::now().to_rfc3339();
            }
            Ok(())
        }

        async fn delete(&self, session_id: &SessionId, user_id: &UserId) -> Result<(), AppError> {
            self.sessions.lock().unwrap().remove(session_id);
            if let Some(ids) = self.user_sessions.lock().unwrap().get_mut(user_id) {
                ids.retain(|id| id != session_id);
            }
            Ok(())
        }

        async fn delete_all_for_user(&self, user_id: &UserId) -> Result<u64, AppError> {
            let mut sessions = self.sessions.lock().unwrap();
            let mut user_sessions = self.user_sessions.lock().unwrap();

            let ids = user_sessions.remove(user_id).unwrap_or_default();
            let count = ids.len() as u64;
            for id in &ids {
                sessions.remove(id);
            }
            Ok(count)
        }

        async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<Session>, AppError> {
            let sessions = self.sessions.lock().unwrap();
            let user_sessions = self.user_sessions.lock().unwrap();

            let ids = match user_sessions.get(user_id) {
                Some(ids) => ids.clone(),
                None => return Ok(vec![]),
            };

            let mut result = Vec::new();
            for id in &ids {
                if let Some(s) = sessions.get(id) {
                    result.push(s.clone());
                }
            }
            Ok(result)
        }

        async fn add_to_revocation_list(&self, jti: &str, _ttl_secs: u64) -> Result<(), AppError> {
            self.revoked_jwts
                .lock()
                .unwrap()
                .insert(jti.to_string(), true);
            Ok(())
        }

        async fn is_revoked(&self, jti: &str) -> Result<bool, AppError> {
            Ok(self.revoked_jwts.lock().unwrap().contains_key(jti))
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn test_service() -> SessionService {
        let repo = Arc::new(MockSessionRepo::new());
        let clock = Arc::new(TestClock { now: Utc::now() });
        SessionService::new(repo, clock)
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn create_session_returns_token_and_session() {
        let svc = test_service();
        let (token, session) = svc
            .create_session(
                &UserId::new(),
                &ProjectId::new(),
                DeviceInfo::default(),
                3600,
            )
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert!(!session.id.to_string().is_empty());
    }

    #[tokio::test]
    async fn validate_existing_session() {
        let svc = test_service();
        let (_, session) = svc
            .create_session(
                &UserId::new(),
                &ProjectId::new(),
                DeviceInfo::default(),
                3600,
            )
            .await
            .unwrap();

        let found = svc.validate_session(&session.id).await.unwrap();
        assert_eq!(found.id, session.id);
    }

    #[tokio::test]
    async fn validate_nonexistent_session_fails() {
        let svc = test_service();
        let result = svc.validate_session(&SessionId::new()).await;
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::SessionExpired))
        ));
    }

    #[tokio::test]
    async fn revoke_session_then_validate_fails() {
        let svc = test_service();
        let user_id = UserId::new();
        let (_, session) = svc
            .create_session(&user_id, &ProjectId::new(), DeviceInfo::default(), 3600)
            .await
            .unwrap();

        svc.revoke_session(&session.id, &user_id, Some("jti_test"), 300)
            .await
            .unwrap();

        let result = svc.validate_session(&session.id).await;
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::SessionExpired))
        ));
    }

    #[tokio::test]
    async fn revoke_all_sessions() {
        let svc = test_service();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        svc.create_session(&user_id, &project_id, DeviceInfo::default(), 3600)
            .await
            .unwrap();
        svc.create_session(&user_id, &project_id, DeviceInfo::default(), 3600)
            .await
            .unwrap();

        let count = svc.revoke_all_sessions(&user_id).await.unwrap();
        assert_eq!(count, 2);

        let sessions = svc.list_user_sessions(&user_id).await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn validate_session_with_correct_token() {
        let svc = test_service();
        let (token, session) = svc
            .create_session(
                &UserId::new(),
                &ProjectId::new(),
                DeviceInfo::default(),
                3600,
            )
            .await
            .unwrap();

        let found = svc
            .validate_session_with_token(&session.id, &token)
            .await
            .unwrap();
        assert_eq!(found.id, session.id);
    }

    #[tokio::test]
    async fn validate_session_with_wrong_token_fails() {
        let svc = test_service();
        let (_token, session) = svc
            .create_session(
                &UserId::new(),
                &ProjectId::new(),
                DeviceInfo::default(),
                3600,
            )
            .await
            .unwrap();

        let result = svc
            .validate_session_with_token(&session.id, "wrong_token")
            .await;
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::SessionInvalid))
        ));
    }

    #[tokio::test]
    async fn jwt_revocation_list() {
        let svc = test_service();
        assert!(!svc.is_jwt_revoked("jti_123").await.unwrap());
        svc.revoke_jwt("jti_123", 300).await.unwrap();
        assert!(svc.is_jwt_revoked("jti_123").await.unwrap());
    }

    #[tokio::test]
    async fn list_user_sessions() {
        let svc = test_service();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        svc.create_session(&user_id, &project_id, DeviceInfo::default(), 3600)
            .await
            .unwrap();
        svc.create_session(&user_id, &project_id, DeviceInfo::default(), 3600)
            .await
            .unwrap();

        let sessions = svc.list_user_sessions(&user_id).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
