use chrono::{DateTime, Duration, Utc};
use nucleus_core::crypto;
use nucleus_core::error::{AppError, AuthError, OrgError};
use nucleus_core::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: InvitationId,
    pub org_id: OrgId,
    pub email: String,
    pub role_id: RoleId,
    pub invited_by: UserId,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

pub struct InvitationGenerated {
    pub invitation_token: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

pub struct InvitationService;

impl InvitationService {
    /// Generate an invitation token (7-day expiry)
    pub fn generate() -> InvitationGenerated {
        let token = crypto::generate_token();
        let token_hash = crypto::generate_token_hash(&token);
        let expires_at = Utc::now() + Duration::days(7);

        InvitationGenerated {
            invitation_token: token,
            token_hash,
            expires_at,
        }
    }

    /// Verify an invitation token can be accepted
    pub fn verify_token(
        provided_token: &str,
        stored_hash: &str,
        status: &InvitationStatus,
        expires_at: &DateTime<Utc>,
    ) -> Result<(), AppError> {
        // 1. Check status
        match status {
            InvitationStatus::Pending => {} // ok, continue
            InvitationStatus::Accepted => {
                return Err(AppError::Org(OrgError::InvitationAlreadyUsed));
            }
            InvitationStatus::Expired | InvitationStatus::Revoked => {
                return Err(AppError::Org(OrgError::InvitationExpired));
            }
        }

        // 2. Check expiry
        if Utc::now() > *expires_at {
            return Err(AppError::Org(OrgError::InvitationExpired));
        }

        // 3. Verify hash
        let hash = crypto::generate_token_hash(provided_token);
        if hash != stored_hash {
            return Err(AppError::Auth(AuthError::TokenInvalid));
        }

        Ok(())
    }

    /// Build invitation URL
    pub fn build_url(base_url: &str, org_slug: &str, token: &str) -> String {
        format!("{}/orgs/{}/invite?token={}", base_url, org_slug, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_token_and_hash() {
        let result = InvitationService::generate();
        assert!(!result.invitation_token.is_empty());
        assert!(!result.token_hash.is_empty());
        assert_ne!(result.invitation_token, result.token_hash);
    }

    #[test]
    fn expires_in_7_days() {
        let result = InvitationService::generate();
        let expected = Utc::now() + Duration::days(7);
        let diff = (result.expires_at - expected).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn verify_valid_pending_invitation() {
        let gen = InvitationService::generate();
        let result = InvitationService::verify_token(
            &gen.invitation_token,
            &gen.token_hash,
            &InvitationStatus::Pending,
            &gen.expires_at,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verify_rejects_already_accepted() {
        let gen = InvitationService::generate();
        let result = InvitationService::verify_token(
            &gen.invitation_token,
            &gen.token_hash,
            &InvitationStatus::Accepted,
            &gen.expires_at,
        );
        assert!(matches!(
            result,
            Err(AppError::Org(OrgError::InvitationAlreadyUsed))
        ));
    }

    #[test]
    fn verify_rejects_expired() {
        let gen = InvitationService::generate();
        let expired = Utc::now() - Duration::hours(1);
        let result = InvitationService::verify_token(
            &gen.invitation_token,
            &gen.token_hash,
            &InvitationStatus::Pending,
            &expired,
        );
        assert!(matches!(
            result,
            Err(AppError::Org(OrgError::InvitationExpired))
        ));
    }

    #[test]
    fn verify_rejects_revoked() {
        let gen = InvitationService::generate();
        let result = InvitationService::verify_token(
            &gen.invitation_token,
            &gen.token_hash,
            &InvitationStatus::Revoked,
            &gen.expires_at,
        );
        assert!(matches!(
            result,
            Err(AppError::Org(OrgError::InvitationExpired))
        ));
    }

    #[test]
    fn verify_rejects_wrong_token() {
        let gen = InvitationService::generate();
        let result = InvitationService::verify_token(
            "wrong_token",
            &gen.token_hash,
            &InvitationStatus::Pending,
            &gen.expires_at,
        );
        assert!(result.is_err());
    }

    #[test]
    fn build_url_correct_format() {
        let url = InvitationService::build_url("https://nucleus.dev", "acme", "tok_123");
        assert_eq!(
            url,
            "https://nucleus.dev/orgs/acme/invite?token=tok_123"
        );
    }

    #[test]
    fn unique_tokens() {
        let g1 = InvitationService::generate();
        let g2 = InvitationService::generate();
        assert_ne!(g1.invitation_token, g2.invitation_token);
    }
}
