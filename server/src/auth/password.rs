use crate::core::crypto;
use crate::core::error::AppError;
use crate::core::validation;

pub struct PasswordService;

impl PasswordService {
    /// Validate password policy and hash it
    pub fn hash(password: &str) -> Result<String, AppError> {
        validation::validate_password(password)?;
        crypto::hash_password(password)
    }

    /// Verify a password against a stored hash
    pub fn verify(password: &str, hash: &str) -> Result<bool, AppError> {
        crypto::verify_password(password, hash)
    }

    /// Check if a password hash needs rehashing (params upgraded)
    /// Returns true if the hash was created with older/weaker parameters
    pub fn needs_rehash(hash: &str) -> bool {
        // Parse the PHC string and check if params match current config
        // Current: argon2id, m=19456, t=2, p=1
        // If hash has different params, it needs rehash
        if !hash.starts_with("$argon2id$") {
            return true;
        }
        // Check if memory/iteration params match
        !hash.contains("m=19456") || !hash.contains("t=2") || !hash.contains("p=1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_valid_password() {
        let hash = PasswordService::hash("SecurePass123!").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn hash_rejects_too_short() {
        let result = PasswordService::hash("short");
        assert!(result.is_err());
    }

    #[test]
    fn hash_rejects_too_long() {
        let long_pass = "a".repeat(129);
        let result = PasswordService::hash(&long_pass);
        assert!(result.is_err());
    }

    #[test]
    fn verify_correct_password() {
        let hash = PasswordService::hash("SecurePass123!").unwrap();
        assert!(PasswordService::verify("SecurePass123!", &hash).unwrap());
    }

    #[test]
    fn verify_wrong_password() {
        let hash = PasswordService::hash("SecurePass123!").unwrap();
        assert!(!PasswordService::verify("WrongPass456!", &hash).unwrap());
    }

    #[test]
    fn needs_rehash_false_for_current_params() {
        let hash = PasswordService::hash("SecurePass123!").unwrap();
        assert!(!PasswordService::needs_rehash(&hash));
    }

    #[test]
    fn needs_rehash_true_for_non_argon2id() {
        assert!(PasswordService::needs_rehash("$2b$12$somebcrypthash"));
    }

    #[test]
    fn needs_rehash_true_for_old_params() {
        // Simulated hash with different memory param
        assert!(PasswordService::needs_rehash(
            "$argon2id$v=19$m=4096,t=3,p=1$salt$hash"
        ));
    }

    #[test]
    fn hash_produces_unique_hashes() {
        let h1 = PasswordService::hash("SamePassword!").unwrap();
        let h2 = PasswordService::hash("SamePassword!").unwrap();
        assert_ne!(h1, h2); // different salts
    }

    #[test]
    fn accepts_max_length_password() {
        let pass = "a".repeat(128);
        assert!(PasswordService::hash(&pass).is_ok());
    }

    #[test]
    fn accepts_min_length_password() {
        assert!(PasswordService::hash("12345678").is_ok());
    }
}
