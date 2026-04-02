use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use shared_kernel::prelude::AppError;

use crate::error::ApiError;
use crate::state::AppState;

/// Extracts and validates the Bearer token from Authorization header.
/// Injects validated `Claims` as a request extension.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::new("E_UNAUTHORIZED", "missing authorization header"))?;

    let claims = state.jwt.validate(token)?;

    if state.deny_list.is_revoked(&claims.jti) {
        return Err(AppError::new("E_UNAUTHORIZED", "token has been revoked").into());
    }

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
