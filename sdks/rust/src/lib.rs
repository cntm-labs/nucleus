//! # nucleus-rs
//!
//! Rust backend SDK for [Nucleus](https://nucleus.dev) authentication.
//!
//! Provides JWT verification with JWKS caching, an admin API client for managing
//! users and organisations, and optional middleware for Axum and Actix-web.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use nucleus_rs::{NucleusClient, NucleusConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = NucleusClient::new(NucleusConfig {
//!         secret_key: "sk_live_...".into(),
//!         base_url: None,
//!         jwks_cache_ttl_secs: None,
//!     });
//!
//!     let claims = client.verify_token("eyJ...").await.unwrap();
//!     println!("user id: {}", claims.user_id());
//! }
//! ```
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `axum`  | Adds [`axum::NucleusLayer`] and [`axum::NucleusClaims`] extractor |
//! | `actix` | Adds [`actix::NucleusClaims`] extractor for Actix-web |

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init() {
    if VERSION.contains("dev") {
        eprintln!("[Nucleus] WARNING: You are using a dev preview ({VERSION}). Do not use in production.");
    }
}

pub mod admin;
pub mod claims;
pub mod client;
pub mod verify;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "actix")]
pub mod actix;

// Re-exports for convenience.
pub use claims::NucleusClaims;
pub use client::{NucleusClient, NucleusConfig};
pub use verify::NucleusError;
