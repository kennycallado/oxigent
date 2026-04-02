use shared_kernel::prelude::AppError;

use crate::user::domain::{User, UserId};

/// Write port — persists or removes a User aggregate (ADR-012).
pub trait UserRegistry {
    fn save(&self, user: &User) -> Result<(), AppError>;
    fn delete(&self, id: &UserId) -> Result<(), AppError>;
}
