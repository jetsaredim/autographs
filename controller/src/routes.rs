use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{auth::AuthState, config::ControllerConfig};

const SESSION_COOKIE: &str = "autographs_admin_session";

#[derive(Clone)]
pub struct AppState {
    config: ControllerConfig,
    auth: AuthState,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminHealthResponse {
    ok: bool,
    service: &'static str,
    oracle_configured: bool,
    media_configured: bool,
    static_release_configured: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
}

pub fn router(config: ControllerConfig) -> Router {
    let auth = AuthState::new(
        config.admin_password.clone(),
        config.admin_password_hash.clone(),
        config.operator_token.clone(),
    );
    let state = AppState { config, auth };

    Router::new()
        .route("/health", get(health))
        .route("/admin/api/health", get(admin_health))
        .route("/admin/api/login", post(login))
        .route("/admin/api/logout", post(logout))
        .route("/admin/api/protected", get(protected))
        .route("/admin/api/items", post(protected_mutation))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "autographs-controller",
    })
}

async fn admin_health(State(state): State<AppState>) -> Json<AdminHealthResponse> {
    Json(AdminHealthResponse {
        ok: true,
        service: "autographs-controller",
        oracle_configured: state.config.oracle_configured,
        media_configured: state.config.media_configured,
        static_release_configured: state.config.static_release_configured,
    })
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    match state.auth.login(&payload.password) {
        Ok(session) => {
            let mut response = StatusCode::NO_CONTENT.into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&session_cookie(&session, state.config.secure_cookies))
                    .expect("session cookie header"),
            );
            response
        }
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn logout(State(state): State<AppState>, method: Method, headers: HeaderMap) -> Response {
    let auth = match authenticate(&state, &headers) {
        Some(auth) => auth,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if !csrf_allowed(&state, &method, &headers, &auth) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if let AuthKind::Session(session) = auth {
        state.auth.logout(&session);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "autographs_admin_session=; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=0",
        ),
    );
    response
}

async fn protected(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    authenticate(&state, &headers)
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::UNAUTHORIZED)
}

async fn protected_mutation(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> StatusCode {
    let Some(auth) = authenticate(&state, &headers) else {
        return StatusCode::UNAUTHORIZED;
    };

    if csrf_allowed(&state, &method, &headers, &auth) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::FORBIDDEN
    }
}

enum AuthKind {
    Session(String),
    OperatorToken,
}

fn authenticate(state: &AppState, headers: &HeaderMap) -> Option<AuthKind> {
    if let Some(token) = bearer_token(headers)
        && state.auth.has_operator_token(token)
    {
        return Some(AuthKind::OperatorToken);
    }

    let session = cookie_value(headers, SESSION_COOKIE)?;
    state
        .auth
        .has_session(session)
        .then(|| AuthKind::Session(session.to_owned()))
}

fn csrf_allowed(state: &AppState, method: &Method, headers: &HeaderMap, auth: &AuthKind) -> bool {
    if matches!(auth, AuthKind::OperatorToken) || matches!(*method, Method::GET | Method::HEAD) {
        return true;
    }

    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin == state.config.public_origin)
        || headers
            .get(header::REFERER)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|referer| referer.starts_with(&format!("{}/", state.config.public_origin)))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(&format!("{name}=")))
}

fn session_cookie(session: &str, secure: bool) -> String {
    format!(
        "{SESSION_COOKIE}={session}; Path=/admin; HttpOnly; SameSite=Strict{}",
        if secure { "; Secure" } else { "" }
    )
}
