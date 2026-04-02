use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    Admin,
    #[default]
    Member,
    Viewer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Member => write!(f, "member"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}
