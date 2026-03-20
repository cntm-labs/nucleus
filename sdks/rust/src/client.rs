use crate::admin::{OrgsApi, UsersApi};
use crate::claims::NucleusClaims;
use crate::verify::{JwksVerifier, NucleusError};
use std::sync::Arc;

const DEFAULT_BASE_URL: &str = "https://api.nucleus.dev";

/// Configuration for [`NucleusClient`].
#[derive(Debug, Clone)]
pub struct NucleusConfig {
    /// Admin / secret API key for your Nucleus project.
    pub secret_key: String,

    /// Override the default Nucleus API base URL.
    /// Defaults to `https://api.nucleus.dev`.
    pub base_url: Option<String>,

    /// How long (in seconds) the JWKS key set should be cached.
    /// Defaults to 3600 (1 hour).
    pub jwks_cache_ttl_secs: Option<u64>,
}

/// Main entry point for interacting with Nucleus from a Rust backend.
///
/// ```rust,no_run
/// use nucleus_rs::{NucleusClient, NucleusConfig};
///
/// let client = NucleusClient::new(NucleusConfig {
///     secret_key: "sk_live_...".into(),
///     base_url: None,
///     jwks_cache_ttl_secs: None,
/// });
/// ```
#[derive(Clone)]
pub struct NucleusClient {
    pub(crate) config: NucleusConfig,
    pub(crate) verifier: Arc<JwksVerifier>,

    /// Access the Admin Users API.
    pub users: UsersApi,

    /// Access the Admin Orgs API.
    pub orgs: OrgsApi,
}

impl NucleusClient {
    /// Create a new [`NucleusClient`] with the given configuration.
    pub fn new(config: NucleusConfig) -> Self {
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let ttl_secs = config.jwks_cache_ttl_secs.unwrap_or(3600);
        let verifier = Arc::new(JwksVerifier::new(&base_url, ttl_secs));

        let http = Arc::new(crate::admin::HttpClient::new(
            base_url,
            config.secret_key.clone(),
        ));

        Self {
            config,
            verifier,
            users: UsersApi::new(Arc::clone(&http)),
            orgs: OrgsApi::new(Arc::clone(&http)),
        }
    }

    /// Verify a JWT access token against the Nucleus JWKS endpoint and return
    /// the parsed claims.
    pub async fn verify_token(&self, token: &str) -> Result<NucleusClaims, NucleusError> {
        self.verifier.verify(token).await
    }

    /// Return the configured base URL.
    pub fn base_url(&self) -> &str {
        self.config
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
    }
}
