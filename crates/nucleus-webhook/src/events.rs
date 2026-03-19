use serde::{Deserialize, Serialize};

/// All webhook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebhookEventType {
    // User events
    #[serde(rename = "user.created")]
    UserCreated,
    #[serde(rename = "user.updated")]
    UserUpdated,
    #[serde(rename = "user.deleted")]
    UserDeleted,
    #[serde(rename = "user.banned")]
    UserBanned,
    #[serde(rename = "user.unbanned")]
    UserUnbanned,

    // Session events
    #[serde(rename = "session.created")]
    SessionCreated,
    #[serde(rename = "session.revoked")]
    SessionRevoked,

    // Organization events
    #[serde(rename = "org.created")]
    OrgCreated,
    #[serde(rename = "org.updated")]
    OrgUpdated,
    #[serde(rename = "org.deleted")]
    OrgDeleted,
    #[serde(rename = "org.member.added")]
    OrgMemberAdded,
    #[serde(rename = "org.member.removed")]
    OrgMemberRemoved,
    #[serde(rename = "org.member.role_changed")]
    OrgMemberRoleChanged,

    // MFA events
    #[serde(rename = "mfa.enabled")]
    MfaEnabled,
    #[serde(rename = "mfa.disabled")]
    MfaDisabled,

    // Security events
    #[serde(rename = "security.brute_force_detected")]
    BruteForceDetected,
    #[serde(rename = "security.account_locked")]
    AccountLocked,
    #[serde(rename = "security.suspicious_login")]
    SuspiciousLogin,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UserCreated => "user.created",
            Self::UserUpdated => "user.updated",
            Self::UserDeleted => "user.deleted",
            Self::UserBanned => "user.banned",
            Self::UserUnbanned => "user.unbanned",
            Self::SessionCreated => "session.created",
            Self::SessionRevoked => "session.revoked",
            Self::OrgCreated => "org.created",
            Self::OrgUpdated => "org.updated",
            Self::OrgDeleted => "org.deleted",
            Self::OrgMemberAdded => "org.member.added",
            Self::OrgMemberRemoved => "org.member.removed",
            Self::OrgMemberRoleChanged => "org.member.role_changed",
            Self::MfaEnabled => "mfa.enabled",
            Self::MfaDisabled => "mfa.disabled",
            Self::BruteForceDetected => "security.brute_force_detected",
            Self::AccountLocked => "security.account_locked",
            Self::SuspiciousLogin => "security.suspicious_login",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_serializes_correctly() {
        let event = WebhookEventType::UserCreated;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""user.created""#);

        let event = WebhookEventType::OrgMemberRoleChanged;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""org.member.role_changed""#);

        let event = WebhookEventType::BruteForceDetected;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""security.brute_force_detected""#);
    }

    #[test]
    fn event_type_deserializes_correctly() {
        let event: WebhookEventType = serde_json::from_str(r#""user.created""#).unwrap();
        assert_eq!(event, WebhookEventType::UserCreated);

        let event: WebhookEventType = serde_json::from_str(r#""session.revoked""#).unwrap();
        assert_eq!(event, WebhookEventType::SessionRevoked);
    }

    #[test]
    fn event_type_as_str() {
        assert_eq!(WebhookEventType::UserCreated.as_str(), "user.created");
        assert_eq!(WebhookEventType::UserUpdated.as_str(), "user.updated");
        assert_eq!(WebhookEventType::UserDeleted.as_str(), "user.deleted");
        assert_eq!(WebhookEventType::UserBanned.as_str(), "user.banned");
        assert_eq!(WebhookEventType::UserUnbanned.as_str(), "user.unbanned");
        assert_eq!(WebhookEventType::SessionCreated.as_str(), "session.created");
        assert_eq!(WebhookEventType::SessionRevoked.as_str(), "session.revoked");
        assert_eq!(WebhookEventType::OrgCreated.as_str(), "org.created");
        assert_eq!(WebhookEventType::OrgUpdated.as_str(), "org.updated");
        assert_eq!(WebhookEventType::OrgDeleted.as_str(), "org.deleted");
        assert_eq!(WebhookEventType::OrgMemberAdded.as_str(), "org.member.added");
        assert_eq!(WebhookEventType::OrgMemberRemoved.as_str(), "org.member.removed");
        assert_eq!(
            WebhookEventType::OrgMemberRoleChanged.as_str(),
            "org.member.role_changed"
        );
        assert_eq!(WebhookEventType::MfaEnabled.as_str(), "mfa.enabled");
        assert_eq!(WebhookEventType::MfaDisabled.as_str(), "mfa.disabled");
        assert_eq!(
            WebhookEventType::BruteForceDetected.as_str(),
            "security.brute_force_detected"
        );
        assert_eq!(
            WebhookEventType::AccountLocked.as_str(),
            "security.account_locked"
        );
        assert_eq!(
            WebhookEventType::SuspiciousLogin.as_str(),
            "security.suspicious_login"
        );
    }
}
