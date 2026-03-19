use anyhow::{Context, Result};

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub master_encryption_key: [u8; 32],
    pub host: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // optional .env file

        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let master_key_hex = std::env::var("MASTER_ENCRYPTION_KEY")
            .context("MASTER_ENCRYPTION_KEY must be set")?;
        let master_encryption_key = hex::decode(&master_key_hex)
            .context("MASTER_ENCRYPTION_KEY must be valid hex")?
            .try_into()
            .map_err(|_| {
                anyhow::anyhow!("MASTER_ENCRYPTION_KEY must be 32 bytes (64 hex chars)")
            })?;
        let host =
            std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("PORT must be a valid number")?;
        let rust_log = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "nucleus=debug,tower_http=debug".to_string());

        Ok(Self {
            database_url,
            redis_url,
            master_encryption_key,
            host,
            port,
            rust_log,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
