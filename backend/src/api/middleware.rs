use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{header, header::AUTHORIZATION, request::Parts, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::api::cookies;
use crate::services::auth::{extract_bearer_token, validate_jwt, AuthError, Claims};
use crate::services::session::is_session_valid;
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

fn auth_error(status: StatusCode, error: &str, code: &str) -> Response {
    (
        status,
        Json(AuthErrorResponse {
            error: error.to_string(),
            code: code.to_string(),
        }),
    )
        .into_response()
}

/// Extract the raw token from a request: `pinas_session` cookie first,
/// then `Authorization: Bearer <token>` as a fallback.
pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(cookies::extract_token_from_cookies)
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(extract_bearer_token)
                .map(|s| s.to_string())
        })
}

/// Validate a token end-to-end: JWT signature/expiry, then the session must
/// still exist in the database (logout / user deletion revoke it).
pub async fn authenticate(token: &str, state: &AppState) -> Result<AuthUser, Response> {
    let claims = validate_jwt(token, &state.config).map_err(|e| e.into_response())?;

    match is_session_valid(&state.db, token).await {
        Ok(true) => Ok(AuthUser::from(claims)),
        Ok(false) => Err(auth_error(
            StatusCode::UNAUTHORIZED,
            "Session has been revoked",
            "SESSION_REVOKED",
        )),
        Err(e) => {
            tracing::error!("Session lookup failed: {}", e);
            Err(auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication backend error",
                "AUTH_ERROR",
            ))
        }
    }
}

/// Router-level middleware: every route behind it requires a valid, non-revoked
/// session. The resolved `AuthUser` is stored in the request extensions so the
/// `AuthUser` / `AdminUser` extractors don't hit the database a second time.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(token) = extract_token(req.headers()) else {
        return auth_error(StatusCode::UNAUTHORIZED, "Missing authentication", "MISSING_AUTH");
    };

    match authenticate(&token, &state).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(response) => response,
    }
}

/// Router-level middleware: requires `require_auth` to have run first and the
/// user to be an administrator. Used for whole routers that are admin-only
/// (storage, docker, packages, updates, printers, terminal, display).
pub async fn require_admin(req: Request, next: Next) -> Response {
    match req.extensions().get::<AuthUser>() {
        Some(user) if user.is_admin => next.run(req).await,
        Some(_) => auth_error(StatusCode::FORBIDDEN, "Admin access required", "FORBIDDEN"),
        None => auth_error(StatusCode::UNAUTHORIZED, "Missing authentication", "MISSING_AUTH"),
    }
}

/// Extractor for authenticated users.
/// Reuses the user resolved by `require_auth` when present; otherwise (public
/// routers such as /api/auth) performs the full validation itself.
#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(user.clone());
        }

        let token = extract_token(&parts.headers).ok_or_else(|| {
            auth_error(StatusCode::UNAUTHORIZED, "Missing authentication", "MISSING_AUTH")
        })?;

        authenticate(&token, state).await
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
            return Err(auth_error(StatusCode::FORBIDDEN, "Admin access required", "FORBIDDEN"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn token_comes_from_cookie_first_then_bearer() {
        let mut headers = HeaderMap::new();
        assert_eq!(extract_token(&headers), None);

        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer bearer-token"));
        assert_eq!(extract_token(&headers).as_deref(), Some("bearer-token"));

        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; pinas_session=cookie-token; other=1"),
        );
        assert_eq!(extract_token(&headers).as_deref(), Some("cookie-token"));
    }
}
