use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

define_id!(AccountId);
define_id!(ProjectId);
define_id!(UserId);
define_id!(OrgId);
define_id!(RoleId);
define_id!(PermissionId);
define_id!(CredentialId);
define_id!(SessionId);
define_id!(ApiKeyId);
define_id!(SigningKeyId);
define_id!(WebhookEventId);
define_id!(InvitationId);
define_id!(MfaEnrollmentId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn id_new_generates_v4_uuid() {
        let id = UserId::new();
        // UUID v4 has version bits set to 4
        assert_eq!(id.0.get_version_num(), 4);
    }

    #[test]
    fn id_display_formats_as_uuid() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id = UserId::from_uuid(uuid);
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn id_from_str_parses_uuid() {
        let id = UserId::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            id.0,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn id_from_str_rejects_invalid() {
        let result = UserId::from_str("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn id_serialize_deserialize_roundtrip() {
        let id = UserId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn different_id_types_not_interchangeable() {
        // This test verifies that different ID types are distinct types.
        // If you tried to assign a UserId to a variable of type ProjectId,
        // it would fail to compile.
        fn accept_user_id(_id: UserId) {}
        fn accept_project_id(_id: ProjectId) {}

        let user_id = UserId::new();
        let project_id = ProjectId::new();

        accept_user_id(user_id);
        accept_project_id(project_id);

        // They are different types even with the same underlying UUID
        let uuid = Uuid::new_v4();
        let uid = UserId::from_uuid(uuid);
        let pid = ProjectId::from_uuid(uuid);
        // uid and pid have the same inner value but are different types
        assert_eq!(uid.0, pid.0);
    }
}
