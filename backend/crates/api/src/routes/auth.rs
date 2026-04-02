use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use shared_kernel::prelude::AppError;

use identity_access::user::application::AuthenticateUserCommand;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    AppJson(body): AppJson<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    let email = body.email.ok_or_else(|| {
        AppError::new("E_VALIDATION", "email is required").with_detail("email", "required")
    })?;
    let password = body.password.ok_or_else(|| {
        AppError::new("E_VALIDATION", "password is required")
            .with_detail("password", "required")
    })?;

    let user = state
        .authenticate
        .execute(AuthenticateUserCommand { email, password })?;
    let token = state.jwt.issue(&user)?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<StatusCode, ApiError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::new("E_UNAUTHORIZED", "missing authorization header"))?;

    let claims = state.jwt.validate(token)?;

    if !state.deny_list.revoke_if_not_revoked(&claims.jti, claims.exp) {
        return Err(AppError::new("E_UNAUTHORIZED", "token already revoked").into());
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        build_router, build_router_arc,
        state::AppState,
    };

    const SECRET: &str = "test-secret-key-that-is-32-bytes!!";

    fn test_state() -> AppState {
        use identity_access::user::{
            adapters::{Argon2PasswordHasher, InMemoryUserRepository},
            application::{RegisterUser, RegisterUserCommand},
            domain::Role,
        };

        let repo = InMemoryUserRepository::new();
        let register = RegisterUser {
            repository: repo.clone(),
            hasher: Argon2PasswordHasher,
        };
        register
            .execute(RegisterUserCommand {
                email: "alice@example.com".into(),
                password: "correct-password".into(),
                role: Role::Member,
            })
            .unwrap();

        AppState {
            authenticate: identity_access::user::application::AuthenticateUser {
                finder: repo,
                hasher: Argon2PasswordHasher,
            },
            jwt: crate::jwt::JwtService::new(SECRET, 3600),
            deny_list: crate::deny_list::DenyList::new(),
        }
    }

    async fn body_json(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn login_malformed_json_returns_400_with_api_error_contract() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("not valid json"))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = body_json(res.into_body()).await;
        assert_eq!(body["code"], "E_VALIDATION");
    }

    #[tokio::test]
    async fn login_wrong_content_type_returns_400_with_api_error_contract() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("email=alice"))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = body_json(res.into_body()).await;
        assert_eq!(body["code"], "E_VALIDATION");
    }

    #[tokio::test]
    async fn login_valid_credentials_returns_200_with_token() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"email": "alice@example.com", "password": "correct-password"})
                    .to_string(),
            ))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res.into_body()).await;
        assert!(body["token"].is_string(), "response must contain token");
    }

    #[tokio::test]
    async fn login_wrong_password_returns_401() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"email": "alice@example.com", "password": "wrong"}).to_string(),
            ))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(res.into_body()).await;
        assert_eq!(body["code"], "E_UNAUTHORIZED");
    }

    #[tokio::test]
    async fn login_missing_password_returns_400() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"email": "alice@example.com"}).to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = body_json(res.into_body()).await;
        assert_eq!(body["code"], "E_VALIDATION");
    }

    #[tokio::test]
    async fn logout_valid_token_returns_204_and_revokes() {
        // Single shared state so the deny-list is shared across requests
        let state = Arc::new(test_state());

        // Step 1: login to get a real token
        let login_req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"email": "alice@example.com", "password": "correct-password"}).to_string(),
            ))
            .unwrap();
        let login_res = build_router_arc(state.clone()).oneshot(login_req).await.unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);
        let token = body_json(login_res.into_body()).await["token"]
            .as_str()
            .unwrap()
            .to_string();

        // Step 2: logout — revokes the token in the shared deny-list
        let logout_req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/logout")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let logout_res = build_router_arc(state.clone()).oneshot(logout_req).await.unwrap();
        assert_eq!(logout_res.status(), StatusCode::NO_CONTENT);

        // Step 3: second logout with the same token must be rejected (non-idempotent)
        let second_logout = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/logout")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let second_res = build_router_arc(state.clone()).oneshot(second_logout).await.unwrap();
        assert_eq!(second_res.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(second_res.into_body()).await;
        assert_eq!(body["code"], "E_UNAUTHORIZED");
    }

    #[tokio::test]
    async fn logout_missing_auth_header_returns_401() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method(Method::POST)
            .uri("/v1/auth/logout")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
