use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{Request, StatusCode},
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    _: State<Arc<AppState>>,
    _: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    unimplemented!()
}

pub async fn logout(
    _: State<Arc<AppState>>,
    _: Request<axum::body::Body>,
) -> Result<StatusCode, ApiError> {
    unimplemented!()
}
