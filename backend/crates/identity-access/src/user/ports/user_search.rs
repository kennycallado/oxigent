use shared_kernel::prelude::{AppError, Criteria};

use crate::user::domain::User;

/// Context-local field enum for Criteria<UserField> queries (ADR-011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserField {
    Email,
    Role,
}

/// Read port — collection queries via Criteria<UserField> (ADR-012).
pub trait UserSearch {
    fn find(&self, criteria: Criteria<UserField>) -> Result<Vec<User>, AppError>;
    fn count(&self, criteria: Criteria<UserField>) -> Result<u64, AppError>;
}
