# Identity-Access Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> **IMPORTANT:** Read AGENTS.md before starting. If ULTRATHINK is needed, it will be indicated per task.

**Goal:** Scaffold `backend/crates/identity-access` with a `user/` module following the vertical slice structure (ADR-002), including domain types, port traits, and in-memory adapter stubs — providing the foundation for the auth endpoint in issue #2.

**Architecture:** Vertical slice `src/user/{domain,application,ports,adapters}/` per ADR-002. Domain defines `User` aggregate + `Role` value object. Ports follow ADR-012 naming (`UserRegistry`, `UserSearch`, `UserFinder`, composed `UserRepository`). Queries use `Criteria<UserField>` from `shared-kernel` per ADR-011. Application layer contains `RegisterUser` and `FindUsers` use-case stubs. Errors use `AppError` from shared-kernel per ADR-009.

**Tech Stack:** Rust (edition 2024), `shared-kernel` crate, `uuid` (v7), `chrono`, `thiserror`, `serde`

---

## File Map

```
backend/crates/shared-kernel/src/
  criteria.rs                          CREATE — Criteria<F>, Filter<F>, Op
  errors.rs                            CREATE — AppError, ErrorDetail
  prelude.rs                           MODIFY — re-export Criteria, AppError, ErrorDetail

backend/crates/identity-access/src/
  lib.rs                               MODIFY — replace flat modules with pub mod user
  user/
    mod.rs                             CREATE
    domain/
      mod.rs                           CREATE
      user.rs                          CREATE — UserId, User
      role.rs                          CREATE — Role enum
    application/
      mod.rs                           CREATE
      registry.rs                      CREATE — RegisterUser use-case stub
      search.rs                        CREATE — FindUsers use-case stub
    ports/
      mod.rs                           CREATE
      user_registry.rs                 CREATE — trait UserRegistry
      user_search.rs                   CREATE — trait UserSearch + UserField enum
      user_finder.rs                   CREATE — trait UserFinder
      user_repository.rs               CREATE — composed alias UserRepository
      password_hasher.rs               CREATE — trait PasswordHasher
    adapters/
      mod.rs                           CREATE
      in_memory_user_repository.rs     CREATE — InMemoryUserRepository stub
      plain_password_hasher.rs         CREATE — PlainPasswordHasher stub (todo!)
```

---

## Task 1: Add `AppError` and `ErrorDetail` to `shared-kernel`

**Files:**
- Create: `backend/crates/shared-kernel/src/errors.rs`
- Modify: `backend/crates/shared-kernel/src/lib.rs`
- Modify: `backend/crates/shared-kernel/src/prelude.rs`

- [ ] **Step 1: Create `errors.rs`**

```rust
// backend/crates/shared-kernel/src/errors.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code:    String,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub field: String,
    pub issue: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code:    code.into(),
            message: message.into(),
            details: vec![],
        }
    }

    pub fn with_detail(mut self, field: impl Into<String>, issue: impl Into<String>) -> Self {
        self.details.push(ErrorDetail { field: field.into(), issue: issue.into() });
        self
    }
}
```

- [ ] **Step 2: Expose `errors` module in `lib.rs`**

Current `backend/crates/shared-kernel/src/lib.rs`:
```rust
pub mod prelude;
```

Add:
```rust
pub mod errors;
pub mod prelude;
```

- [ ] **Step 3: Re-export from `prelude.rs`**

Add to `backend/crates/shared-kernel/src/prelude.rs`:
```rust
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
pub use crate::errors::{AppError, ErrorDetail};
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build -p shared-kernel
```
Expected: `Finished` with no errors.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/shared-kernel/src/errors.rs \
        backend/crates/shared-kernel/src/lib.rs \
        backend/crates/shared-kernel/src/prelude.rs
git commit -m "feat(shared-kernel): add AppError and ErrorDetail types"
```

---

## Task 2: Add `Criteria<F>` to `shared-kernel`

**Files:**
- Create: `backend/crates/shared-kernel/src/criteria.rs`
- Modify: `backend/crates/shared-kernel/src/lib.rs`
- Modify: `backend/crates/shared-kernel/src/prelude.rs`

- [ ] **Step 1: Create `criteria.rs`**

```rust
// backend/crates/shared-kernel/src/criteria.rs

/// A set of filters and optional pagination to apply to a query.
/// All filters are AND-conjoined. OR conditions are not supported.
/// `F` is a context-local field enum (e.g. `UserField`, `TaskField`).
#[derive(Debug, Clone, PartialEq)]
pub struct Criteria<F> {
    pub filters: Vec<Filter<F>>,
    pub limit:   Option<u64>,
    pub offset:  Option<u64>,
}

/// A single filter triple: (field, operator, value).
/// `value` is always `String`; adapters parse it to the native type.
/// Invalid values must return AppError with code `E_INVALID_FILTER_VALUE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter<F> {
    pub field: F,
    pub op:    Op,
    pub value: String,
}

/// Comparison operators. Exact semantics are determined by the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq,
    NotEq,
    Like,   // substring / pattern match — semantics determined by the adapter
    Gt,
    Lt,
    Gte,
    Lte,
}

impl<F> Criteria<F> {
    pub fn new() -> Self {
        Self { filters: vec![], limit: None, offset: None }
    }

    pub fn filter(mut self, f: Filter<F>) -> Self {
        self.filters.push(f);
        self
    }

    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }
}

impl<F> Default for Criteria<F> {
    fn default() -> Self { Self::new() }
}
```

- [ ] **Step 2: Expose `criteria` module in `lib.rs`**

```rust
pub mod criteria;
pub mod errors;
pub mod prelude;
```

- [ ] **Step 3: Re-export from `prelude.rs`**

```rust
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
pub use crate::criteria::{Criteria, Filter, Op};
pub use crate::errors::{AppError, ErrorDetail};
```

- [ ] **Step 4: Write unit tests in `criteria.rs`**

Add at the bottom of `backend/crates/shared-kernel/src/criteria.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestField { Name, Age }

    #[test]
    fn empty_criteria_has_no_filters() {
        let c: Criteria<TestField> = Criteria::new();
        assert!(c.filters.is_empty());
        assert!(c.limit.is_none());
        assert!(c.offset.is_none());
    }

    #[test]
    fn builder_adds_filters_and_pagination() {
        let c = Criteria::new()
            .filter(Filter { field: TestField::Name, op: Op::Eq, value: "alice".into() })
            .limit(10)
            .offset(20);
        assert_eq!(c.filters.len(), 1);
        assert_eq!(c.limit, Some(10));
        assert_eq!(c.offset, Some(20));
    }

    #[test]
    fn multiple_filters_are_all_present() {
        let c = Criteria::new()
            .filter(Filter { field: TestField::Name, op: Op::Eq,  value: "alice".into() })
            .filter(Filter { field: TestField::Age,  op: Op::Gte, value: "18".into() });
        assert_eq!(c.filters.len(), 2);
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p shared-kernel
```
Expected: `3 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/shared-kernel/src/criteria.rs \
        backend/crates/shared-kernel/src/lib.rs \
        backend/crates/shared-kernel/src/prelude.rs
git commit -m "feat(shared-kernel): add Criteria<F>, Filter<F>, and Op types"
```

---

## Task 3: Domain layer — `UserId`, `User`, `Role`

**Files:**
- Modify: `backend/crates/identity-access/src/lib.rs`
- Delete: `backend/crates/identity-access/src/domain.rs` (scaffold leftover)
- Delete: `backend/crates/identity-access/src/application.rs` (scaffold leftover)
- Create: `backend/crates/identity-access/src/user/mod.rs`
- Create: `backend/crates/identity-access/src/user/domain/mod.rs`
- Create: `backend/crates/identity-access/src/user/domain/role.rs`
- Create: `backend/crates/identity-access/src/user/domain/user.rs`

- [ ] **Step 1: Remove scaffold leftovers and replace `lib.rs`**

The existing `domain.rs` and `application.rs` are empty scaffold files that become orphaned
once `lib.rs` switches to `pub mod user`. Delete them:

```bash
rm backend/crates/identity-access/src/domain.rs
rm backend/crates/identity-access/src/application.rs
```

Replace full content of `backend/crates/identity-access/src/lib.rs`:
```rust
pub mod user;
```

- [ ] **Step 2: Create `user/mod.rs` — domain only for now**

Only declare `domain` here. The remaining submodules are added in their respective tasks
to avoid compilation failures before the files exist:

```rust
// backend/crates/identity-access/src/user/mod.rs
pub mod domain;
```

- [ ] **Step 3: Create `user/domain/mod.rs`**

```rust
// backend/crates/identity-access/src/user/domain/mod.rs
pub mod role;
pub mod user;

pub use role::Role;
pub use user::{User, UserId};
```

- [ ] **Step 4: Create `user/domain/role.rs`**

```rust
// backend/crates/identity-access/src/user/domain/role.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Member,
    Viewer,
}

impl Default for Role {
    fn default() -> Self { Self::Member }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin  => write!(f, "admin"),
            Role::Member => write!(f, "member"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}
```

- [ ] **Step 5: Create `user/domain/user.rs`**

```rust
// backend/crates/identity-access/src/user/domain/user.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::Role;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for UserId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:            UserId,
    pub email:         String,
    pub password_hash: String,
    pub role:          Role,
}

impl User {
    pub fn new(email: impl Into<String>, password_hash: impl Into<String>, role: Role) -> Self {
        Self {
            id:            UserId::new(),
            email:         email.into(),
            password_hash: password_hash.into(),
            role,
        }
    }
}
```

- [ ] **Step 6: Write domain unit tests in `user.rs`**

Add at the bottom of `backend/crates/identity-access/src/user/domain/user.rs`:

```rust
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
```

- [ ] **Step 7: Run tests**

```bash
cargo test -p identity-access
```
Expected: `2 passed; 0 failed`.

- [ ] **Step 8: Commit**

```bash
git add backend/crates/identity-access/src/
git commit -m "feat(identity-access): add user domain — UserId, User, Role"
```

---

## Task 4: Port traits — `UserRegistry`, `UserSearch`, `UserFinder`, `UserRepository`, `PasswordHasher`

**Files:**
- Create: `backend/crates/identity-access/src/user/ports/mod.rs`
- Create: `backend/crates/identity-access/src/user/ports/user_registry.rs`
- Create: `backend/crates/identity-access/src/user/ports/user_search.rs`
- Create: `backend/crates/identity-access/src/user/ports/user_finder.rs`
- Create: `backend/crates/identity-access/src/user/ports/user_repository.rs`
- Create: `backend/crates/identity-access/src/user/ports/password_hasher.rs`

- [ ] **Step 1: Create `ports/mod.rs`**

```rust
// backend/crates/identity-access/src/user/ports/mod.rs
pub mod password_hasher;
pub mod user_finder;
pub mod user_registry;
pub mod user_repository;
pub mod user_search;

pub use password_hasher::PasswordHasher;
pub use user_finder::UserFinder;
pub use user_registry::UserRegistry;
pub use user_repository::UserRepository;
pub use user_search::{UserField, UserSearch};
```

- [ ] **Step 2: Create `user_registry.rs`**

```rust
// backend/crates/identity-access/src/user/ports/user_registry.rs
use shared_kernel::prelude::AppError;
use crate::user::domain::{User, UserId};

/// Write port — persists or removes a User aggregate (ADR-012).
pub trait UserRegistry {
    fn save(&self, user: &User) -> Result<(), AppError>;
    fn delete(&self, id: &UserId) -> Result<(), AppError>;
}
```

- [ ] **Step 3: Create `user_search.rs`** (includes `UserField` enum)

```rust
// backend/crates/identity-access/src/user/ports/user_search.rs
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
```

- [ ] **Step 4: Create `user_finder.rs`**

```rust
// backend/crates/identity-access/src/user/ports/user_finder.rs
use shared_kernel::prelude::AppError;
use crate::user::domain::{User, UserId};

/// Read port — single-entity lookup by identity (ADR-012).
pub trait UserFinder {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AppError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
}
```

- [ ] **Step 5: Create `user_repository.rs`** (composed alias)

```rust
// backend/crates/identity-access/src/user/ports/user_repository.rs
use super::{UserFinder, UserRegistry, UserSearch};

/// Composed alias — do not add methods here (ADR-012).
pub trait UserRepository: UserRegistry + UserFinder + UserSearch {}

impl<T> UserRepository for T where T: UserRegistry + UserFinder + UserSearch {}
```

- [ ] **Step 6: Create `password_hasher.rs`**

```rust
// backend/crates/identity-access/src/user/ports/password_hasher.rs
use shared_kernel::prelude::AppError;

/// Domain service port — hashes and verifies passwords (ADR-012, exempt from suffix rule).
pub trait PasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AppError>;
    fn verify(&self, plain: &str, hash: &str) -> Result<bool, AppError>;
}
```

- [ ] **Step 7: Verify compilation**

```bash
cargo build -p identity-access
```
Expected: `Finished` with no errors.

- [ ] **Step 8: Commit**

```bash
git add backend/crates/identity-access/src/user/ports/
git commit -m "feat(identity-access): add port traits — UserRegistry, UserSearch, UserFinder, UserRepository, PasswordHasher"
```

---

## Task 5: Adapters — `InMemoryUserRepository` and `PlainPasswordHasher` stubs

**Files:**
- Create: `backend/crates/identity-access/src/user/adapters/mod.rs`
- Create: `backend/crates/identity-access/src/user/adapters/in_memory_user_repository.rs`
- Create: `backend/crates/identity-access/src/user/adapters/plain_password_hasher.rs`

- [ ] **Step 1: Create `adapters/mod.rs`**

```rust
// backend/crates/identity-access/src/user/adapters/mod.rs
pub mod in_memory_user_repository;
pub mod plain_password_hasher;

pub use in_memory_user_repository::InMemoryUserRepository;
pub use plain_password_hasher::PlainPasswordHasher;
```

- [ ] **Step 2: Create `in_memory_user_repository.rs`**

```rust
// backend/crates/identity-access/src/user/adapters/in_memory_user_repository.rs
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
    pub fn new() -> Self { Self::default() }
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
        Ok(self.store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .get(&id.to_string())
            .cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self.store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?
            .values()
            .find(|u| u.email == email)
            .cloned())
    }
}

impl UserSearch for InMemoryUserRepository {
    fn find(&self, criteria: Criteria<UserField>) -> Result<Vec<User>, AppError> {
        let store = self.store
            .read()
            .map_err(|_| AppError::new("E_LOCK", "store lock poisoned"))?;
        let results = store.values()
            .filter(|u| {
                criteria.filters.iter().all(|f| {
                    let field_val = match &f.field {
                        UserField::Email => u.email.clone(),
                        UserField::Role  => u.role.to_string(),
                    };
                    match &f.op {
                        Op::Eq    => field_val == f.value,
                        Op::NotEq => field_val != f.value,
                        Op::Like  => field_val.contains(f.value.as_str()),
                        Op::Gt    => field_val > f.value,
                        Op::Lt    => field_val < f.value,
                        Op::Gte   => field_val >= f.value,
                        Op::Lte   => field_val <= f.value,
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
```

- [ ] **Step 3: Create `plain_password_hasher.rs`**

```rust
// backend/crates/identity-access/src/user/adapters/plain_password_hasher.rs
use shared_kernel::prelude::AppError;
use crate::user::ports::PasswordHasher;

/// Stub — stores passwords in plain text. NEVER use in production.
/// Replace with Argon2/bcrypt adapter before issue #2 ships.
// TODO(#2): replace with real hashing adapter
pub struct PlainPasswordHasher;

impl PasswordHasher for PlainPasswordHasher {
    fn hash(&self, plain: &str) -> Result<String, AppError> {
        Ok(plain.to_string()) // TODO(#2): use argon2 or bcrypt
    }

    fn verify(&self, plain: &str, hash: &str) -> Result<bool, AppError> {
        Ok(plain == hash) // TODO(#2): use argon2 or bcrypt
    }
}
```

- [ ] **Step 4: Write adapter integration tests**

Add at the bottom of `in_memory_user_repository.rs`:

```rust
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
        repo.save(&make_user("alice@test.com", Role::Admin)).unwrap();
        repo.save(&make_user("bob@test.com",   Role::Member)).unwrap();
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
        let criteria = Criteria::new()
            .filter(Filter { field: UserField::Role, op: Op::Eq, value: "admin".into() });
        let results = repo.find(criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "a@test.com");
    }

    #[test]
    fn search_by_email_like() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("alice@example.com", Role::Member)).unwrap();
        repo.save(&make_user("bob@other.com",     Role::Member)).unwrap();
        let criteria = Criteria::new()
            .filter(Filter { field: UserField::Email, op: Op::Like, value: "example".into() });
        let results = repo.find(criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "alice@example.com");
    }

    #[test]
    fn count_matches_find_length() {
        let repo = InMemoryUserRepository::new();
        repo.save(&make_user("x@test.com", Role::Viewer)).unwrap();
        repo.save(&make_user("y@test.com", Role::Viewer)).unwrap();
        let criteria = Criteria::new()
            .filter(Filter { field: UserField::Role, op: Op::Eq, value: "viewer".into() });
        let count = repo.count(criteria.clone()).unwrap();
        let found = repo.find(criteria).unwrap();
        assert_eq!(count, found.len() as u64);
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p identity-access
```
Expected: `8 passed; 0 failed` (2 domain + 6 adapter).

- [ ] **Step 6: Commit**

```bash
git add backend/crates/identity-access/src/user/adapters/
git commit -m "feat(identity-access): add InMemoryUserRepository and PlainPasswordHasher stubs"
```

---

## Task 6: Application layer — `RegisterUser` and `FindUsers` stubs

**Files:**
- Create: `backend/crates/identity-access/src/user/application/mod.rs`
- Create: `backend/crates/identity-access/src/user/application/registry.rs`
- Create: `backend/crates/identity-access/src/user/application/search.rs`

- [ ] **Step 1: Create `application/mod.rs`**

```rust
// backend/crates/identity-access/src/user/application/mod.rs
pub mod registry;
pub mod search;

pub use registry::RegisterUser;
pub use search::FindUsers;
```

- [ ] **Step 2: Create `application/registry.rs`**

```rust
// backend/crates/identity-access/src/user/application/registry.rs
use shared_kernel::prelude::AppError;
use crate::user::domain::{Role, User};
use crate::user::ports::{PasswordHasher, UserRegistry};

pub struct RegisterUserCommand {
    pub email:    String,
    pub password: String,
    pub role:     Role,
}

/// Use case stub — creates a new User and persists it via UserRegistry.
// TODO(#2): add validation (email format, password strength), duplicate check
pub struct RegisterUser<R: UserRegistry, H: PasswordHasher> {
    pub repository: R,
    pub hasher:     H,
}

impl<R: UserRegistry, H: PasswordHasher> RegisterUser<R, H> {
    pub fn execute(&self, cmd: RegisterUserCommand) -> Result<User, AppError> {
        let hash = self.hasher.hash(&cmd.password)?;
        let user = User::new(cmd.email, hash, cmd.role);
        self.repository.save(&user)?;
        Ok(user)
    }
}
```

- [ ] **Step 3: Create `application/search.rs`**

```rust
// backend/crates/identity-access/src/user/application/search.rs
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
```

- [ ] **Step 4: Write application-layer tests**

Add at the bottom of `backend/crates/identity-access/src/user/application/registry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::adapters::{InMemoryUserRepository, PlainPasswordHasher};

    #[test]
    fn register_user_persists_and_returns_user() {
        use crate::user::ports::UserFinder;

        let repo = InMemoryUserRepository::new();
        let uc = RegisterUser {
            repository: repo.clone(),
            hasher:     PlainPasswordHasher,
        };
        let result = uc.execute(RegisterUserCommand {
            email:    "new@test.com".into(),
            password: "secret".into(),
            role:     Role::Member,
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
```

Add at the bottom of `backend/crates/identity-access/src/user/application/search.rs`:

```rust
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
        repo.save(&User::new("z@test.com", "h", Role::Admin)).unwrap();

        let uc = FindUsers { repository: repo };
        let query = FindUsersQuery {
            criteria: Criteria::new()
                .filter(Filter { field: UserField::Role, op: Op::Eq, value: "admin".into() }),
        };
        let result = uc.execute(query).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].email, "z@test.com");
    }
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test -p identity-access
```
Expected: `10 passed; 0 failed`.

- [ ] **Step 6: Run clippy**

```bash
cargo clippy -p identity-access -- -D warnings
```
Expected: no warnings.

- [ ] **Step 7: Commit**

```bash
git add backend/crates/identity-access/src/user/application/
git commit -m "feat(identity-access): add RegisterUser and FindUsers use-case stubs"
```

---

## Task 7: Final verification and PR

- [ ] **Step 1: Run full workspace build and tests**

```bash
cargo build && cargo test
```
Expected: `Finished` + all tests pass, 0 failures.

- [ ] **Step 2: Run clippy on full workspace**

```bash
cargo clippy -- -D warnings
```
Expected: no warnings.

- [ ] **Step 3: Push branch**

```bash
git push -u origin feat/1-identity-access-scaffold
```

- [ ] **Step 4: Open PR**

```bash
gh pr create \
  --title "feat(identity-access): scaffold user module with domain, ports, adapters, and application stubs" \
  --body "$(cat <<'EOF'
## Summary

- Add \`AppError\` and \`ErrorDetail\` to \`shared-kernel\` (ADR-009)
- Add \`Criteria<F>\`, \`Filter<F>\`, \`Op\` to \`shared-kernel\` (ADR-011)
- Scaffold \`identity-access/src/user/\` vertical slice:
  - Domain: \`UserId\`, \`User\`, \`Role\`
  - Ports: \`UserRegistry\`, \`UserSearch\` + \`UserField\`, \`UserFinder\`, \`UserRepository\`, \`PasswordHasher\`
  - Adapters: \`InMemoryUserRepository\` (tested), \`PlainPasswordHasher\` (stub)
  - Application: \`RegisterUser\`, \`FindUsers\` use-case stubs

## Test Plan
- \`cargo test -p shared-kernel\` — 3 tests (Criteria builder)
- \`cargo test -p identity-access\` — 10 tests (domain + adapter + application)
- \`cargo clippy -- -D warnings\` — no warnings

Closes #1
EOF
)"
```

- [ ] **Step 5: Move issue #1 to "In Review" on the project board**

Open https://github.com/users/kennycallado/projects/2 and drag issue #1 to the "In Review" column.
