use shared_kernel::prelude::AppError;

use crate::user::domain::{User, UserId};

/// Read port — single-entity lookup by identity (ADR-012).
pub trait UserFinder {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AppError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
}
