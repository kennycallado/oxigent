use shared_kernel::prelude::{AppError, Criteria};

use crate::user::domain::User;
use crate::user::ports::{UserField, UserSearch};

pub struct FindUsersQuery {
    pub criteria: Criteria<UserField>,
}

/// Use case stub — queries users via UserSearch port using Criteria<UserField>.
pub struct FindUsers<S: UserSearch> {
    pub repository: S,
}

impl<S: UserSearch> FindUsers<S> {
    pub fn execute(&self, query: FindUsersQuery) -> Result<Vec<User>, AppError> {
        self.repository.find(query.criteria)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_kernel::prelude::{Filter, Op};

    use crate::user::adapters::InMemoryUserRepository;
    use crate::user::domain::{Role, User};
    use crate::user::ports::UserRegistry;

    #[test]
    fn find_users_delegates_to_search_port() {
        let repo = InMemoryUserRepository::new();
        repo.save(&User::new("z@test.com", "h", Role::Admin))
            .unwrap();

        let uc = FindUsers { repository: repo };
        let query = FindUsersQuery {
            criteria: Criteria::new().filter(Filter {
                field: UserField::Role,
                op: Op::Eq,
                value: "admin".into(),
            }),
        };
        let result = uc.execute(query).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].email, "z@test.com");
    }
}
