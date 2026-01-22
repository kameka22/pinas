use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::AppConfig;
use crate::models::user::User;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// Admin status
    pub is_admin: bool,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at (Unix timestamp)
    pub iat: usize,
}

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Password hashing failed: {0}")]
    HashingError(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),
}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashingError(e.to_string()))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AuthError::HashingError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT token for a user
pub fn generate_jwt(user: &User, config: &AppConfig) -> Result<String, AuthError> {
    let now = Utc::now();
    let expiration = now + Duration::hours(config.jwt_expiration_hours as i64);

    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        is_admin: user.is_admin,
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
}

/// Validate a JWT token and return the claims
pub fn validate_jwt(token: &str, config: &AppConfig) -> Result<Claims, AuthError> {
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::InvalidToken,
    })
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(header: &str) -> Option<&str> {
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Hashing should succeed");

        assert!(hash.starts_with("$argon2"));
        assert!(verify_password(password, &hash).expect("Verification should succeed"));
        assert!(!verify_password("wrong_password", &hash).expect("Verification should succeed"));
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let config = AppConfig {
            bind_address: "0.0.0.0:3000".to_string(),
            database_url: "sqlite::memory:".to_string(),
            jwt_secret: "test-secret-key".to_string(),
            jwt_expiration_hours: 24,
            files_root: "./data/files".to_string(),
            static_dir: None,
            dev_mode: false,
        };

        let user = User::new(
            "testuser".to_string(),
            "hash".to_string(),
            Some("test@example.com".to_string()),
            true,
        );

        let token = generate_jwt(&user, &config).expect("Token generation should succeed");
        let claims = validate_jwt(&token, &config).expect("Token validation should succeed");

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, "testuser");
        assert!(claims.is_admin);
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_bearer_token("bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
        assert_eq!(extract_bearer_token("abc123"), None);
    }
}
