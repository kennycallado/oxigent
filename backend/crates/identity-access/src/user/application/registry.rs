use shared_kernel::prelude::AppError;

use crate::user::domain::{Role, User};
use crate::user::ports::{PasswordHasher, UserRegistry};

pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
    pub role: Role,
}

/// Use case stub — creates a new User and persists it via UserRegistry.
// TODO(#2): add validation (email format, password strength), duplicate check
pub struct RegisterUser<R: UserRegistry, H: PasswordHasher> {
    pub repository: R,
    pub hasher: H,
}

impl<R: UserRegistry, H: PasswordHasher> RegisterUser<R, H> {
    pub fn execute(&self, cmd: RegisterUserCommand) -> Result<User, AppError> {
        let hash = self.hasher.hash(&cmd.password)?;
        let user = User::new(cmd.email, hash, cmd.role);
        self.repository.save(&user)?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::adapters::{Argon2PasswordHasher, InMemoryUserRepository};
    use crate::user::ports::UserFinder;

    #[test]
    fn register_user_persists_and_returns_user() {
        let repo = InMemoryUserRepository::new();
        let uc = RegisterUser {
            repository: repo.clone(),
            hasher: Argon2PasswordHasher,
        };
        let result = uc.execute(RegisterUserCommand {
            email: "new@test.com".into(),
            password: "secret".into(),
            role: Role::Member,
        });
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "new@test.com");
        assert_eq!(user.role, Role::Member);

        // Verify the user was actually stored (not just returned)
        let stored = uc.repository.find_by_id(&user.id).unwrap();
        assert!(stored.is_some(), "user should be persisted in the store");
        assert_eq!(stored.unwrap().email, "new@test.com");
    }
}
