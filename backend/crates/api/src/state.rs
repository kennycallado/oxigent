use identity_access::user::adapters::{Argon2PasswordHasher, InMemoryUserRepository};
use identity_access::user::application::AuthenticateUser;

use crate::deny_list::DenyList;
use crate::jwt::JwtService;

pub struct AppState {
    pub authenticate: AuthenticateUser<InMemoryUserRepository, Argon2PasswordHasher>,
    pub jwt: JwtService,
    pub deny_list: DenyList,
}
