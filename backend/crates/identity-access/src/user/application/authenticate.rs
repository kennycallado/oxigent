use shared_kernel::prelude::AppError;

use crate::user::domain::User;
use crate::user::ports::{PasswordHasher, UserFinder};

pub struct AuthenticateUserCommand {
    pub email: String,
    pub password: String,
}

pub struct AuthenticateUser<F: UserFinder, H: PasswordHasher> {
    pub finder: F,
    pub hasher: H,
}

impl<F: UserFinder, H: PasswordHasher> AuthenticateUser<F, H> {
    pub fn execute(&self, cmd: AuthenticateUserCommand) -> Result<User, AppError> {
        let unauthorized = || AppError::new("E_UNAUTHORIZED", "invalid credentials");

        let user = self
            .finder
            .find_by_email(&cmd.email)
            .map_err(|_| unauthorized())?
            .ok_or_else(unauthorized)?;

        let valid = self
            .hasher
            .verify(&cmd.password, &user.password_hash)
            .map_err(|_| unauthorized())?;

        if valid {
            Ok(user)
        } else {
            Err(unauthorized())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::adapters::{Argon2PasswordHasher, InMemoryUserRepository};
    use crate::user::domain::Role;
    use crate::user::ports::{UserFinder, UserRegistry};

    fn setup() -> (InMemoryUserRepository, Argon2PasswordHasher) {
        (InMemoryUserRepository::new(), Argon2PasswordHasher)
    }

    #[test]
    fn authenticate_valid_credentials_returns_user() {
        let (repo, hasher) = setup();
        let hash = hasher.hash("correct-password").unwrap();
        let user = crate::user::domain::User::new("alice@example.com", hash, Role::Member);
        repo.save(&user).unwrap();

        let svc = AuthenticateUser {
            finder: repo,
            hasher: Argon2PasswordHasher,
        };
        let result = svc.execute(AuthenticateUserCommand {
            email: "alice@example.com".into(),
            password: "correct-password".into(),
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "alice@example.com");
    }

    #[test]
    fn authenticate_wrong_password_returns_unauthorized() {
        let (repo, hasher) = setup();
        let hash = hasher.hash("correct-password").unwrap();
        let user = crate::user::domain::User::new("alice@example.com", hash, Role::Member);
        repo.save(&user).unwrap();

        let svc = AuthenticateUser {
            finder: repo,
            hasher: Argon2PasswordHasher,
        };
        let result = svc.execute(AuthenticateUserCommand {
            email: "alice@example.com".into(),
            password: "wrong-password".into(),
        });
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_UNAUTHORIZED");
    }

    #[test]
    fn authenticate_unknown_email_returns_unauthorized() {
        let (repo, _) = setup();
        let svc = AuthenticateUser {
            finder: repo,
            hasher: Argon2PasswordHasher,
        };
        let result = svc.execute(AuthenticateUserCommand {
            email: "nobody@example.com".into(),
            password: "any-password".into(),
        });
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_UNAUTHORIZED");
    }
}
