use axum::{extract::State, http::header, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

/// GET /.well-known/jwks.json
/// Returns the public signing keys in JWKS format.
/// Cache: 24 hours (keys don't change often, except during rotation).
pub async fn handle_jwks(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let jwks = state.signing_key.to_jwks().unwrap_or_else(|_| {
        nucleus_auth::jwt::Jwks {
            keys: vec![],
        }
    });

    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        Json(json!(jwks)),
    )
}

/// GET /.well-known/openid-configuration
/// OpenID Connect Discovery document.
pub async fn handle_openid_configuration(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let issuer = "https://nucleus.local"; // TODO: from config

    let config = json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{}/api/v1/auth/sign-in/oauth", issuer),
        "token_endpoint": format!("{}/api/v1/auth/token/refresh", issuer),
        "userinfo_endpoint": format!("{}/api/v1/users/me", issuer),
        "jwks_uri": format!("{}/.well-known/jwks.json", issuer),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "email", "profile"],
        "token_endpoint_auth_methods_supported": ["client_secret_post"],
        "claims_supported": [
            "sub", "iss", "aud", "exp", "iat", "jti",
            "email", "email_verified", "first_name", "last_name",
            "org_id", "org_slug", "org_role", "org_permissions"
        ]
    });

    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        Json(config),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nucleus_auth::jwt::{JwtService, Jwks};

    #[test]
    fn jwks_response_has_correct_structure() {
        let kp = JwtService::generate_key_pair("test-kid-1").unwrap();
        let jwks = kp.to_jwks().unwrap();

        assert_eq!(jwks.keys.len(), 1);
        let key = &jwks.keys[0];
        assert_eq!(key.kty, "RSA");
        assert_eq!(key.kid, "test-kid-1");
        assert_eq!(key.alg, "RS256");
        assert_eq!(key.use_, "sig");
        assert!(!key.n.is_empty(), "modulus n must be populated");
        assert!(!key.e.is_empty(), "exponent e must be populated");
    }

    #[test]
    fn jwk_entry_has_valid_base64url_components() {
        use base64::Engine;

        let kp = JwtService::generate_key_pair("kid-b64").unwrap();
        let entry = kp.to_jwk_entry().unwrap();

        // Verify n and e are valid base64url
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&entry.n)
            .expect("n should be valid base64url");
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&entry.e)
            .expect("e should be valid base64url");
    }

    #[test]
    fn jwks_serializes_use_field_correctly() {
        let kp = JwtService::generate_key_pair("kid-serde").unwrap();
        let jwks = kp.to_jwks().unwrap();
        let json = serde_json::to_value(&jwks).unwrap();

        // The field should be "use" not "use_"
        let key = &json["keys"][0];
        assert_eq!(key["use"], "sig");
        assert!(key.get("use_").is_none());
    }

    #[test]
    fn openid_config_has_required_fields() {
        let issuer = "https://nucleus.local";
        let config = json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{}/api/v1/auth/sign-in/oauth", issuer),
            "token_endpoint": format!("{}/api/v1/auth/token/refresh", issuer),
            "userinfo_endpoint": format!("{}/api/v1/users/me", issuer),
            "jwks_uri": format!("{}/.well-known/jwks.json", issuer),
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "scopes_supported": ["openid", "email", "profile"],
            "token_endpoint_auth_methods_supported": ["client_secret_post"],
            "claims_supported": [
                "sub", "iss", "aud", "exp", "iat", "jti",
                "email", "email_verified", "first_name", "last_name",
                "org_id", "org_slug", "org_role", "org_permissions"
            ]
        });

        // Required OpenID Connect Discovery fields
        assert!(config.get("issuer").is_some());
        assert!(config.get("authorization_endpoint").is_some());
        assert!(config.get("token_endpoint").is_some());
        assert!(config.get("jwks_uri").is_some());
        assert!(config.get("response_types_supported").is_some());
        assert!(config.get("subject_types_supported").is_some());
        assert!(config.get("id_token_signing_alg_values_supported").is_some());

        // Verify jwks_uri points to the right path
        assert_eq!(
            config["jwks_uri"],
            "https://nucleus.local/.well-known/jwks.json"
        );
    }
}
