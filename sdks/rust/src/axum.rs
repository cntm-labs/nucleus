//! Axum integration for Nucleus authentication.
//!
//! Enable the `axum` feature to use this module.
//!
//! # Example
//!
//! ```rust,no_run
//! use axum::{Router, routing::get, Json};
//! use nucleus_rs::{NucleusClient, NucleusConfig};
//! use nucleus_rs::axum::{NucleusLayer, NucleusClaims};
//!
//! async fn me(claims: NucleusClaims) -> Json<serde_json::Value> {
//!     Json(serde_json::json!({ "user_id": claims.user_id() }))
//! }
//!
//! let client = NucleusClient::new(NucleusConfig {
//!     secret_key: "sk_live_...".into(),
//!     base_url: None,
//!     jwks_cache_ttl_secs: None,
//! });
//!
//! let app = Router::new()
//!     .route("/me", get(me))
//!     .layer(NucleusLayer::new(client));
//! ```

use crate::claims;
use crate::verify::JwksVerifier;
use ::axum::extract::FromRequestParts;
use ::axum::http::{self, request::Parts, StatusCode};
use ::axum::response::{IntoResponse, Response};
use futures_util::future::BoxFuture;
use pin_project_lite::pin_project;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

// ── Extractor newtype ────────────────────────────────────────────────────────

/// An Axum extractor that yields verified [`claims::NucleusClaims`].
///
/// Extracts the claims that were attached to the request by [`NucleusLayer`].
#[derive(Debug, Clone)]
pub struct NucleusClaims(pub claims::NucleusClaims);

impl std::ops::Deref for NucleusClaims {
    type Target = claims::NucleusClaims;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for NucleusClaims {
    type Rejection = Response;

    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut Parts,
        _state: &'b S,
    ) -> BoxFuture<'c, Result<Self, Self::Rejection>>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::pin(async move {
            parts
                .extensions
                .get::<claims::NucleusClaims>()
                .cloned()
                .map(NucleusClaims)
                .ok_or_else(|| {
                    (StatusCode::UNAUTHORIZED, "Missing or invalid token").into_response()
                })
        })
    }
}

// ── Layer ────────────────────────────────────────────────────────────────────

/// A [`tower::Layer`] that verifies Nucleus JWTs on every request.
///
/// Extracts the `Authorization: Bearer <token>` header, verifies it against
/// the Nucleus JWKS endpoint, and inserts the parsed [`claims::NucleusClaims`]
/// into the request extensions. Returns 401 if the token is missing or invalid.
#[derive(Clone)]
pub struct NucleusLayer {
    verifier: Arc<JwksVerifier>,
}

impl NucleusLayer {
    pub fn new(client: crate::client::NucleusClient) -> Self {
        Self {
            verifier: client.verifier,
        }
    }
}

impl<S> Layer<S> for NucleusLayer {
    type Service = NucleusMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NucleusMiddleware {
            inner,
            verifier: Arc::clone(&self.verifier),
        }
    }
}

// ── Service ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NucleusMiddleware<S> {
    inner: S,
    verifier: Arc<JwksVerifier>,
}

impl<S, B> Service<http::Request<B>> for NucleusMiddleware<S>
where
    S: Service<http::Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<B>) -> Self::Future {
        let verifier = Arc::clone(&self.verifier);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let token = req
                .headers()
                .get(http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "));

            let token = match token {
                Some(t) => t.to_owned(),
                None => {
                    return Ok(
                        (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response()
                    );
                }
            };

            match verifier.verify(&token).await {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    inner.call(req).await
                }
                Err(_) => {
                    Ok((StatusCode::UNAUTHORIZED, "Invalid token").into_response())
                }
            }
        })
    }
}
