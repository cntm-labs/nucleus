use crate::core::error::AppError;
use crate::core::types::{ProjectId, SessionId, UserId};
use anyhow::anyhow;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub token_hash: String,
    pub device_type: Option<String>,
    pub device_name: Option<String>,
    pub browser: Option<String>,
    pub ip: Option<String>,
    pub country_code: Option<String>,
    pub created_at: String,
    pub last_active_at: String,
}

pub struct NewSession {
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub token_hash: String,
    pub device_type: Option<String>,
    pub device_name: Option<String>,
    pub browser: Option<String>,
    pub ip: Option<String>,
    pub ttl_secs: u64,
}

fn session_key(session_id: &SessionId) -> String {
    format!("session:{}", session_id.0)
}

fn user_sessions_key(user_id: &UserId) -> String {
    format!("user_sessions:{}", user_id.0)
}

fn revoked_jwt_key(jti: &str) -> String {
    format!("revoked_jwt:{}", jti)
}

fn opt_field(values: &[(String, String)], key: &str) -> Option<String> {
    values
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
}

fn required_field(values: &[(String, String)], key: &str) -> Result<String, AppError> {
    values
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .ok_or_else(|| AppError::Internal(anyhow!("missing session field: {}", key)))
}

fn session_from_hash(values: Vec<(String, String)>) -> Result<Session, AppError> {
    Ok(Session {
        id: SessionId::from_uuid(
            required_field(&values, "id")?
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?,
        ),
        user_id: UserId::from_uuid(
            required_field(&values, "user_id")?
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?,
        ),
        project_id: ProjectId::from_uuid(
            required_field(&values, "project_id")?
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?,
        ),
        token_hash: required_field(&values, "token_hash")?,
        device_type: opt_field(&values, "device_type"),
        device_name: opt_field(&values, "device_name"),
        browser: opt_field(&values, "browser"),
        ip: opt_field(&values, "ip"),
        country_code: opt_field(&values, "country_code"),
        created_at: required_field(&values, "created_at")?,
        last_active_at: required_field(&values, "last_active_at")?,
    })
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &NewSession) -> Result<Session, AppError>;
    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<Session>, AppError>;
    async fn update_last_active(&self, session_id: &SessionId) -> Result<(), AppError>;
    async fn delete(&self, session_id: &SessionId, user_id: &UserId) -> Result<(), AppError>;
    async fn delete_all_for_user(&self, user_id: &UserId) -> Result<u64, AppError>;
    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<Session>, AppError>;
    async fn add_to_revocation_list(&self, jti: &str, ttl_secs: u64) -> Result<(), AppError>;
    async fn is_revoked(&self, jti: &str) -> Result<bool, AppError>;
}

pub struct RedisSessionRepository {
    conn: ConnectionManager,
}

impl RedisSessionRepository {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SessionRepository for RedisSessionRepository {
    async fn create(&self, session: &NewSession) -> Result<Session, AppError> {
        let mut conn = self.conn.clone();
        let session_id = SessionId::new();
        let now = chrono::Utc::now().to_rfc3339();
        let key = session_key(&session_id);

        let mut fields: Vec<(String, String)> = vec![
            ("id".to_string(), session_id.0.to_string()),
            ("user_id".to_string(), session.user_id.0.to_string()),
            ("project_id".to_string(), session.project_id.0.to_string()),
            ("token_hash".to_string(), session.token_hash.clone()),
            ("created_at".to_string(), now.clone()),
            ("last_active_at".to_string(), now.clone()),
        ];

        if let Some(ref v) = session.device_type {
            fields.push(("device_type".to_string(), v.clone()));
        }
        if let Some(ref v) = session.device_name {
            fields.push(("device_name".to_string(), v.clone()));
        }
        if let Some(ref v) = session.browser {
            fields.push(("browser".to_string(), v.clone()));
        }
        if let Some(ref v) = session.ip {
            fields.push(("ip".to_string(), v.clone()));
        }

        redis::pipe()
            .hset_multiple(&key, &fields)
            .expire(&key, session.ttl_secs as i64)
            .sadd(
                user_sessions_key(&session.user_id),
                session_id.0.to_string(),
            )
            .exec_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(Session {
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
        })
    }

    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<Session>, AppError> {
        let mut conn = self.conn.clone();
        let key = session_key(session_id);

        let values: Vec<(String, String)> = conn
            .hgetall(&key)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if values.is_empty() {
            return Ok(None);
        }

        Ok(Some(session_from_hash(values)?))
    }

    async fn update_last_active(&self, session_id: &SessionId) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let key = session_key(session_id);
        let now = chrono::Utc::now().to_rfc3339();

        conn.hset::<_, _, _, ()>(&key, "last_active_at", &now)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn delete(&self, session_id: &SessionId, user_id: &UserId) -> Result<(), AppError> {
        let mut conn = self.conn.clone();

        redis::pipe()
            .del(session_key(session_id))
            .srem(user_sessions_key(user_id), session_id.0.to_string())
            .exec_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn delete_all_for_user(&self, user_id: &UserId) -> Result<u64, AppError> {
        let mut conn = self.conn.clone();
        let set_key = user_sessions_key(user_id);

        let session_ids: Vec<String> = conn
            .smembers(&set_key)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let count = session_ids.len() as u64;

        if !session_ids.is_empty() {
            let mut pipe = redis::pipe();
            for sid in &session_ids {
                pipe.del(format!("session:{}", sid));
            }
            pipe.del(&set_key);
            pipe.exec_async(&mut conn)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
        }

        Ok(count)
    }

    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<Session>, AppError> {
        let mut conn = self.conn.clone();
        let set_key = user_sessions_key(user_id);

        let session_ids: Vec<String> = conn
            .smembers(&set_key)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let mut sessions = Vec::new();
        for sid in &session_ids {
            let key = format!("session:{}", sid);
            let values: Vec<(String, String)> = conn
                .hgetall(&key)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

            if !values.is_empty() {
                sessions.push(session_from_hash(values)?);
            }
        }

        Ok(sessions)
    }

    async fn add_to_revocation_list(&self, jti: &str, ttl_secs: u64) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let key = revoked_jwt_key(jti);

        conn.set_ex::<_, _, ()>(&key, "1", ttl_secs)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn is_revoked(&self, jti: &str) -> Result<bool, AppError> {
        let mut conn = self.conn.clone();
        let key = revoked_jwt_key(jti);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(exists)
    }
}

/// In-memory mock for testing SessionRepository trait contracts.
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    struct MockSessionRepo {
        sessions: Mutex<HashMap<SessionId, Session>>,
        user_sessions: Mutex<HashMap<UserId, Vec<SessionId>>>,
        revoked_jwts: Mutex<HashSet<String>>,
    }

    impl MockSessionRepo {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
                user_sessions: Mutex::new(HashMap::new()),
                revoked_jwts: Mutex::new(HashSet::new()),
            }
        }
    }

    #[async_trait]
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
            Ok(ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect())
        }

        async fn add_to_revocation_list(&self, jti: &str, _ttl_secs: u64) -> Result<(), AppError> {
            self.revoked_jwts.lock().unwrap().insert(jti.to_string());
            Ok(())
        }

        async fn is_revoked(&self, jti: &str) -> Result<bool, AppError> {
            Ok(self.revoked_jwts.lock().unwrap().contains(jti))
        }
    }

    fn make_new_session(user_id: UserId, project_id: ProjectId) -> NewSession {
        NewSession {
            user_id,
            project_id,
            token_hash: "test_hash_abc".to_string(),
            device_type: Some("desktop".to_string()),
            device_name: None,
            browser: Some("Chrome".to_string()),
            ip: Some("127.0.0.1".to_string()),
            ttl_secs: 3600,
        }
    }

    #[tokio::test]
    async fn create_session_returns_session_with_id() {
        let repo = MockSessionRepo::new();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        let session = repo
            .create(&make_new_session(user_id, project_id))
            .await
            .unwrap();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.project_id, project_id);
        assert!(!session.id.0.is_nil());
        assert_eq!(session.browser, Some("Chrome".to_string()));
    }

    #[tokio::test]
    async fn find_by_id_returns_created_session() {
        let repo = MockSessionRepo::new();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        let session = repo
            .create(&make_new_session(user_id, project_id))
            .await
            .unwrap();

        let found = repo.find_by_id(&session.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn find_by_id_returns_none_for_nonexistent() {
        let repo = MockSessionRepo::new();
        let result = repo.find_by_id(&SessionId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_removes_session() {
        let repo = MockSessionRepo::new();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        let session = repo
            .create(&make_new_session(user_id, project_id))
            .await
            .unwrap();

        repo.delete(&session.id, &user_id).await.unwrap();

        let found = repo.find_by_id(&session.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn delete_all_for_user_removes_all_sessions() {
        let repo = MockSessionRepo::new();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        for _ in 0..3 {
            repo.create(&make_new_session(user_id, project_id))
                .await
                .unwrap();
        }

        let count = repo.delete_all_for_user(&user_id).await.unwrap();
        assert_eq!(count, 3);

        let remaining = repo.list_for_user(&user_id).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn list_for_user_returns_only_that_users_sessions() {
        let repo = MockSessionRepo::new();
        let user_a = UserId::new();
        let user_b = UserId::new();
        let project_id = ProjectId::new();

        repo.create(&make_new_session(user_a, project_id))
            .await
            .unwrap();
        repo.create(&make_new_session(user_a, project_id))
            .await
            .unwrap();
        repo.create(&make_new_session(user_b, project_id))
            .await
            .unwrap();

        let a_sessions = repo.list_for_user(&user_a).await.unwrap();
        assert_eq!(a_sessions.len(), 2);

        let b_sessions = repo.list_for_user(&user_b).await.unwrap();
        assert_eq!(b_sessions.len(), 1);
    }

    #[tokio::test]
    async fn jwt_revocation_list_works() {
        let repo = MockSessionRepo::new();

        assert!(!repo.is_revoked("jti_abc").await.unwrap());

        repo.add_to_revocation_list("jti_abc", 300).await.unwrap();

        assert!(repo.is_revoked("jti_abc").await.unwrap());
        assert!(!repo.is_revoked("jti_xyz").await.unwrap());
    }

    #[tokio::test]
    async fn update_last_active_changes_timestamp() {
        let repo = MockSessionRepo::new();
        let user_id = UserId::new();
        let project_id = ProjectId::new();

        let session = repo
            .create(&make_new_session(user_id, project_id))
            .await
            .unwrap();
        let original = session.last_active_at.clone();

        // Small delay to ensure timestamps differ
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        repo.update_last_active(&session.id).await.unwrap();

        let updated = repo.find_by_id(&session.id).await.unwrap().unwrap();
        assert_ne!(updated.last_active_at, original);
    }
}
