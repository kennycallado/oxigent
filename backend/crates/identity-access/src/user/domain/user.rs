use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Role;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: String, // TODO(#2): restrict visibility once PasswordHasher port is used end-to-end
    pub role: Role,
}

impl User {
    pub fn new(email: impl Into<String>, password_hash: impl Into<String>, role: Role) -> Self {
        Self {
            id: UserId::new(),
            email: email.into(),
            password_hash: password_hash.into(),
            role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_new_generates_unique_ids() {
        let a = User::new("a@test.com", "hash_a", Role::Member);
        let b = User::new("b@test.com", "hash_b", Role::Admin);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn user_stores_email_and_role() {
        let u = User::new("x@test.com", "h", Role::Viewer);
        assert_eq!(u.email, "x@test.com");
        assert_eq!(u.role, Role::Viewer);
    }
}
