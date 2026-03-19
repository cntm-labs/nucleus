use nucleus_core::error::{AppError, OrgError};
use nucleus_core::types::*;
use serde::{Deserialize, Serialize};

use crate::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: PermissionId,
    pub project_id: ProjectId,
    pub key: String,
    pub description: Option<String>,
}

/// Permission checker — resolves user's effective permissions in an org
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if a user has a specific permission in an org.
    /// Resolution: user -> org_member -> role -> role_permissions -> permissions
    pub fn has_permission(
        user_role: &Role,
        role_permissions: &[Permission],
        required_permission: &str,
    ) -> bool {
        // System roles have implicit permissions
        match user_role.name.as_str() {
            "owner" => true, // owners have ALL permissions
            "admin" => {
                // admins have everything except ownership transfer
                required_permission != "org:delete" && required_permission != "org:transfer"
            }
            _ => {
                // Custom roles: check explicit permissions
                role_permissions.iter().any(|p| p.key == required_permission)
            }
        }
    }

    /// Check if user can perform action, return error if not
    pub fn require_permission(
        user_role: &Role,
        role_permissions: &[Permission],
        required_permission: &str,
    ) -> Result<(), AppError> {
        if Self::has_permission(user_role, role_permissions, required_permission) {
            Ok(())
        } else {
            Err(AppError::Org(OrgError::InsufficientPermissions))
        }
    }
}

/// Default permissions for a new project
pub fn default_permissions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("members:read", "View organization members"),
        ("members:invite", "Invite new members"),
        ("members:remove", "Remove members from organization"),
        ("members:role", "Change member roles"),
        ("org:read", "View organization details"),
        ("org:update", "Update organization settings"),
        ("org:delete", "Delete the organization"),
        ("org:transfer", "Transfer organization ownership"),
        ("billing:read", "View billing information"),
        ("billing:manage", "Manage billing and subscriptions"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner_role() -> Role {
        Role {
            id: RoleId::new(),
            project_id: ProjectId::new(),
            org_id: None,
            name: "owner".to_string(),
            description: None,
            is_system: true,
        }
    }

    fn admin_role() -> Role {
        Role {
            name: "admin".to_string(),
            is_system: true,
            ..owner_role()
        }
    }

    fn member_role() -> Role {
        Role {
            name: "member".to_string(),
            is_system: true,
            ..owner_role()
        }
    }

    fn custom_role() -> Role {
        Role {
            name: "billing-viewer".to_string(),
            is_system: false,
            ..owner_role()
        }
    }

    fn billing_read_perm() -> Permission {
        Permission {
            id: PermissionId::new(),
            project_id: ProjectId::new(),
            key: "billing:read".to_string(),
            description: Some("View billing".to_string()),
        }
    }

    #[test]
    fn owner_has_all_permissions() {
        assert!(PermissionChecker::has_permission(&owner_role(), &[], "anything"));
        assert!(PermissionChecker::has_permission(&owner_role(), &[], "org:delete"));
        assert!(PermissionChecker::has_permission(&owner_role(), &[], "org:transfer"));
    }

    #[test]
    fn admin_has_most_permissions() {
        assert!(PermissionChecker::has_permission(&admin_role(), &[], "members:invite"));
        assert!(PermissionChecker::has_permission(&admin_role(), &[], "billing:manage"));
    }

    #[test]
    fn admin_cannot_delete_or_transfer_org() {
        assert!(!PermissionChecker::has_permission(&admin_role(), &[], "org:delete"));
        assert!(!PermissionChecker::has_permission(&admin_role(), &[], "org:transfer"));
    }

    #[test]
    fn member_has_no_implicit_permissions() {
        assert!(!PermissionChecker::has_permission(&member_role(), &[], "members:invite"));
    }

    #[test]
    fn custom_role_uses_explicit_permissions() {
        let perms = vec![billing_read_perm()];
        assert!(PermissionChecker::has_permission(&custom_role(), &perms, "billing:read"));
        assert!(!PermissionChecker::has_permission(&custom_role(), &perms, "members:invite"));
    }

    #[test]
    fn require_permission_returns_error_on_denied() {
        let result = PermissionChecker::require_permission(&member_role(), &[], "members:invite");
        assert!(matches!(result, Err(AppError::Org(OrgError::InsufficientPermissions))));
    }

    #[test]
    fn require_permission_ok_for_owner() {
        assert!(PermissionChecker::require_permission(&owner_role(), &[], "anything").is_ok());
    }

    #[test]
    fn default_permissions_list_is_complete() {
        let defaults = default_permissions();
        assert!(defaults.len() >= 10);
        assert!(defaults.iter().any(|(k, _)| *k == "members:invite"));
        assert!(defaults.iter().any(|(k, _)| *k == "org:delete"));
        assert!(defaults.iter().any(|(k, _)| *k == "billing:manage"));
    }
}
