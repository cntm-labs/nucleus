// Internal modules (previously separate crates)
pub mod api;
pub mod auth;
pub mod core;
pub mod db;
pub mod identity;
pub mod migrate;
pub mod org;
pub mod session;
pub mod webhook;

// Server modules
pub mod config;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod services;
pub mod state;
