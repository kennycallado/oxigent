use identity_access::user::adapters::{Argon2PasswordHasher, InMemoryUserRepository};
use identity_access::user::application::AuthenticateUser;

use crate::deny_list::DenyList;
use crate::jwt::JwtService;

/// Concrete AppState for Fase 1 (in-memory repository).
/// Replace repository with SurrealDB adapter in a later milestone.
pub struct AppState {
    pub authenticate: AuthenticateUser<InMemoryUserRepository, Argon2PasswordHasher>,
    pub jwt: JwtService,
    pub deny_list: DenyList,
}

impl AppState {
    pub fn new(jwt_secret: &str, jwt_expiry_secs: u64) -> Self {
        Self {
            authenticate: AuthenticateUser {
                finder: InMemoryUserRepository::new(),
                hasher: Argon2PasswordHasher,
            },
            jwt: JwtService::new(jwt_secret, jwt_expiry_secs),
            deny_list: DenyList::new(),
        }
    }
}
