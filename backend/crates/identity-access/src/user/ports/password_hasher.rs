use shared_kernel::prelude::AppError;

/// Domain service port — hashes and verifies passwords (ADR-012, exempt from suffix rule).
pub trait PasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AppError>;
    fn verify(&self, plain: &str, hash: &str) -> Result<bool, AppError>;
}
