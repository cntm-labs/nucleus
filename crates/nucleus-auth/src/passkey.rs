use base64::Engine;
use nucleus_core::error::{AppError, AuthError};
use nucleus_core::types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Passkey registration challenge (stored in Redis during registration ceremony)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyChallenge {
    pub challenge_id: String,
    pub user_id: UserId,
    pub challenge_bytes: Vec<u8>,
    pub created_at: i64,
    pub expires_at: i64, // 5 minutes from creation
}

/// Stored passkey credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub sign_count: u32,
    pub user_id: UserId,
    pub created_at: i64,
}

/// Passkey registration options (sent to client)
#[derive(Debug, Serialize)]
pub struct RegistrationOptions {
    pub challenge: String, // base64url encoded
    pub rp: RelyingParty,
    pub user: PublicKeyUser,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: u64, // 300000ms (5 min)
    pub authenticator_selection: AuthenticatorSelection,
    pub attestation: String, // "none"
}

#[derive(Debug, Serialize)]
pub struct RelyingParty {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct PublicKeyUser {
    pub id: String,           // base64url user ID
    pub name: String,         // email
    pub display_name: String, // display name
}

#[derive(Debug, Serialize)]
pub struct PubKeyCredParam {
    #[serde(rename = "type")]
    pub type_: String, // "public-key"
    pub alg: i32, // -7 (ES256) or -257 (RS256)
}

#[derive(Debug, Serialize)]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<String>,
    pub resident_key: String,      // "required"
    pub user_verification: String, // "required"
}

/// Authentication options (sent to client)
#[derive(Debug, Serialize)]
pub struct AuthenticationOptions {
    pub challenge: String,
    pub timeout: u64,
    pub rp_id: String,
    pub user_verification: String,
    pub allow_credentials: Vec<AllowCredential>,
}

#[derive(Debug, Serialize)]
pub struct AllowCredential {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

pub struct PasskeyService {
    rp_name: String,
    rp_id: String,
}

impl PasskeyService {
    pub fn new(rp_name: &str, rp_id: &str) -> Self {
        Self {
            rp_name: rp_name.to_string(),
            rp_id: rp_id.to_string(),
        }
    }

    /// Begin passkey registration ceremony
    pub fn begin_registration(
        &self,
        user_id: &UserId,
        user_email: &str,
        user_display_name: &str,
    ) -> Result<(RegistrationOptions, PasskeyChallenge), AppError> {
        let challenge_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let challenge_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&challenge_bytes);

        let now = chrono::Utc::now().timestamp();
        let challenge = PasskeyChallenge {
            challenge_id: format!("passkey_{}", Uuid::new_v4()),
            user_id: *user_id,
            challenge_bytes: challenge_bytes.clone(),
            created_at: now,
            expires_at: now + 300, // 5 min
        };

        let options = RegistrationOptions {
            challenge: challenge_b64,
            rp: RelyingParty {
                name: self.rp_name.clone(),
                id: self.rp_id.clone(),
            },
            user: PublicKeyUser {
                id: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(user_id.0.as_bytes()),
                name: user_email.to_string(),
                display_name: user_display_name.to_string(),
            },
            pub_key_cred_params: vec![
                PubKeyCredParam {
                    type_: "public-key".to_string(),
                    alg: -7,
                }, // ES256
                PubKeyCredParam {
                    type_: "public-key".to_string(),
                    alg: -257,
                }, // RS256
            ],
            timeout: 300_000,
            authenticator_selection: AuthenticatorSelection {
                authenticator_attachment: None,
                resident_key: "required".to_string(),
                user_verification: "required".to_string(),
            },
            attestation: "none".to_string(),
        };

        Ok((options, challenge))
    }

    /// Begin passkey authentication ceremony
    pub fn begin_authentication(
        &self,
        credentials: &[PasskeyCredential],
    ) -> Result<(AuthenticationOptions, PasskeyChallenge), AppError> {
        let challenge_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let challenge_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&challenge_bytes);

        let now = chrono::Utc::now().timestamp();
        let challenge = PasskeyChallenge {
            challenge_id: format!("passkey_{}", Uuid::new_v4()),
            user_id: credentials.first().map(|c| c.user_id).unwrap_or_default(),
            challenge_bytes,
            created_at: now,
            expires_at: now + 300,
        };

        let allow_credentials = credentials
            .iter()
            .map(|c| AllowCredential {
                id: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&c.credential_id),
                type_: "public-key".to_string(),
            })
            .collect();

        let options = AuthenticationOptions {
            challenge: challenge_b64,
            timeout: 300_000,
            rp_id: self.rp_id.clone(),
            user_verification: "required".to_string(),
            allow_credentials,
        };

        Ok((options, challenge))
    }

    /// Verify a challenge has not expired
    pub fn verify_challenge_not_expired(challenge: &PasskeyChallenge) -> Result<(), AppError> {
        let now = chrono::Utc::now().timestamp();
        if now > challenge.expires_at {
            return Err(AppError::Auth(AuthError::PasskeyChallenged));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> PasskeyService {
        PasskeyService::new("Nucleus Test", "nucleus.test")
    }

    #[test]
    fn begin_registration_returns_valid_options() {
        let svc = test_service();
        let user_id = UserId::new();
        let (options, challenge) = svc
            .begin_registration(&user_id, "test@example.com", "Test User")
            .unwrap();

        assert_eq!(options.rp.name, "Nucleus Test");
        assert_eq!(options.rp.id, "nucleus.test");
        assert_eq!(options.attestation, "none");
        assert_eq!(options.timeout, 300_000);
        assert!(!options.challenge.is_empty());
        assert!(!challenge.challenge_id.is_empty());
    }

    #[test]
    fn registration_options_include_es256_and_rs256() {
        let svc = test_service();
        let (options, _) = svc
            .begin_registration(&UserId::new(), "test@example.com", "Test")
            .unwrap();
        let algs: Vec<i32> = options.pub_key_cred_params.iter().map(|p| p.alg).collect();
        assert!(algs.contains(&-7)); // ES256
        assert!(algs.contains(&-257)); // RS256
    }

    #[test]
    fn begin_authentication_with_credentials() {
        let svc = test_service();
        let creds = vec![PasskeyCredential {
            credential_id: vec![1, 2, 3],
            public_key: vec![4, 5, 6],
            sign_count: 0,
            user_id: UserId::new(),
            created_at: chrono::Utc::now().timestamp(),
        }];
        let (options, _) = svc.begin_authentication(&creds).unwrap();
        assert_eq!(options.allow_credentials.len(), 1);
        assert_eq!(options.rp_id, "nucleus.test");
    }

    #[test]
    fn challenge_expires_after_5_minutes() {
        let challenge = PasskeyChallenge {
            challenge_id: "test".to_string(),
            user_id: UserId::new(),
            challenge_bytes: vec![],
            created_at: chrono::Utc::now().timestamp() - 400,
            expires_at: chrono::Utc::now().timestamp() - 100, // expired
        };
        let result = PasskeyService::verify_challenge_not_expired(&challenge);
        assert!(result.is_err());
    }

    #[test]
    fn valid_challenge_not_expired() {
        let now = chrono::Utc::now().timestamp();
        let challenge = PasskeyChallenge {
            challenge_id: "test".to_string(),
            user_id: UserId::new(),
            challenge_bytes: vec![],
            created_at: now,
            expires_at: now + 300,
        };
        assert!(PasskeyService::verify_challenge_not_expired(&challenge).is_ok());
    }

    #[test]
    fn unique_challenges_per_registration() {
        let svc = test_service();
        let (opts1, _) = svc
            .begin_registration(&UserId::new(), "a@test.com", "A")
            .unwrap();
        let (opts2, _) = svc
            .begin_registration(&UserId::new(), "b@test.com", "B")
            .unwrap();
        assert_ne!(opts1.challenge, opts2.challenge);
    }
}
