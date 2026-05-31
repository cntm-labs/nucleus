use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Org(#[from] OrgError),
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(transparent)]
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
    #[error("Invalid MFA challenge")]
    MfaInvalidChallenge,
    #[error("MFA not enrolled")]
    MfaNotEnrolled,
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
    #[error("OAuth provider not found: {0}")]
    OAuthProviderNotFound(String),
    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),
    #[error("Magic link has expired")]
    MagicLinkExpired,
    #[error("Invalid redirect URL")]
    InvalidRedirectUrl,
    #[error("OTP has expired")]
    OtpExpired,
    #[error("Too many OTP attempts")]
    OtpMaxAttempts,
    #[error("Passkey challenge required")]
    PasskeyChallenged,
    #[error("Password is too weak")]
    PasswordTooWeak,
}

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account already exists with this email")]
    EmailTaken,
    #[error("The email or password you entered is incorrect")]
    InvalidCredentials,
    #[error("Email address has not been verified")]
    EmailNotVerified,
    #[error("Account not found")]
    NotFound,
    #[error("Invalid verification token")]
    TokenInvalid,
    #[error("Verification token expired")]
    TokenExpired,
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("User already exists with this email")]
    EmailTaken,
    #[error("User already exists with this username")]
    UsernameTaken,
    #[error("Invalid email address")]
    InvalidEmail,
}

#[derive(Debug, Error)]
pub enum OrgError {
    #[error("Organization not found")]
    NotFound,
    #[error("Organization slug is already taken")]
    SlugTaken,
    #[error("Insufficient permissions to perform this action")]
    InsufficientPermissions,
    #[error("Invitation has already been used")]
    InvitationAlreadyUsed,
    #[error("Invitation has expired")]
    InvitationExpired,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid request parameters")]
    ValidationError { details: Vec<ValidationDetail> },
    #[error("Resource not found")]
    NotFound,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Forbidden access")]
    Forbidden,
    #[error("Rate limit exceeded")]
    RateLimited { retry_after_secs: u64 },
    #[error("Plan limit exceeded: {limit_type}")]
    PlanLimitExceeded { limit_type: String },
    #[error("API key has been revoked")]
    KeyRevoked,
    #[error("API key has expired")]
    KeyExpired,
}

#[derive(Debug, Serialize, Clone)]
pub struct ValidationDetail {
    pub field: String,
    pub message: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
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
            Self::MfaInvalidChallenge => "auth/mfa_invalid_challenge",
            Self::MfaNotEnrolled => "auth/mfa_not_enrolled",
            Self::MfaInvalidCode => "auth/mfa_invalid_code",
            Self::SessionExpired => "auth/session_expired",
            Self::SessionRevoked => "auth/session_revoked",
            Self::SessionInvalid => "auth/session_invalid",
            Self::TokenExpired => "auth/token_expired",
            Self::TokenInvalid => "auth/token_invalid",
            Self::TokenRevoked => "auth/token_revoked",
            Self::OAuthStateMismatch => "auth/oauth_state_mismatch",
            Self::OAuthProviderNotFound(_) => "auth/oauth_provider_not_found",
            Self::OAuthProviderError(_) => "auth/oauth_provider_error",
            Self::MagicLinkExpired => "auth/magic_link_expired",
            Self::InvalidRedirectUrl => "auth/invalid_redirect_url",
            Self::OtpExpired => "auth/otp_expired",
            Self::OtpMaxAttempts => "auth/otp_max_attempts",
            Self::PasskeyChallenged => "auth/passkey_challenged",
            Self::PasswordTooWeak => "auth/password_too_weak",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials
            | Self::EmailNotVerified
            | Self::MfaRequired { .. }
            | Self::MfaInvalidChallenge
            | Self::MfaNotEnrolled
            | Self::MfaInvalidCode
            | Self::MagicLinkExpired
            | Self::OtpExpired
            | Self::OtpMaxAttempts
            | Self::PasswordTooWeak => StatusCode::UNAUTHORIZED,
            Self::AccountLocked | Self::AccountBanned => StatusCode::FORBIDDEN,
            Self::SessionExpired | Self::SessionRevoked | Self::SessionInvalid => {
                StatusCode::UNAUTHORIZED
            }
            Self::TokenExpired | Self::TokenInvalid | Self::TokenRevoked => {
                StatusCode::UNAUTHORIZED
            }
            Self::OAuthStateMismatch
            | Self::OAuthProviderNotFound(_)
            | Self::InvalidRedirectUrl => StatusCode::BAD_REQUEST,
            Self::OAuthProviderError(_) => StatusCode::BAD_GATEWAY,
            Self::PasskeyChallenged => StatusCode::UNAUTHORIZED,
        }
    }
}

impl AccountError {
    pub fn code(&self) -> &str {
        match self {
            Self::EmailTaken => "account/email_taken",
            Self::InvalidCredentials => "account/invalid_credentials",
            Self::EmailNotVerified => "account/email_not_verified",
            Self::NotFound => "account/not_found",
            Self::TokenInvalid => "account/token_invalid",
            Self::TokenExpired => "account/token_expired",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::EmailTaken => StatusCode::CONFLICT,
            Self::InvalidCredentials | Self::EmailNotVerified => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::TokenInvalid | Self::TokenExpired => StatusCode::UNAUTHORIZED,
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
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::EmailTaken | Self::UsernameTaken => StatusCode::CONFLICT,
            Self::InvalidEmail => StatusCode::BAD_REQUEST,
        }
    }
}

impl OrgError {
    pub fn code(&self) -> &str {
        match self {
            Self::NotFound => "org/not_found",
            Self::SlugTaken => "org/slug_taken",
            Self::InsufficientPermissions => "org/insufficient_permissions",
            Self::InvitationAlreadyUsed => "org/invitation_already_used",
            Self::InvitationExpired => "org/invitation_expired",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::SlugTaken => StatusCode::CONFLICT,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::InvitationAlreadyUsed => StatusCode::GONE,
            Self::InvitationExpired => StatusCode::GONE,
        }
    }
}

impl ApiError {
    pub fn code(&self) -> &str {
        match self {
            Self::ValidationError { .. } => "api/validation_error",
            Self::NotFound => "api/not_found",
            Self::Unauthorized => "api/unauthorized",
            Self::Forbidden => "api/forbidden",
            Self::RateLimited { .. } => "api/rate_limited",
            Self::PlanLimitExceeded { .. } => "api/plan_limit_exceeded",
            Self::KeyRevoked => "api/api_key_revoked",
            Self::KeyExpired => "api/api_key_expired",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::PlanLimitExceeded { .. } => StatusCode::PAYMENT_REQUIRED,
            Self::KeyRevoked | Self::KeyExpired => StatusCode::UNAUTHORIZED,
        }
    }
}

impl AppError {
    pub fn code(&self) -> &str {
        match self {
            Self::Account(e) => e.code(),
            Self::Auth(e) => e.code(),
            Self::User(e) => e.code(),
            Self::Org(e) => e.code(),
            Self::Api(e) => e.code(),
            Self::Internal(_) => "internal_server_error",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Account(e) => e.status(),
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

        let metadata = match self {
            Self::Auth(AuthError::MfaRequired { mfa_id }) => {
                Some(serde_json::json!({ "mfa_id": mfa_id }))
            }
            Self::Api(ApiError::RateLimited { retry_after_secs }) => {
                Some(serde_json::json!({ "retry_after_secs": retry_after_secs }))
            }
            Self::Api(ApiError::PlanLimitExceeded { limit_type }) => {
                Some(serde_json::json!({ "limit_type": limit_type }))
            }
            _ => None,
        };

        ErrorResponse {
            error: ErrorBody {
                code: code_str.to_string(),
                message: self.to_string(),
                status: self.status().as_u16(),
                request_id: request_id.to_string(),
                details,
                metadata,
                docs_url: format!("https://docs.nucleus.dev/errors/{}", docs_slug),
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = "TODO_EXTRACT_FROM_CONTEXT"; // Real ID provided by middleware re-mapping
        let status = self.status();
        let body = Json(self.to_response(request_id));

        (status, body).into_response()
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
        let err = AppError::Api(ApiError::NotFound);
        assert_eq!(err.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_serializes_to_json() {
        let err = AppError::Api(ApiError::NotFound);
        let response = err.to_response("req_123");
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["error"]["code"], "api/not_found");
        assert_eq!(json["error"]["request_id"], "req_123");
        assert!(json["error"]["docs_url"]
            .as_str()
            .unwrap()
            .contains("api-not_found"));
    }

    #[test]
    fn validation_error_includes_details() {
        let details = vec![ValidationDetail {
            field: "email".to_string(),
            message: "invalid email".to_string(),
        }];
        let err = AppError::Api(ApiError::ValidationError { details });
        let response = err.to_response("req_test");

        assert_eq!(response.error.details.len(), 1);
        assert_eq!(response.error.details[0].field, "email");
    }

    #[test]
    fn mfa_required_includes_mfa_id() {
        let err = AppError::Auth(AuthError::MfaRequired {
            mfa_id: "mfa_123".to_string(),
        });
        let response = err.to_response("req_test");
        assert_eq!(response.error.code, "auth/mfa_required");
    }

    #[test]
    fn rate_limited_includes_retry_after() {
        let err = AppError::Api(ApiError::RateLimited {
            retry_after_secs: 60,
        });
        let response = err.to_response("req_test");
        assert_eq!(response.error.code, "api/rate_limited");
    }

    #[test]
    fn mfa_required_includes_metadata() {
        let err = AppError::Auth(AuthError::MfaRequired {
            mfa_id: "mfa_123".to_string(),
        });
        let response = err.to_response("req_test");
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["error"]["metadata"]["mfa_id"], "mfa_123");
    }

    #[test]
    fn rate_limited_includes_metadata() {
        let err = AppError::Api(ApiError::RateLimited {
            retry_after_secs: 42,
        });
        let response = err.to_response("req_test");
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["error"]["metadata"]["retry_after_secs"], 42);
    }

    #[test]
    fn error_without_metadata_omits_field() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        let response = err.to_response("req_test");
        let json = serde_json::to_value(&response).unwrap();
        assert!(json["error"]["metadata"].is_null());
    }

    #[test]
    fn into_response_sets_correct_status() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AppError::Api(ApiError::NotFound);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let err = AppError::Api(ApiError::RateLimited {
            retry_after_secs: 60,
        });
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
