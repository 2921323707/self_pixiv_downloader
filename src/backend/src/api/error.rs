use axum::Json;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, ErrorCode};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
}

#[derive(Debug)]
pub struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) app_error: AppError,
}

impl ApiError {
    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            app_error: AppError::new(ErrorCode::TaskNotFound, message),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(app_error: AppError) -> Self {
        let status = match app_error.code {
            ErrorCode::ValidationError
            | ErrorCode::MissingPixivCookie
            | ErrorCode::AiConfigMissing => StatusCode::BAD_REQUEST,
            ErrorCode::PixivAuthFailed => StatusCode::UNAUTHORIZED,
            ErrorCode::PixivForbidden => StatusCode::FORBIDDEN,
            ErrorCode::PixivNotFound | ErrorCode::TaskNotFound => StatusCode::NOT_FOUND,
            ErrorCode::PixivRateLimited => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::PixivNetworkError | ErrorCode::PixivParseError => StatusCode::BAD_GATEWAY,
            ErrorCode::FilesystemWriteFailed
            | ErrorCode::FilesystemPathCollision
            | ErrorCode::SqliteError
            | ErrorCode::AiParseFailed
            | ErrorCode::TaskCancelled
            | ErrorCode::R18PolicySkipped
            | ErrorCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self { status, app_error }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let body = ApiErrorEnvelope {
            error: ApiErrorBody {
                code: self.app_error.code.as_str().to_owned(),
                message: self.app_error.message,
                details: serde_json::json!({}),
            },
        };
        (self.status, Json(body)).into_response()
    }
}
