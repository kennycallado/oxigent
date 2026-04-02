#[derive(Debug, Clone, Default)]
pub struct DenyList;

impl DenyList {
    pub fn new() -> Self {
        Self
    }

    pub fn revoke(&self, _jti: &str, _exp: u64) {}

    pub fn is_revoked(&self, _jti: &str) -> bool {
        false
    }
}
