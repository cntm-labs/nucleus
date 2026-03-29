use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    User(#[from] UserError),
    #[error("{0}")]
    Org(#[from] OrgError),
    #[error("{0}")]
    Api(#[from] ApiError),
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("The email or password you entered is incorrect")]
    InvalidCredentials,
    #[error("Account is locked due to too many failed attempts")]
    AccountLocked,
    #[error("Account has been banned")]
    AccountBanned,
    #[error("Email address has not been verified")]
    EmailNotVerified,
    #[error("Multi-factor authentication is required")]
    MfaRequired { mfa_id: String },
    #[error("MFA code is invalid")]
    MfaInvalidCode,
    #[error("Session has expired")]
    SessionExpired,
    #[error("Session has been revoked")]
    SessionRevoked,
    #[error("Session token is invalid")]
    SessionInvalid,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Token is invalid")]
    TokenInvalid,
    #[error("Token has been revoked")]
    TokenRevoked,
    #[error("OAuth state mismatch")]
    OAuthStateMismatch,
    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),
    #[error("Passkey challenge failed")]
    PasskeyChallenged,
    #[error("Magic link has expired")]
    MagicLinkExpired,
    #[error("OTP has expired")]
    OtpExpired,
    #[error("OTP maximum attempts exceeded")]
    OtpMaxAttempts,
    #[error("Password does not meet requirements")]
    PasswordTooWeak,
    #[error("Redirect URL is not allowed")]
    InvalidRedirectUrl,
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("Email address is already taken")]
    EmailTaken,
    #[error("Username is already taken")]
    UsernameTaken,
    #[error("Invalid email address")]
    InvalidEmail,
    #[error("Update forbidden")]
    UpdateForbidden,
}

#[derive(Debug, Error)]
pub enum OrgError {
    #[error("Organization not found")]
    NotFound,
    #[error("Organization slug is already taken")]
    SlugTaken,
    #[error("Member already exists in organization")]
    MemberAlreadyExists,
    #[error("Organization has reached its member limit")]
    MemberLimitReached,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Invitation has expired")]
    InvitationExpired,
    #[error("Invitation has already been used")]
    InvitationAlreadyUsed,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("API key has been revoked")]
    KeyRevoked,
    #[error("API key has expired")]
    KeyExpired,
    #[error("Insufficient scopes for this operation")]
    InsufficientScopes,
    #[error("Rate limit exceeded")]
    RateLimited { retry_after_secs: u64 },
    #[error("Project has been suspended")]
    ProjectSuspended,
    #[error("Plan limit exceeded")]
    PlanLimitExceeded { limit_type: String },
    #[error("Validation error")]
    ValidationError { details: Vec<ValidationDetail> },
    #[error("Resource not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationDetail {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub status: u16,
    pub request_id: String,
    pub details: Vec<ValidationDetail>,
    pub docs_url: String,
}

impl AuthError {
    pub fn code(&self) -> &str {
        match self {
            Self::InvalidCredentials => "auth/invalid_credentials",
            Self::AccountLocked => "auth/account_locked",
            Self::AccountBanned => "auth/account_banned",
            Self::EmailNotVerified => "auth/email_not_verified",
            Self::MfaRequired { .. } => "auth/mfa_required",
            Self::MfaInvalidCode => "auth/mfa_invalid_code",
            Self::SessionExpired => "auth/session_expired",
            Self::SessionRevoked => "auth/session_revoked",
            Self::SessionInvalid => "auth/session_invalid",
            Self::TokenExpired => "auth/token_expired",
            Self::TokenInvalid => "auth/token_invalid",
            Self::TokenRevoked => "auth/token_revoked",
            Self::OAuthStateMismatch => "auth/oauth_state_mismatch",
            Self::OAuthProviderError(_) => "auth/oauth_provider_error",
            Self::PasskeyChallenged => "auth/passkey_challenged",
            Self::MagicLinkExpired => "auth/magic_link_expired",
            Self::OtpExpired => "auth/otp_expired",
            Self::OtpMaxAttempts => "auth/otp_max_attempts",
            Self::PasswordTooWeak => "auth/password_too_weak",
            Self::InvalidRedirectUrl => "auth/invalid_redirect_url",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AccountLocked => StatusCode::FORBIDDEN,
            Self::AccountBanned => StatusCode::FORBIDDEN,
            Self::EmailNotVerified => StatusCode::FORBIDDEN,
            Self::MfaRequired { .. } => StatusCode::FORBIDDEN,
            Self::MfaInvalidCode => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::SessionRevoked => StatusCode::UNAUTHORIZED,
            Self::SessionInvalid => StatusCode::UNAUTHORIZED,
            Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::TokenInvalid => StatusCode::UNAUTHORIZED,
            Self::TokenRevoked => StatusCode::UNAUTHORIZED,
            Self::OAuthStateMismatch => StatusCode::BAD_REQUEST,
            Self::OAuthProviderError(_) => StatusCode::BAD_GATEWAY,
            Self::PasskeyChallenged => StatusCode::UNAUTHORIZED,
            Self::MagicLinkExpired => StatusCode::GONE,
            Self::OtpExpired => StatusCode::GONE,
            Self::OtpMaxAttempts => StatusCode::TOO_MANY_REQUESTS,
            Self::PasswordTooWeak => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidRedirectUrl => StatusCode::BAD_REQUEST,
        }
    }
}

impl UserError {
    pub fn code(&self) -> &str {
        match self {
            Self::NotFound => "user/not_found",
            Self::EmailTaken => "user/email_taken",
            Self::UsernameTaken => "user/username_taken",
            Self::InvalidEmail => "user/invalid_email",
            Self::UpdateForbidden => "user/update_forbidden",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::EmailTaken => StatusCode::CONFLICT,
            Self::UsernameTaken => StatusCode::CONFLICT,
            Self::InvalidEmail => StatusCode::UNPROCESSABLE_ENTITY,
            Self::UpdateForbidden => StatusCode::FORBIDDEN,
        }
    }
}

impl OrgError {
    pub fn code(&self) -> &str {
        match self {
            Self::NotFound => "org/not_found",
            Self::SlugTaken => "org/slug_taken",
            Self::MemberAlreadyExists => "org/member_already_exists",
            Self::MemberLimitReached => "org/member_limit_reached",
            Self::InsufficientPermissions => "org/insufficient_permissions",
            Self::InvitationExpired => "org/invitation_expired",
            Self::InvitationAlreadyUsed => "org/invitation_already_used",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::SlugTaken => StatusCode::CONFLICT,
            Self::MemberAlreadyExists => StatusCode::CONFLICT,
            Self::MemberLimitReached => StatusCode::FORBIDDEN,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::InvitationExpired => StatusCode::GONE,
            Self::InvitationAlreadyUsed => StatusCode::CONFLICT,
        }
    }
}

impl ApiError {
    pub fn code(&self) -> &str {
        match self {
            Self::InvalidApiKey => "api/invalid_api_key",
            Self::KeyRevoked => "api/key_revoked",
            Self::KeyExpired => "api/key_expired",
            Self::InsufficientScopes => "api/insufficient_scopes",
            Self::RateLimited { .. } => "api/rate_limited",
            Self::ProjectSuspended => "api/project_suspended",
            Self::PlanLimitExceeded { .. } => "api/plan_limit_exceeded",
            Self::ValidationError { .. } => "api/validation_error",
            Self::NotFound => "api/not_found",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::KeyRevoked => StatusCode::UNAUTHORIZED,
            Self::KeyExpired => StatusCode::UNAUTHORIZED,
            Self::InsufficientScopes => StatusCode::FORBIDDEN,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::ProjectSuspended => StatusCode::FORBIDDEN,
            Self::PlanLimitExceeded { .. } => StatusCode::PAYMENT_REQUIRED,
            Self::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl AppError {
    pub fn code(&self) -> &str {
        match self {
            Self::Auth(e) => e.code(),
            Self::User(e) => e.code(),
            Self::Org(e) => e.code(),
            Self::Api(e) => e.code(),
            Self::Internal(_) => "internal/server_error",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Auth(e) => e.status(),
            Self::User(e) => e.status(),
            Self::Org(e) => e.status(),
            Self::Api(e) => e.status(),
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_response(&self, request_id: &str) -> ErrorResponse {
        let code_str = self.code();
        let docs_slug = code_str.replace('/', "-");

        let details = match self {
            Self::Api(ApiError::ValidationError { details }) => details.clone(),
            _ => vec![],
        };

        ErrorResponse {
            error: ErrorBody {
                code: code_str.to_string(),
                message: self.to_string(),
                status: self.status().as_u16(),
                request_id: request_id.to_string(),
                details,
                docs_url: format!("https://docs.nucleus.dev/errors/{}", docs_slug),
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let response = self.to_response("unknown");
        let body = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"error":{"code":"internal/server_error","message":"Failed to serialize error","status":500,"request_id":"unknown","details":[],"docs_url":"https://docs.nucleus.dev/errors/internal-server-error"}}"#.to_string()
        });

        (status, [("content-type", "application/json")], body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_formats_correctly() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(err.code(), "auth/invalid_credentials");
    }

    #[test]
    fn error_status_codes_correct() {
        assert_eq!(
            AppError::Auth(AuthError::InvalidCredentials).status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::User(UserError::NotFound).status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::Org(OrgError::SlugTaken).status(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::Api(ApiError::RateLimited {
                retry_after_secs: 60
            })
            .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::Internal(anyhow::anyhow!("boom")).status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn error_serializes_to_json() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        let response = err.to_response("req_abc123");
        let json = serde_json::to_value(&response).unwrap();

        let error_obj = &json["error"];
        assert_eq!(error_obj["code"], "auth/invalid_credentials");
        assert_eq!(
            error_obj["message"],
            "The email or password you entered is incorrect"
        );
        assert_eq!(error_obj["status"], 401);
        assert_eq!(error_obj["request_id"], "req_abc123");
        assert!(error_obj["details"].is_array());
        assert_eq!(
            error_obj["docs_url"],
            "https://docs.nucleus.dev/errors/auth-invalid_credentials"
        );
    }

    #[test]
    fn mfa_required_includes_mfa_id() {
        let err = AppError::Auth(AuthError::MfaRequired {
            mfa_id: "mfa_123".to_string(),
        });
        let response = err.to_response("req_test");
        assert_eq!(response.error.code, "auth/mfa_required");
        assert_eq!(response.error.status, 403);
    }

    #[test]
    fn rate_limited_includes_retry_after() {
        let err = AppError::Api(ApiError::RateLimited {
            retry_after_secs: 30,
        });
        let response = err.to_response("req_test");
        assert_eq!(response.error.code, "api/rate_limited");
        assert_eq!(response.error.status, 429);
    }

    #[test]
    fn validation_error_includes_details() {
        let details = vec![ValidationDetail {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
            code: "invalid_format".to_string(),
        }];
        let err = AppError::Api(ApiError::ValidationError {
            details: details.clone(),
        });
        let response = err.to_response("req_test");
        assert_eq!(response.error.details.len(), 1);
        assert_eq!(response.error.details[0].field, "email");
        assert_eq!(response.error.details[0].message, "Invalid email format");
    }

    #[test]
    fn into_response_sets_correct_status() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AppError::User(UserError::NotFound);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let err = AppError::Api(ApiError::RateLimited {
            retry_after_secs: 60,
        });
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
