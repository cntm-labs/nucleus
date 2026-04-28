//! Shared test constants and helpers.
//!
//! New tests should prefer these over inlining literals to keep
//! Sonar's duplicate-string-literal rule satisfied. Existing
//! tests that pre-date this module are not retroactively migrated.

use uuid::Uuid;

use crate::core::crypto;
use crate::core::types::{ProjectId, UserId};
use crate::db::repos::user_repo::NewUser;
use crate::session::DeviceInfo;

const TEST_PROJECT_A_UUID: &str = "00000000-0000-4000-8000-000000000001";
const TEST_PROJECT_B_UUID: &str = "00000000-0000-4000-8000-000000000002";

pub const TEST_EMAIL: &str = "fixture@example.test";
pub const TEST_TTL_SECS: u64 = 3600;

/// Deterministic project A id, useful for cross-tenant isolation tests.
pub fn test_project_a() -> ProjectId {
    ProjectId::from_uuid(Uuid::parse_str(TEST_PROJECT_A_UUID).expect("valid uuid literal"))
}

/// Deterministic project B id, paired with [`test_project_a`].
pub fn test_project_b() -> ProjectId {
    ProjectId::from_uuid(Uuid::parse_str(TEST_PROJECT_B_UUID).expect("valid uuid literal"))
}

/// Generate a fresh random session token. Wraps [`crypto::generate_token`]
/// for clarity at the call site.
pub fn fake_token() -> String {
    crypto::generate_token()
}

/// Default device info — all fields `None`. Suitable for tests that do
/// not exercise device tracking specifically.
pub fn fake_device_info() -> DeviceInfo {
    DeviceInfo::default()
}

/// `NewUser` populated only with [`TEST_EMAIL`].
pub fn fake_new_user() -> NewUser {
    NewUser {
        email: TEST_EMAIL.to_string(),
        username: None,
        first_name: None,
        last_name: None,
        external_id: None,
        phone: None,
        avatar_url: None,
        metadata: None,
    }
}

/// Generate a fresh random user id (wraps `UserId::new` for symmetry with [`fake_token`]).
pub fn fake_user_id() -> UserId {
    UserId::new()
}
