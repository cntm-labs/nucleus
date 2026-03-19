use async_trait::async_trait;
use nucleus_core::error::AppError;
use nucleus_core::types::{ProjectId, SessionId, UserId};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Serialize;
use anyhow::anyhow;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub project_id: ProjectId,
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
            .sadd(user_sessions_key(&session.user_id), session_id.0.to_string())
            .exec_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(Session {
            id: session_id,
            user_id: session.user_id,
            project_id: session.project_id,
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
