use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    ValidationError,
    MissingPixivCookie,
    PixivAuthFailed,
    PixivNotFound,
    PixivForbidden,
    PixivRateLimited,
    PixivNetworkError,
    PixivParseError,
    R18PolicySkipped,
    FilesystemWriteFailed,
    FilesystemPathCollision,
    SqliteError,
    AiConfigMissing,
    AiParseFailed,
    TaskCancelled,
    TaskNotFound,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::MissingPixivCookie => "MISSING_PIXIV_COOKIE",
            Self::PixivAuthFailed => "PIXIV_AUTH_FAILED",
            Self::PixivNotFound => "PIXIV_NOT_FOUND",
            Self::PixivForbidden => "PIXIV_FORBIDDEN",
            Self::PixivRateLimited => "PIXIV_RATE_LIMITED",
            Self::PixivNetworkError => "PIXIV_NETWORK_ERROR",
            Self::PixivParseError => "PIXIV_PARSE_ERROR",
            Self::R18PolicySkipped => "R18_POLICY_SKIPPED",
            Self::FilesystemWriteFailed => "FILESYSTEM_WRITE_FAILED",
            Self::FilesystemPathCollision => "FILESYSTEM_PATH_COLLISION",
            Self::SqliteError => "SQLITE_ERROR",
            Self::AiConfigMissing => "AI_CONFIG_MISSING",
            Self::AiParseFailed => "AI_PARSE_FAILED",
            Self::TaskCancelled => "TASK_CANCELLED",
            Self::TaskNotFound => "TASK_NOT_FOUND",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::new(ErrorCode::FilesystemWriteFailed, value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        if value.status().is_some_and(|status| status.as_u16() == 401) {
            return Self::new(ErrorCode::PixivAuthFailed, "Pixiv authentication failed");
        }
        if value.status().is_some_and(|status| status.as_u16() == 403) {
            return Self::new(ErrorCode::PixivForbidden, "Pixiv refused access");
        }
        if value.status().is_some_and(|status| status.as_u16() == 404) {
            return Self::new(ErrorCode::PixivNotFound, "Pixiv resource was not found");
        }
        if value.status().is_some_and(|status| status.as_u16() == 429) {
            return Self::new(
                ErrorCode::PixivRateLimited,
                "Pixiv rate limited the request",
            );
        }
        Self::new(ErrorCode::PixivNetworkError, value.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::new(ErrorCode::SqliteError, value.to_string())
    }
}
