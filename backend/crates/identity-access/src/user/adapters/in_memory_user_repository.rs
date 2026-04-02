use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use shared_kernel::prelude::{AppError, Criteria, Op};

use crate::user::domain::{User, UserId};
use crate::user::ports::{UserField, UserFinder, UserRegistry, UserSearch};

/// In-memory test double — implements UserRepository (UserRegistry + UserFinder + UserSearch).
#[derive(Debug, Default, Clone)]
pub struct InMemoryUserRepository {
    store: Arc<RwLock<HashMap<String, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserRegistry for InMemoryUserRepository {
    fn save(&self, user: &User) -> Result<(), AppError> {
        self.store
            .write()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .insert(user.id.to_string(), user.clone());
        Ok(())
    }

    fn delete(&self, id: &UserId) -> Result<(), AppError> {
        self.store
            .write()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .remove(&id.to_string());
        Ok(())
    }
}

impl UserFinder for InMemoryUserRepository {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AppError> {
        Ok(self
            .store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .get(&id.to_string())
            .cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self
            .store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .values()
            .find(|u| u.email == email)
            .cloned())
    }
}

impl UserSearch for InMemoryUserRepository {
    fn find(&self, criteria: Criteria<UserField>) -> Result<Vec<User>, AppError> {
        // TODO: apply criteria.limit and criteria.offset after filtering
        let store = self
            .store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?;

        let results = store
            .values()
            .filter(|u| {
                criteria.filters.iter().all(|f| {
                    let field_val = match &f.field {
                        UserField::Email => u.email.clone(),
                        UserField::Role => u.role.to_string(),
                    };

                    match &f.op {
                        Op::Eq => field_val == f.value,
                        Op::NotEq => field_val != f.value,
                        Op::Like => field_val.contains(f.value.as_str()),
                        Op::Gt => field_val > f.value,
                        Op::Lt => field_val < f.value,
                        Op::Gte => field_val >= f.value,
                        Op::Lte => field_val <= f.value,
                    }
                })
            })
            .cloned()
            .collect();

        Ok(results)
    }

    fn count(&self, criteria: Criteria<UserField>) -> Result<u64, AppError> {
        self.find(criteria).map(|v| v.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_kernel::prelude::{Filter, Op};

    use crate::user::domain::Role;

    fn make_user(email: &str, role: Role) -> User {
        User::new(email, "hash", role)
    }

    #[test]
    fn save_and_find_by_id() {
        let repo = InMemoryUserRepository::new();
        let user = make_user("a@test.com", Role::Member);
        let id = user.id.clone();
        repo.save(&user).unwrap();

        let found = repo.find_by_id(&id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "a@test.com");
    }

    #[test]
    fn find_by_email_returns_correct_user() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("alice@test.com", Role::Admin))
            .unwrap();
        repo.save(&make_user("bob@test.com", Role::Member)).unwrap();

        let found = repo.find_by_email("bob@test.com").unwrap();
        assert_eq!(found.unwrap().email, "bob@test.com");
    }

    #[test]
    fn delete_removes_user() {
        let repo = InMemoryUserRepository::new();
        let user = make_user("del@test.com", Role::Viewer);
        let id = user.id.clone();
        repo.save(&user).unwrap();

        repo.delete(&id).unwrap();
        assert!(repo.find_by_id(&id).unwrap().is_none());
    }

    #[test]
    fn search_by_role_eq() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("a@test.com", Role::Admin)).unwrap();
        repo.save(&make_user("b@test.com", Role::Member)).unwrap();

        let criteria = Criteria::new().filter(Filter {
            field: UserField::Role,
            op: Op::Eq,
            value: "admin".into(),
        });
        let results = repo.find(criteria).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "a@test.com");
    }

    #[test]
    fn search_by_email_like() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("alice@example.com", Role::Member))
            .unwrap();
        repo.save(&make_user("bob@other.com", Role::Member))
            .unwrap();

        let criteria = Criteria::new().filter(Filter {
            field: UserField::Email,
            op: Op::Like,
            value: "example".into(),
        });
        let results = repo.find(criteria).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "alice@example.com");
    }

    #[test]
    fn count_matches_find_length() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("x@test.com", Role::Viewer)).unwrap();
        repo.save(&make_user("y@test.com", Role::Viewer)).unwrap();

        let criteria = Criteria::new().filter(Filter {
            field: UserField::Role,
            op: Op::Eq,
            value: "viewer".into(),
        });
        let count = repo.count(criteria.clone()).unwrap();
        let found = repo.find(criteria).unwrap();

        assert_eq!(count, found.len() as u64);
    }
}
