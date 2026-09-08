//! Unified API error type.
//!
//! Every error leaving a handler is serialized as `{ "error": <message>, "code": <CODE> }`
//! with the matching HTTP status, so the frontend has one shape to deal with.
//! Service errors convert with `?` through the `From` impls below; ad-hoc errors use
//! the constructors (`ApiError::not_found("…")`, `.with_code("…")`).

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::services::auth::AuthError;
use crate::services::group::GroupError;
use crate::services::home::HomeError;
use crate::services::kodi::KodiError;
use crate::services::permission::PermissionError;
use crate::services::service_access::ServiceAccessError;
use crate::services::session::SessionError;
use crate::services::share::ShareError;
use crate::services::user::UserError;

/// JSON body of every error response
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub code: String,
}

/// An error ready to be turned into an HTTP response
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { status, code: code.into(), message: message.into() }
    }

    /// Override the machine-readable code (default is derived from the status)
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn internal(message: impl ToString) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message.to_string())
    }

    pub fn bad_request(message: impl ToString) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", message.to_string())
    }

    pub fn not_found(message: impl ToString) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message.to_string())
    }

    pub fn forbidden(message: impl ToString) -> Self {
        Self::new(StatusCode::FORBIDDEN, "FORBIDDEN", message.to_string())
    }

    pub fn unauthorized(message: impl ToString) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message.to_string())
    }

    pub fn conflict(message: impl ToString) -> Self {
        Self::new(StatusCode::CONFLICT, "CONFLICT", message.to_string())
    }

    pub fn too_many_requests(message: impl ToString) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", message.to_string())
    }

    pub fn service_unavailable(message: impl ToString) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", message.to_string())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {}", self.status, self.code, self.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!("{}", self);
        }
        (
            self.status,
            Json(ErrorBody { error: self.message, code: self.code }),
        )
            .into_response()
    }
}

// ─── Conversions from infrastructure errors ───────────────────────

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::internal(e)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ApiError::not_found("Not found"),
            other => ApiError::internal(other).with_code("DATABASE_ERROR"),
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => ApiError::not_found(e),
            std::io::ErrorKind::PermissionDenied => ApiError::forbidden(e),
            std::io::ErrorKind::AlreadyExists => ApiError::conflict(e),
            _ => ApiError::internal(e).with_code("IO_ERROR"),
        }
    }
}

// ─── Conversions from service errors ──────────────────────────────

impl From<UserError> for ApiError {
    fn from(err: UserError) -> Self {
        let (status, code) = match &err {
            UserError::NotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            UserError::DuplicateUsername => (StatusCode::CONFLICT, "DUPLICATE_USERNAME"),
            UserError::CannotDeleteSelf => (StatusCode::FORBIDDEN, "CANNOT_DELETE_SELF"),
            UserError::CannotDeleteLastAdmin => (StatusCode::FORBIDDEN, "CANNOT_DELETE_LAST_ADMIN"),
            UserError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            UserError::AuthError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR"),
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<GroupError> for ApiError {
    fn from(err: GroupError) -> Self {
        let (status, code) = match &err {
            GroupError::NotFound => (StatusCode::NOT_FOUND, "GROUP_NOT_FOUND"),
            GroupError::DuplicateName => (StatusCode::CONFLICT, "DUPLICATE_NAME"),
            GroupError::CannotDeleteSystemGroup => (StatusCode::FORBIDDEN, "CANNOT_DELETE_SYSTEM_GROUP"),
            GroupError::NotAMember => (StatusCode::NOT_FOUND, "NOT_A_MEMBER"),
            GroupError::AlreadyMember => (StatusCode::CONFLICT, "ALREADY_MEMBER"),
            GroupError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<ShareError> for ApiError {
    fn from(err: ShareError) -> Self {
        let (status, code) = match &err {
            ShareError::NotFound => (StatusCode::NOT_FOUND, "SHARE_NOT_FOUND"),
            ShareError::DuplicateName => (StatusCode::CONFLICT, "DUPLICATE_NAME"),
            ShareError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            ShareError::SystemError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "SYSTEM_ERROR"),
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<PermissionError> for ApiError {
    fn from(err: PermissionError) -> Self {
        let (status, code) = match &err {
            PermissionError::NotFound => (StatusCode::NOT_FOUND, "PERMISSION_NOT_FOUND"),
            PermissionError::InvalidPermission => (StatusCode::BAD_REQUEST, "INVALID_PERMISSION"),
            PermissionError::AlreadyExists => (StatusCode::CONFLICT, "PERMISSION_EXISTS"),
            PermissionError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        let (status, code) = match &err {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN"),
            AuthError::HashingError(_) | AuthError::TokenGenerationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR")
            }
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<KodiError> for ApiError {
    fn from(err: KodiError) -> Self {
        let (status, code) = match &err {
            KodiError::NotConnected => (StatusCode::SERVICE_UNAVAILABLE, "KODI_NOT_CONNECTED"),
            KodiError::InvalidAction(_) => (StatusCode::BAD_REQUEST, "INVALID_ACTION"),
            KodiError::JsonRpc(_) | KodiError::XmlParse(_) | KodiError::Io(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "KODI_ERROR")
            }
        };
        ApiError::new(status, code, err.to_string())
    }
}

impl From<SessionError> for ApiError {
    fn from(err: SessionError) -> Self {
        ApiError::internal(err).with_code("DATABASE_ERROR")
    }
}

impl From<HomeError> for ApiError {
    fn from(err: HomeError) -> Self {
        match &err {
            HomeError::InvalidUsername => ApiError::bad_request(err).with_code("INVALID_USERNAME"),
            HomeError::Io(_) => ApiError::internal(err).with_code("IO_ERROR"),
        }
    }
}

impl From<ServiceAccessError> for ApiError {
    fn from(err: ServiceAccessError) -> Self {
        match &err {
            ServiceAccessError::InvalidService(_) => ApiError::bad_request(err).with_code("INVALID_SERVICE"),
            ServiceAccessError::DatabaseError(_) => ApiError::internal(err).with_code("DATABASE_ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_errors_map_to_expected_status() {
        assert_eq!(ApiError::from(UserError::NotFound).status, StatusCode::NOT_FOUND);
        assert_eq!(ApiError::from(GroupError::AlreadyMember).status, StatusCode::CONFLICT);
        assert_eq!(ApiError::from(ShareError::SystemError("x".into())).status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::from(PermissionError::InvalidPermission).status, StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::from(AuthError::TokenExpired).code, "TOKEN_EXPIRED");
        assert_eq!(ApiError::from(sqlx::Error::RowNotFound).status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn constructors_and_code_override() {
        let e = ApiError::forbidden("nope").with_code("SYSTEM_DISK");
        assert_eq!(e.status, StatusCode::FORBIDDEN);
        assert_eq!(e.code, "SYSTEM_DISK");
        assert_eq!(e.message, "nope");
        assert_eq!(ApiError::internal(anyhow::anyhow!("boom")).message, "boom");
    }
}
