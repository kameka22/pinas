use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::api::cookies;
use crate::services::auth::{extract_bearer_token, validate_jwt, AuthError, Claims};
use crate::AppState;

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub is_admin: bool,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            username: claims.username,
            is_admin: claims.is_admin,
        }
    }
}

/// Error response for authentication failures
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub code: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN"),
            AuthError::HashingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR"),
            AuthError::TokenGenerationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR")
            }
        };

        let body = AuthErrorResponse {
            error: self.to_string(),
            code: code.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

/// Extractor for authenticated users
/// Extracts and validates the JWT from the Authorization header
#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Try cookie first, then Authorization header fallback
        let token: String = {
            // 1. Try pinas_session cookie
            if let Some(cookie_token) = parts
                .headers
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(cookies::extract_token_from_cookies)
            {
                cookie_token.to_string()
            }
            // 2. Fallback: Authorization: Bearer <token>
            else if let Some(bearer_token) = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(extract_bearer_token)
            {
                bearer_token.to_string()
            }
            // 3. No auth found
            else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        error: "Missing authentication".to_string(),
                        code: "MISSING_AUTH".to_string(),
                    }),
                )
                    .into_response());
            }
        };

        // Validate JWT
        let claims = validate_jwt(&token, &state.config).map_err(|e| e.into_response())?;

        Ok(AuthUser::from(claims))
    }
}

/// Extractor for admin users only
/// Wraps AuthUser and requires is_admin = true
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl std::ops::Deref for AdminUser {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_admin {
            return Err((
                StatusCode::FORBIDDEN,
                Json(AuthErrorResponse {
                    error: "Admin access required".to_string(),
                    code: "FORBIDDEN".to_string(),
                }),
            )
                .into_response());
        }

        Ok(AdminUser(user))
    }
}

/// Optional authenticated user extractor
/// Returns None if no valid auth header is present
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
