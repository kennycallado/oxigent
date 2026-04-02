pub struct AppConfig {
    pub jwt_secret: String,
    pub jwt_expiry_secs: u64,
}

impl AppConfig {
    /// Reads from environment variables. Panics at startup if JWT_SECRET is absent or < 32 bytes.
    pub fn from_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env var must be set");
        assert!(
            jwt_secret.len() >= 32,
            "JWT_SECRET must be at least 32 bytes, got {}",
            jwt_secret.len()
        );
        let jwt_expiry_secs = std::env::var("JWT_EXPIRY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86400);

        Self {
            jwt_secret,
            jwt_expiry_secs,
        }
    }
}
