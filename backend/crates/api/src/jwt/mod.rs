pub mod claims;

use identity_access::user::domain::User;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use shared_kernel::prelude::AppError;
use uuid::Uuid;

pub use claims::Claims;

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_secs: u64,
}

impl JwtService {
    pub fn new(secret: &str, expiry_secs: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_secs,
        }
    }

    pub fn issue(&self, user: &User) -> Result<String, AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| AppError::new("E_INTERNAL", "system clock error"))?
            .as_secs();

        let role = match user.role {
            identity_access::user::domain::Role::Admin => "Admin",
            identity_access::user::domain::Role::Member => "Member",
            identity_access::user::domain::Role::Viewer => "Viewer",
        };

        let claims = Claims {
            sub: user.id.to_string(),
            role: role.to_string(),
            jti: Uuid::new_v4().to_string(),
            iat: now,
            exp: now + self.expiry_secs,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::new("E_INTERNAL", format!("jwt encode: {e}")))
    }

    pub fn validate(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|d| d.claims)
            .map_err(|_| AppError::new("E_UNAUTHORIZED", "invalid or expired token"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use identity_access::user::domain::{Role, User};

    fn test_user() -> User {
        User::new("test@example.com", "hash", Role::Member)
    }

    #[test]
    fn issue_and_validate_round_trip() {
        let svc = JwtService::new("a-secret-key-that-is-at-least-32-bytes!!", 3600);
        let user = test_user();
        let token = svc.issue(&user).unwrap();
        let claims = svc.validate(&token).unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.role, "Member");
    }

    #[test]
    fn validate_invalid_token_returns_unauthorized() {
        let svc = JwtService::new("a-secret-key-that-is-at-least-32-bytes!!", 3600);
        let err = svc.validate("not.a.jwt").unwrap_err();
        assert_eq!(err.code, "E_UNAUTHORIZED");
    }

    #[test]
    fn jti_is_unique_per_token() {
        let svc = JwtService::new("a-secret-key-that-is-at-least-32-bytes!!", 3600);
        let user = test_user();
        let t1 = svc.issue(&user).unwrap();
        let t2 = svc.issue(&user).unwrap();
        let c1 = svc.validate(&t1).unwrap();
        let c2 = svc.validate(&t2).unwrap();
        assert_ne!(c1.jti, c2.jti);
    }
}
