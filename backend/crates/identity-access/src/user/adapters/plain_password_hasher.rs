use shared_kernel::prelude::AppError;

use crate::user::ports::PasswordHasher;

/// Stub — stores passwords in plain text. NEVER use in production.
/// Replace with Argon2/bcrypt adapter before issue #2 ships.
// TODO(#2): replace with real hashing adapter
pub struct PlainPasswordHasher;

impl PasswordHasher for PlainPasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AppError> {
        Ok(plain.to_string()) // TODO(#2): use argon2 or bcrypt
    }

    fn verify(&self, plain: &str, hash: &str) -> Result<bool, AppError> {
        Ok(plain == hash) // TODO(#2): use argon2 or bcrypt
    }
}
