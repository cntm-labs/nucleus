use anyhow::{Context, Result};

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub master_encryption_key: [u8; 32],
    pub host: String,
    pub port: u16,
    pub rust_log: String,
    pub allowed_origins: Vec<String>,
    pub issuer_url: String,
    pub jwt_lifetime_secs: i64,
    pub rp_name: String,
    pub rp_id: String,
    pub rate_limit_auth_max: u32,
    pub rate_limit_auth_window_secs: u64,
    pub rate_limit_api_max: u32,
    pub rate_limit_api_window_secs: u64,
    pub trusted_proxies: Vec<String>,
    pub sendgrid_api_key: Option<String>,
    pub from_email: String,
    pub from_name: String,
    pub twilio_account_sid: Option<String>,
    pub twilio_auth_token: Option<String>,
    pub twilio_from_number: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // optional .env file

        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let master_key_hex =
            std::env::var("MASTER_ENCRYPTION_KEY").context("MASTER_ENCRYPTION_KEY must be set")?;
        let master_encryption_key: [u8; 32] = hex::decode(&master_key_hex)
            .context("MASTER_ENCRYPTION_KEY must be valid hex")?
            .try_into()
            .map_err(|_| {
                anyhow::anyhow!("MASTER_ENCRYPTION_KEY must be 32 bytes (64 hex chars)")
            })?;
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("PORT must be a valid number")?;
        let rust_log = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "nucleus=debug,tower_http=debug".to_string());
        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let issuer_url =
            std::env::var("ISSUER_URL").unwrap_or_else(|_| "https://nucleus.local".to_string());
        let jwt_lifetime_secs = std::env::var("JWT_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        let rp_name = std::env::var("RP_NAME").unwrap_or_else(|_| "Nucleus".to_string());
        let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rate_limit_auth_max = std::env::var("RATE_LIMIT_AUTH_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let rate_limit_auth_window_secs = std::env::var("RATE_LIMIT_AUTH_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let rate_limit_api_max = std::env::var("RATE_LIMIT_API_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);
        let rate_limit_api_window_secs = std::env::var("RATE_LIMIT_API_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let sendgrid_api_key = std::env::var("SENDGRID_API_KEY").ok();
        let from_email =
            std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@nucleus.dev".to_string());
        let from_name = std::env::var("FROM_NAME").unwrap_or_else(|_| "Nucleus".to_string());
        let twilio_account_sid = std::env::var("TWILIO_ACCOUNT_SID").ok();
        let twilio_auth_token = std::env::var("TWILIO_AUTH_TOKEN").ok();
        let twilio_from_number = std::env::var("TWILIO_FROM_NUMBER").ok();

        if master_encryption_key.iter().all(|&b| b == 0) {
            anyhow::bail!("MASTER_ENCRYPTION_KEY must not be all zeros");
        }

        Ok(Self {
            database_url,
            redis_url,
            master_encryption_key,
            host,
            port,
            rust_log,
            allowed_origins,
            issuer_url,
            jwt_lifetime_secs,
            rp_name,
            rp_id,
            rate_limit_auth_max,
            rate_limit_auth_window_secs,
            rate_limit_api_max,
            rate_limit_api_window_secs,
            trusted_proxies,
            sendgrid_api_key,
            from_email,
            from_name,
            twilio_account_sid,
            twilio_auth_token,
            twilio_from_number,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
