//! Application configuration loaded from environment variables.

use std::env;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// Server listen address (e.g., "0.0.0.0:8080")
    pub listen_addr: String,

    /// JWT secret key for token signing
    pub jwt_secret: String,

    /// JWT token expiration time in seconds (default: 7 days)
    pub jwt_expires_secs: i64,

    /// Admin username (for initial setup)
    pub admin_username: String,

    /// Admin password (for initial setup)
    pub admin_password: String,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://vanmoi:vanmoi@localhost:5432/vanmoi".to_string()),

            listen_addr: env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),

            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                // Generate a random secret if not provided
                use std::collections::hash_map::RandomState;
                use std::hash::{BuildHasher, Hasher};
                let hasher = RandomState::new().build_hasher();
                format!("vanmoi-secret-{}", hasher.finish())
            }),

            jwt_expires_secs: env::var("JWT_EXPIRES_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7 * 24 * 60 * 60), // 7 days

            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string()),

            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
                // Generate a random password if not provided
                uuid::Uuid::new_v4().to_string()[..8].to_string()
            }),
        }
    }
}
