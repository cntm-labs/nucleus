//! Account auth — separate stakeholder from "user" auth.
//! An "account" is a Nucleus customer who signs up to use the platform
//! to authenticate THEIR app's users. Account auth gates the dashboard.

// pub mod handlers;  // uncomment in Task 8
pub mod repo;
pub mod service;

pub use repo::{Account, AccountRepository, NewAccount, PgAccountRepository, UpdateAccount};
pub use service::AccountService;
