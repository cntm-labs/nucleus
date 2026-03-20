//! Actix-web integration for Nucleus authentication.
//!
//! Enable the `actix` feature to use this module.
//!
//! # Example
//!
//! ```rust,no_run
//! use actix_web::{web, App, HttpServer, HttpResponse};
//! use nucleus_rs::{NucleusClient, NucleusConfig};
//! use nucleus_rs::actix::NucleusClaims;
//!
//! async fn me(claims: NucleusClaims) -> HttpResponse {
//!     HttpResponse::Ok().json(serde_json::json!({ "user_id": claims.user_id() }))
//! }
//!
//! let client = NucleusClient::new(NucleusConfig {
//!     secret_key: "sk_live_...".into(),
//!     base_url: None,
//!     jwks_cache_ttl_secs: None,
//! });
//!
//! HttpServer::new(move || {
//!     App::new()
//!         .app_data(web::Data::new(client.clone()))
//!         .route("/me", web::get().to(me))
//! });
//! ```

use crate::claims;
use crate::client::NucleusClient;
use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{web, Error, FromRequest, HttpRequest};
use futures_util::future::BoxFuture;

/// An Actix-web extractor that yields verified [`claims::NucleusClaims`].
///
/// Requires that a [`NucleusClient`] is registered as `web::Data<NucleusClient>`
/// in the Actix app data. The extractor reads the `Authorization: Bearer <token>`
/// header, verifies it, and yields the parsed claims.
#[derive(Debug, Clone)]
pub struct NucleusClaims(pub claims::NucleusClaims);

impl std::ops::Deref for NucleusClaims {
    type Target = claims::NucleusClaims;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for NucleusClaims {
    type Error = Error;
    type Future = BoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let client = req
            .app_data::<web::Data<NucleusClient>>()
            .cloned();

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_owned());

        Box::pin(async move {
            let client = client
                .ok_or_else(|| ErrorUnauthorized("NucleusClient not configured"))?;

            let token = auth_header
                .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"))?;

            let claims = client
                .verify_token(&token)
                .await
                .map_err(|e| ErrorUnauthorized(format!("Invalid token: {e}")))?;

            Ok(NucleusClaims(claims))
        })
    }
}
