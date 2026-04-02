use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared_kernel::prelude::AppError;

/// Newtype wrapper over `axum::Json` that maps `JsonRejection` to `ApiError`
/// so malformed or missing JSON bodies return the standard AppError contract.
pub struct AppJson<T>(pub T);

impl<T, S> FromRequest<S> for AppJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => {
                let err = match rejection {
                    JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                        AppError::new("E_VALIDATION", "invalid JSON body")
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        AppError::new("E_VALIDATION", "Content-Type must be application/json")
                    }
                    _ => AppError::new("E_VALIDATION", "failed to parse request body"),
                };
                Err(ApiError(err))
            }
        }
    }
}

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
