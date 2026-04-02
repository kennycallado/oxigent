pub mod config;
pub mod deny_list;
pub mod error;
pub mod jwt;
pub mod middleware;
pub mod routes;
pub mod state;

pub use config::AppConfig;
pub use state::AppState;

use axum::Router;
use std::sync::Arc;

/// Build the router with an owned AppState (wraps it in Arc internally).
pub fn build_router(state: AppState) -> Router {
    build_router_arc(Arc::new(state))
}

/// Build the router with a pre-shared Arc<AppState> (used in tests to share deny-list).
pub fn build_router_arc(state: Arc<AppState>) -> Router {
    use axum::routing::post;

    Router::new()
        .route("/v1/auth/login", post(routes::auth::login))
        .route("/v1/auth/logout", post(routes::auth::logout))
        .with_state(state)
}
