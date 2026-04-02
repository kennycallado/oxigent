use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory token deny-list keyed on JTI → expiry (unix seconds).
/// Entries are pruned on insert to avoid unbounded growth.
#[derive(Debug, Clone, Default)]
pub struct DenyList {
    inner: Arc<Mutex<HashMap<String, u64>>>,
}

impl DenyList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically revoke a token if it has not already been revoked.
    /// Returns `true` if the token was newly revoked, `false` if it was already revoked.
    /// `exp` is the token's expiry unix timestamp.
    pub fn revoke_if_not_revoked(&self, jti: &str, exp: u64) -> bool {
        let mut map = self.inner.lock().unwrap();
        if map.contains_key(jti) {
            return false;
        }
        self.prune(&mut map);
        map.insert(jti.to_string(), exp);
        true
    }

    /// Returns true if this JTI has been revoked.
    pub fn is_revoked(&self, jti: &str) -> bool {
        let map = self.inner.lock().unwrap();
        map.contains_key(jti)
    }

    /// Remove expired entries (called on every insert).
    fn prune(&self, map: &mut HashMap<String, u64>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        map.retain(|_, exp| *exp > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn far_future() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    #[test]
    fn revoke_if_not_revoked_returns_true_on_first_call() {
        let dl = DenyList::new();
        assert!(dl.revoke_if_not_revoked("jti-abc", far_future()));
        assert!(dl.is_revoked("jti-abc"));
    }

    #[test]
    fn revoke_if_not_revoked_returns_false_on_duplicate() {
        let dl = DenyList::new();
        assert!(dl.revoke_if_not_revoked("jti-abc", far_future()));
        assert!(!dl.revoke_if_not_revoked("jti-abc", far_future()));
    }

    #[test]
    fn unknown_jti_is_not_denied() {
        let dl = DenyList::new();
        assert!(!dl.is_revoked("jti-unknown"));
    }

    #[test]
    fn expired_entries_are_pruned_on_next_insert() {
        let dl = DenyList::new();
        // Insert an already-expired entry directly
        dl.inner.lock().unwrap().insert("old-jti".to_string(), 1);
        // Insert a new entry — prune runs
        dl.revoke_if_not_revoked("new-jti", far_future());
        assert!(!dl.is_revoked("old-jti"), "expired entry should be pruned");
        assert!(dl.is_revoked("new-jti"));
    }
}
