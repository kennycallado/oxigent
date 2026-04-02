use super::{UserFinder, UserRegistry, UserSearch};

/// Composed alias — do not add methods here (ADR-012).
pub trait UserRepository: UserRegistry + UserFinder + UserSearch {}

impl<T> UserRepository for T where T: UserRegistry + UserFinder + UserSearch {}
