use std::sync::Arc;

use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::webapp::{
    auth::{clear_session_cookie, session_from_headers, set_session_cookie, UserSession},
    html::{render_chat_page, render_login_page},
    policy::handle_chat,
    state::{AppState, SecurityProfile},
};

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    profile: String,
}

#[derive(Debug, Deserialize)]
struct ChatForm {
    message: String,
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatQuery {
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiChatRequest {
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiChatResponse {
    user: String,
    profile: String,
    answer: String,
    tool_calls_attempted: Vec<String>,
    tool_calls_allowed: Vec<String>,
    tool_calls_denied: Vec<String>,
    redactions: Vec<String>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", post(logout))
        .route("/chat", get(chat_page).post(chat_submit))
        .route("/api/chat", post(api_chat))
        .with_state(state)
}

async fn root(headers: HeaderMap) -> impl IntoResponse {
    if session_from_headers(&headers).is_some() {
        Redirect::to("/chat").into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "target": "acme-support-web",
    }))
}

async fn login_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginQuery>,
) -> impl IntoResponse {
    Html(render_login_page(state.users(), query.error.as_deref()))
}

async fn login_submit(State(state): State<Arc<AppState>>, Form(form): Form<LoginForm>) -> Response {
    let Some(user) = state.find_user(&form.username) else {
        return Redirect::to("/login?error=unknown-user").into_response();
    };

    let Ok(profile) = SecurityProfile::from_form_value(&form.profile) else {
        return Redirect::to("/login?error=bad-profile").into_response();
    };

    let mut response = Redirect::to("/chat").into_response();
    if set_session_cookie(
        response.headers_mut(),
        &UserSession {
            username: user.username.clone(),
            profile,
        },
    )
    .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("failed to set session cookie".to_string()),
        )
            .into_response();
    }
    response
}

async fn logout() -> Response {
    let mut response = Redirect::to("/login").into_response();
    clear_session_cookie(response.headers_mut());
    response
}

async fn chat_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ChatQuery>,
) -> Response {
    let Some(session) = session_from_headers(&headers) else {
        return Redirect::to("/login?error=session-required").into_response();
    };

    let Some(user) = state.find_user(&session.username) else {
        return Redirect::to("/login?error=unknown-user").into_response();
    };

    let transcript = state.transcript_for(&session);
    Html(render_chat_page(
        user,
        session.profile,
        &transcript,
        query.error.as_deref(),
    ))
    .into_response()
}

async fn chat_submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<ChatForm>,
) -> Response {
    let Some(session) = session_from_headers(&headers) else {
        return Redirect::to("/login?error=session-required").into_response();
    };

    let Ok(result) = handle_chat(state.as_ref(), &session, &form.message) else {
        return Redirect::to("/chat?error=chat-failed").into_response();
    };

    state.append_turn(&session, &form.message, &result.answer);
    Redirect::to("/chat").into_response()
}

async fn api_chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ApiChatRequest>,
) -> Response {
    let Some(session) = session_from_headers(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "session-required"})),
        )
            .into_response();
    };

    match handle_chat(state.as_ref(), &session, &payload.message) {
        Ok(result) => {
            state.append_turn(&session, &payload.message, &result.answer);
            Json(ApiChatResponse {
                user: session.username,
                profile: result.profile.to_string(),
                answer: result.answer,
                tool_calls_attempted: result.tool_calls_attempted,
                tool_calls_allowed: result.tool_calls_allowed,
                tool_calls_denied: result.tool_calls_denied,
                redactions: result.redactions,
            })
            .into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}
