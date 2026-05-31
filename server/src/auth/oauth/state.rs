use crate::auth::handlers::oauth::{OAuthStateData, OAuthStateStore};
use crate::core::error::AppError;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

pub struct RedisOAuthStateStore {
    redis: ConnectionManager,
}

impl RedisOAuthStateStore {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    fn key(state: &str) -> String {
        format!("oauth_state:{}", state)
    }
}

#[async_trait]
impl OAuthStateStore for RedisOAuthStateStore {
    async fn store_state(
        &self,
        state: &str,
        data: &OAuthStateData,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = Self::key(state);
        let json = serde_json::to_string(data)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?;

        let mut conn = self.redis.clone();
        conn.set_ex::<_, _, ()>(&key, &json, ttl_secs)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn consume_state(&self, state: &str) -> Result<Option<OAuthStateData>, AppError> {
        let key = Self::key(state);
        let mut conn = self.redis.clone();

        let json: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if let Some(json) = json {
            // Delete immediately (single use)
            let _: () = conn
                .del(&key)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

            let data: OAuthStateData = serde_json::from_str(&json)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("deserialize error: {}", e)))?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}
