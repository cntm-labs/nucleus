use redis::aio::ConnectionManager;

pub async fn create_redis_pool(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    ConnectionManager::new(client).await
}
