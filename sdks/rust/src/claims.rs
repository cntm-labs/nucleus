use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT claims issued by Nucleus.
///
/// These are extracted from a verified access token and contain user identity
/// information plus optional organisation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusClaims {
    /// Subject — the Nucleus user ID.
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Audience — the Nucleus project ID.
    pub aud: String,
    /// Expiration time (Unix timestamp).
    pub exp: u64,
    /// Issued-at time (Unix timestamp).
    pub iat: u64,
    /// Unique token identifier.
    #[serde(default)]
    pub jti: Option<String>,

    // ── User fields ──────────────────────────────────────────────
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    // ── Organisation fields ──────────────────────────────────────
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_slug: Option<String>,
    #[serde(default)]
    pub org_role: Option<String>,
    #[serde(default)]
    pub org_permissions: Option<Vec<String>>,
}

impl NucleusClaims {
    /// Returns the Nucleus user ID (the `sub` claim).
    pub fn user_id(&self) -> &str {
        &self.sub
    }
}
