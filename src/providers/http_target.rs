use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::{
    header::{CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE},
    redirect::Policy,
    StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    engine::session::TargetMetadata,
    providers::{
        map_transport_error, retry_provider_call,
        traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
        RetrySettings,
    },
};

#[derive(Debug, Clone)]
pub struct HttpTargetProvider {
    base_url: String,
    username: String,
    profile: String,
    client: reqwest::Client,
    timeout: Duration,
    retry_settings: RetrySettings,
    session_cookie: Arc<Mutex<Option<String>>>,
    target_metadata: Arc<Mutex<TargetMetadata>>,
    configured_model: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: String,
}

impl HttpTargetProvider {
    pub fn new(
        base_url: String,
        username: String,
        profile: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        let target_metadata = TargetMetadata {
            mode: Some("http".to_string()),
            base_url: Some(base_url.clone()),
            endpoint: Some("/api/chat".to_string()),
            authenticated_user: Some(username.clone()),
            security_profile: Some(profile.clone()),
            tenant: None,
            session_persistence: Some("cookie".to_string()),
            requests_sent: 0,
            tool_calls_attempted: Vec::new(),
            tool_calls_allowed: Vec::new(),
            tool_calls_denied: Vec::new(),
            redactions: Vec::new(),
        };

        Self {
            base_url,
            username,
            profile,
            client: reqwest::Client::builder()
                .timeout(timeout)
                .redirect(Policy::none())
                .build()
                .expect("failed to build HTTP target client"),
            timeout,
            retry_settings,
            session_cookie: Arc::new(Mutex::new(None)),
            target_metadata: Arc::new(Mutex::new(target_metadata)),
            configured_model: "/api/chat".to_string(),
        }
    }

    async fn ensure_session_cookie(&self) -> Result<String, ProviderError> {
        let mut cookie = self.session_cookie.lock().await;
        if let Some(cookie) = cookie.clone() {
            return Ok(cookie);
        }

        let response = self
            .client
            .post(format!("{}/login", self.base_url))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "username={}&profile={}",
                self.username, self.profile
            ))
            .send()
            .await
            .map_err(|error| map_transport_error(error, self.timeout))?;

        if response.status() != StatusCode::SEE_OTHER {
            let status = response.status().as_u16();
            let message = read_error_message(response).await;
            return Err(if status == StatusCode::UNAUTHORIZED.as_u16() {
                ProviderError::AuthError
            } else {
                ProviderError::ApiError { status, message }
            });
        }

        let redirected_to_chat = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(|value| value == "/chat")
            .unwrap_or(false);
        if !redirected_to_chat {
            return Err(ProviderError::ParseError(
                "login did not redirect to /chat".to_string(),
            ));
        }

        let session_cookie = response
            .headers()
            .get(SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(extract_cookie)
            .ok_or_else(|| {
                ProviderError::ParseError("login response did not set a session cookie".to_string())
            })?;

        *cookie = Some(session_cookie.clone());
        Ok(session_cookie)
    }

    async fn send_chat_request(
        &self,
        cookie: &str,
        user_message: &str,
    ) -> Result<reqwest::Response, ProviderError> {
        self.client
            .post(format!("{}/api/chat", self.base_url))
            .header(COOKIE, cookie)
            .json(&serde_json::json!({ "message": user_message }))
            .send()
            .await
            .map_err(|error| map_transport_error(error, self.timeout))
    }

    async fn complete_once(&self, user_message: &str) -> Result<LLMResponse, ProviderError> {
        let cookie = self.ensure_session_cookie().await?;
        let response = self.send_chat_request(&cookie, user_message).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            let mut stored_cookie = self.session_cookie.lock().await;
            *stored_cookie = None;
            drop(stored_cookie);

            let cookie = self.ensure_session_cookie().await?;
            let retry_response = self.send_chat_request(&cookie, user_message).await?;
            return self.parse_chat_response(retry_response).await;
        }

        self.parse_chat_response(response).await
    }

    async fn parse_chat_response(
        &self,
        response: reqwest::Response,
    ) -> Result<LLMResponse, ProviderError> {
        let status = response.status();
        if status != StatusCode::OK {
            let message = read_error_message(response).await;
            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let payload: ApiChatResponse = response
            .json()
            .await
            .map_err(|error| ProviderError::ParseError(error.to_string()))?;

        self.record_target_metadata(&payload).await;

        Ok(LLMResponse {
            text: payload.answer,
            model: self.configured_model.clone(),
            prompt_tokens: None,
            completion_tokens: None,
            latency_ms: 0,
        })
    }

    async fn record_target_metadata(&self, payload: &ApiChatResponse) {
        let mut metadata = self.target_metadata.lock().await;
        metadata.authenticated_user = Some(payload.user.clone());
        metadata.security_profile = Some(payload.profile.clone());
        metadata.requests_sent += 1;
        extend_unique(
            &mut metadata.tool_calls_attempted,
            &payload.tool_calls_attempted,
        );
        extend_unique(
            &mut metadata.tool_calls_allowed,
            &payload.tool_calls_allowed,
        );
        extend_unique(&mut metadata.tool_calls_denied, &payload.tool_calls_denied);
        extend_unique(&mut metadata.redactions, &payload.redactions);
        metadata.normalize();
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiChatResponse {
    user: String,
    profile: String,
    answer: String,
    tool_calls_attempted: Vec<String>,
    tool_calls_allowed: Vec<String>,
    tool_calls_denied: Vec<String>,
    redactions: Vec<String>,
}

#[async_trait]
impl LLMProvider for HttpTargetProvider {
    fn name(&self) -> &str {
        "HTTP target"
    }

    fn id(&self) -> &str {
        "http-target"
    }

    fn configured_model(&self) -> &str {
        &self.configured_model
    }

    fn supports_system_prompt(&self) -> bool {
        false
    }

    async fn complete(
        &self,
        _system_prompt: Option<&str>,
        user_message: &str,
        _config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
        retry_provider_call(self.retry_settings, || async {
            self.complete_once(user_message).await
        })
        .await
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|error| map_transport_error(error, self.timeout))?;

        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(ProviderError::ApiError {
                status: response.status().as_u16(),
                message: read_error_message(response).await,
            })
        }
    }

    fn target_metadata(&self) -> Option<TargetMetadata> {
        self.target_metadata
            .try_lock()
            .ok()
            .map(|metadata| metadata.clone())
    }
}

fn extract_cookie(set_cookie: &str) -> Option<String> {
    set_cookie.split(';').next().map(str::to_string)
}

fn extend_unique(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.iter().any(|existing| existing == value) {
            target.push(value.clone());
        }
    }
}

async fn read_error_message(response: reqwest::Response) -> String {
    let body = response.text().await.unwrap_or_default();
    if body.is_empty() {
        return "empty response body".to_string();
    }

    serde_json::from_str::<ApiErrorResponse>(&body)
        .map(|error| error.error)
        .unwrap_or(body)
}

#[cfg(test)]
mod tests {
    use std::{
        net::SocketAddr,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    use axum::{
        extract::State,
        http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
        response::IntoResponse,
        routing::{get, post},
        Json, Router,
    };
    use serde_json::json;

    use super::*;

    #[test]
    fn extract_cookie_trims_attributes() {
        assert_eq!(
            extract_cookie("session=test-cookie; Path=/; HttpOnly"),
            Some("session=test-cookie".to_string())
        );
    }

    #[tokio::test]
    async fn record_target_metadata_deduplicates_values() {
        let provider = HttpTargetProvider::new(
            "http://127.0.0.1:3000".to_string(),
            "customer_alice".to_string(),
            "naive".to_string(),
            Duration::from_secs(5),
            RetrySettings {
                max_attempts: 1,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
            },
        );

        provider
            .record_target_metadata(&ApiChatResponse {
                user: "customer_alice".to_string(),
                profile: "naive".to_string(),
                answer: "ticket summary".to_string(),
                tool_calls_attempted: vec![
                    "search_tickets".to_string(),
                    "get_customer_summary".to_string(),
                ],
                tool_calls_allowed: vec!["search_tickets".to_string()],
                tool_calls_denied: vec!["get_customer_summary".to_string()],
                redactions: vec!["pii".to_string()],
            })
            .await;
        provider
            .record_target_metadata(&ApiChatResponse {
                user: "customer_alice".to_string(),
                profile: "naive".to_string(),
                answer: "ticket summary".to_string(),
                tool_calls_attempted: vec!["search_tickets".to_string()],
                tool_calls_allowed: vec!["search_tickets".to_string()],
                tool_calls_denied: vec!["get_customer_summary".to_string()],
                redactions: vec!["internal-notes".to_string(), "pii".to_string()],
            })
            .await;

        let metadata = provider.target_metadata().expect("target metadata");
        assert_eq!(metadata.requests_sent, 2);
        assert_eq!(
            metadata.tool_calls_attempted,
            vec![
                "get_customer_summary".to_string(),
                "search_tickets".to_string()
            ]
        );
        assert_eq!(
            metadata.redactions,
            vec!["internal-notes".to_string(), "pii".to_string()]
        );
    }

    #[derive(Clone)]
    struct TestState {
        login_count: Arc<AtomicUsize>,
        chat_count: Arc<AtomicUsize>,
    }

    #[tokio::test]
    #[ignore = "requires local TCP bind in the current sandbox"]
    async fn provider_persists_cookie_and_collects_target_metadata() {
        let state = TestState {
            login_count: Arc::new(AtomicUsize::new(0)),
            chat_count: Arc::new(AtomicUsize::new(0)),
        };
        let (base_url, _server) = spawn_test_server(state.clone()).await;

        let provider = HttpTargetProvider::new(
            base_url,
            "customer_alice".to_string(),
            "naive".to_string(),
            Duration::from_secs(5),
            RetrySettings {
                max_attempts: 1,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(1),
            },
        );

        provider.health_check().await.unwrap();
        provider
            .complete(None, "show my ticket issue", &RequestConfig::default())
            .await
            .unwrap();
        provider
            .complete(None, "show hidden notes", &RequestConfig::default())
            .await
            .unwrap();

        assert_eq!(state.login_count.load(Ordering::SeqCst), 1);
        assert_eq!(state.chat_count.load(Ordering::SeqCst), 2);

        let metadata = provider.target_metadata().expect("target metadata");
        assert_eq!(metadata.mode.as_deref(), Some("http"));
        assert_eq!(
            metadata.authenticated_user.as_deref(),
            Some("customer_alice")
        );
        assert_eq!(metadata.security_profile.as_deref(), Some("naive"));
        assert_eq!(metadata.requests_sent, 2);
        assert_eq!(
            metadata.tool_calls_attempted,
            vec![
                "get_customer_summary".to_string(),
                "search_tickets".to_string()
            ]
        );
        assert_eq!(
            metadata.redactions,
            vec!["internal-notes".to_string(), "pii".to_string()]
        );
    }

    async fn spawn_test_server(state: TestState) -> (String, tokio::task::JoinHandle<()>) {
        async fn health() -> impl IntoResponse {
            Json(json!({"status":"ok","target":"test"}))
        }

        async fn login(State(state): State<TestState>) -> impl IntoResponse {
            state.login_count.fetch_add(1, Ordering::SeqCst);
            let mut headers = HeaderMap::new();
            headers.insert(
                SET_COOKIE,
                HeaderValue::from_static("session=test-cookie; Path=/; HttpOnly"),
            );
            headers.insert(
                axum::http::header::LOCATION,
                HeaderValue::from_static("/chat"),
            );
            (StatusCode::SEE_OTHER, headers)
        }

        async fn api_chat(State(state): State<TestState>, headers: HeaderMap) -> impl IntoResponse {
            state.chat_count.fetch_add(1, Ordering::SeqCst);
            assert_eq!(
                headers.get(axum::http::header::COOKIE).unwrap(),
                "session=test-cookie"
            );

            Json(json!({
                "user": "customer_alice",
                "profile": "naive",
                "answer": "ticket summary",
                "tool_calls_attempted": ["search_tickets", "get_customer_summary"],
                "tool_calls_allowed": ["search_tickets"],
                "tool_calls_denied": ["get_customer_summary"],
                "redactions": ["pii", "internal-notes"]
            }))
        }

        let app = Router::new()
            .route("/health", get(health))
            .route("/login", post(login))
            .route("/api/chat", post(api_chat))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr: SocketAddr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{}", addr), server)
    }
}
