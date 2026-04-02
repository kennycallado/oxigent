use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_kernel::prelude::AppError;

#[derive(Debug, Clone)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let err = self.0;
        let status = match err.code.as_str() {
            "E_VALIDATION" => StatusCode::BAD_REQUEST,
            "E_UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "E_FORBIDDEN" => StatusCode::FORBIDDEN,
            "E_NOT_FOUND" => StatusCode::NOT_FOUND,
            "E_DOMAIN" => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = if status == StatusCode::INTERNAL_SERVER_ERROR {
            AppError::new("E_INTERNAL", "an unexpected error occurred")
        } else {
            err
        };

        (status, Json(body)).into_response()
    }
}
