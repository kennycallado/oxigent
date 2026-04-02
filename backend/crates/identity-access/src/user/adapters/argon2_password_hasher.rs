use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
use shared_kernel::prelude::AppError;
use uuid::Uuid;

use crate::user::ports::PasswordHasher;

pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AppError> {
        let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
            .map_err(|e| AppError::new("E_INTERNAL", format!("salt generation failed: {e}")))?;
        let argon2 = Argon2::default();
        argon2
            .hash_password(plain.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::new("E_INTERNAL", format!("hashing failed: {e}")))
    }

    fn verify(&self, plain: &str, hash: &str) -> Result<bool, AppError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| AppError::new("E_INTERNAL", format!("invalid hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_round_trip() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash("my-secret-password").unwrap();
        assert!(hasher.verify("my-secret-password", &hash).unwrap());
    }

    #[test]
    fn wrong_password_returns_false() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash("correct-password").unwrap();
        assert!(!hasher.verify("wrong-password", &hash).unwrap());
    }
}
