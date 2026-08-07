//! Axum REST API for Steel Horizons.
//!
//! ## Authoritative references
//!
//! - ADR-0003 — Command/Query API with WebSocket Streaming
//! - ADR-0004 — Game Lifecycle State Machine
//! - TDD 00 — Architecture
//! - TDD 02 — API Protocol

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, Method, Request, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::Value;
use tokio::sync::oneshot;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::actor::ActorMessage;
use crate::command::CommandEnvelope;
use crate::id::{BodyId, ShipId, StationId};
use crate::lifecycle::ServerStatus;
use crate::types::GameLifecycle;

// ─── Shared application state ─────────────────────────────────────────

/// Application state injected into every handler.
#[derive(Clone)]
pub struct AppState {
    /// The actor's mailbox sender.
    pub mailbox_tx: tokio::sync::mpsc::UnboundedSender<ActorMessage>,
    /// Publisher for immutable game snapshots.
    pub snapshot_rx: tokio::sync::watch::Receiver<Option<Arc<crate::state::GameSnapshot>>>,
    /// Publisher for server status.
    pub status_rx: tokio::sync::watch::Receiver<Arc<ServerStatus>>,
    /// Random bearer token for authentication.
    pub token: String,
    /// The content catalog (immutable).
    pub content: Arc<crate::content::ContentCatalog>,
}

// ─── Authentication middleware ───────────────────────────────────────

/// Extract the Bearer token from the Authorization header.
///
/// When `state.token` is empty, authentication is disabled (test mode).
async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    // Empty token → skip authentication (test / dev mode).
    if state.token.is_empty() {
        return next.run(req).await;
    }
    let auth_header = req.headers().get(header::AUTHORIZATION);
    let valid = auth_header
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|token| token == state.token)
        .unwrap_or(false);

    if valid {
        next.run(req).await
    } else {
        let body = serde_json::json!({
            "protocol_version": "v1",
            "error": {
                "code": "Unauthorized",
                "message": "Missing or invalid bearer token",
                "details": {}
            }
        });
        (
            StatusCode::UNAUTHORIZED,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            Json(body),
        )
            .into_response()
    }
}

// ─── Error envelope ───────────────────────────────────────────────────

/// A generic API error envelope.
fn api_error(code: &str, message: &str) -> Json<Value> {
    Json(serde_json::json!({
        "protocol_version": "v1",
        "error": {
            "code": code,
            "message": message,
            "details": {}
        }
    }))
}

// ─── Handler: GET /api/v1/status ──────────────────────────────────────

async fn handle_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.status_rx.borrow().clone();
    Json(serde_json::json!({
        "protocol_version": "v1",
        "server": "ready",
        "game_state": status.game_state,
        "tick": status.tick,
        "latest_event_sequence": status.latest_event_sequence,
        "schema_version": status.schema_version,
        "content_version": status.content_version,
        "loading": status.loading,
    }))
}

// ─── Handler: GET /api/v1/state ───────────────────────────────────────

async fn handle_state(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.snapshot_rx.borrow().clone();
    match snapshot {
        Some(snap) => Json(snap).into_response(),
        None => {
            let status = state.status_rx.borrow();
            let loading_info = status.loading.as_ref().map(|l| {
                serde_json::json!({
                    "operation": l.operation,
                    "stage": l.stage,
                })
            });
            let (code, msg) = if status.game_state == GameLifecycle::Loading {
                (StatusCode::SERVICE_UNAVAILABLE, "Game is loading")
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, "Game not loaded")
            };
            let mut body = serde_json::json!({
                "protocol_version": "v1",
                "error": {
                    "code": "GameUnavailable",
                    "message": msg,
                    "details": {}
                }
            });
            if let Some(info) = loading_info {
                body["error"]["details"]["loading"] = info;
            }
            (code, Json(body)).into_response()
        }
    }
}

// ─── Handler: GET /api/v1/content ─────────────────────────────────────

async fn handle_content(State(state): State<AppState>) -> impl IntoResponse {
    let catalog: Value = serde_json::to_value(&*state.content).unwrap_or(Value::Null);
    Json(serde_json::json!({
        "protocol_version": "v1",
        "content": catalog,
    }))
}

// ─── Handler: GET /api/v1/state/{collection} ──────────────────────────

/// Collection queries: ships, stations, celestial_bodies, research, build-orders.
async fn handle_collection(
    State(state): State<AppState>,
    Path(collection): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.snapshot_rx.borrow().clone();
    match snapshot {
        Some(snap) => {
            let state_ref = &snap.state;
            let items: Value = match collection.as_str() {
                "ships" => serde_json::to_value(&state_ref.ships).unwrap_or(Value::Null),
                "stations" => serde_json::to_value(&state_ref.stations).unwrap_or(Value::Null),
                "celestial_bodies" | "bodies" => {
                    serde_json::to_value(&state_ref.celestial_bodies).unwrap_or(Value::Null)
                }
                "research" => {
                    serde_json::to_value(&state_ref.research_projects).unwrap_or(Value::Null)
                }
                "build-orders" | "build_orders" => {
                    serde_json::to_value(&state_ref.build_orders).unwrap_or(Value::Null)
                }
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        api_error(
                            "UnknownCollection",
                            format!("Unknown collection: {}", collection).as_str(),
                        ),
                    )
                        .into_response();
                }
            };
            Json(serde_json::json!({
                "protocol_version": "v1",
                "collection": collection,
                "items": items,
            }))
            .into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            api_error("GameUnavailable", "Game not loaded"),
        )
            .into_response(),
    }
}

// ─── Handler: GET /api/v1/state/{collection}/{id} ─────────────────────

async fn handle_collection_item(
    State(state): State<AppState>,
    Path((collection, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let snapshot = state.snapshot_rx.borrow().clone();
    match snapshot {
        Some(snap) => {
            let state_ref = &snap.state;
            let item: Option<Value> = match collection.as_str() {
                "ships" => {
                    let sid = ShipId(id.clone());
                    state_ref
                        .ships
                        .get(&sid)
                        .map(|v| serde_json::to_value(v).unwrap())
                }
                "stations" => {
                    let sid = StationId(id.clone());
                    state_ref
                        .stations
                        .get(&sid)
                        .map(|v| serde_json::to_value(v).unwrap())
                }
                "celestial_bodies" | "bodies" => {
                    let bid = BodyId(id.clone());
                    state_ref
                        .celestial_bodies
                        .get(&bid)
                        .map(|v| serde_json::to_value(v).unwrap())
                }
                _ => None,
            };
            match item {
                Some(val) => Json(serde_json::json!({
                    "protocol_version": "v1",
                    "collection": collection,
                    "id": id,
                    "item": val,
                }))
                .into_response(),
                None => (
                    StatusCode::NOT_FOUND,
                    api_error(
                        "NotFound",
                        format!("{} '{}' not found", collection, id).as_str(),
                    ),
                )
                    .into_response(),
            }
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            api_error("GameUnavailable", "Game not loaded"),
        )
            .into_response(),
    }
}

// ─── Handler: POST /api/v1/command ───────────────────────────────────

async fn handle_command(
    State(state): State<AppState>,
    Json(envelope): Json<CommandEnvelope>,
) -> impl IntoResponse {
    let response_tx = oneshot::channel();
    state
        .mailbox_tx
        .send(ActorMessage::SubmitCommand {
            envelope,
            response_tx: response_tx.0,
        })
        .unwrap();

    let ack = response_tx.1.await.unwrap();

    let status_code = match ack.status {
        crate::command::CommandStatus::Accepted => StatusCode::ACCEPTED,
        crate::command::CommandStatus::Applied => StatusCode::OK,
        crate::command::CommandStatus::Rejected => StatusCode::CONFLICT,
        crate::command::CommandStatus::Failed => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status_code, Json(ack)).into_response()
}

// ─── Router construction ──────────────────────────────────────────────

/// Error response for unknown routes.
async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        api_error("NotFound", "Resource not found"),
    )
        .into_response()
}

/// Build the API router with all endpoints and middleware.
///
/// The auth layer is only applied when `token` is non-empty.  For tests
/// that want to skip auth, set `token` to an empty string — the middleware
/// will short-circuit to allow all requests.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts| {
                origin.as_bytes().is_empty()
                    || origin
                        .to_str()
                        .ok()
                        .map(|s| {
                            s.contains("127.0.0.1")
                                || s.contains("localhost")
                                || s.contains("::1")
                                || s == "null"
                        })
                        .unwrap_or(false)
            },
        ))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Router::new()
        .route("/api/v1/status", get(handle_status))
        .route("/api/v1/state", get(handle_state))
        .route("/api/v1/content", get(handle_content))
        .route("/api/v1/state/{collection}", get(handle_collection))
        .route(
            "/api/v1/state/{collection}/{id}",
            get(handle_collection_item),
        )
        .route("/api/v1/command", post(handle_command))
        .fallback(handle_404)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{ContentCatalog, DefinitionsCatalog, StartingScenario};
    use std::fs;

    /// Load the real content files for testing.
    fn load_test_content() -> ContentCatalog {
        // The test binary runs from the engine/ directory when cargo test is
        // invoked from the repo root.  Find content/ relative to that.
        let content_dir = {
            let mut cwd = std::env::current_dir().unwrap();
            loop {
                let engine_dir = cwd.join("engine");
                let content_dir = cwd.join("content");
                if engine_dir.is_dir() && content_dir.is_dir() {
                    break content_dir;
                }
                if !cwd.parent().is_some_and(|p| p.join("engine").is_dir()) {
                    panic!(
                        "Cannot find content directory from {:?}",
                        std::env::current_dir().unwrap()
                    );
                }
                cwd = cwd.parent().unwrap().to_path_buf();
            }
        };

        let definitions_path = content_dir.join("definitions.v1.json");
        let starting_system_path = content_dir.join("starting_system.v1.json");

        let definitions: DefinitionsCatalog =
            serde_json::from_str(&fs::read_to_string(&definitions_path).unwrap()).unwrap();
        let starting_system: StartingScenario =
            serde_json::from_str(&fs::read_to_string(&starting_system_path).unwrap()).unwrap();
        ContentCatalog {
            definitions,
            starting_system,
        }
    }

    /// Build an `AppState` with a real actor on the real content catalog.
    fn make_test_state() -> AppState {
        let catalog = load_test_content();
        let (actor, mailbox_tx, snapshot_rx, status_rx) =
            crate::actor::SimulationActor::new(catalog);
        AppState {
            mailbox_tx,
            snapshot_rx,
            status_rx,
            token: String::new(), // empty → no auth
            content: actor.content,
        }
    }

    /// Test that GET /api/v1/status returns a valid status response.
    #[tokio::test]
    async fn test_status_endpoint() {
        let state = make_test_state();
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        // Give the server a moment to start.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/api/v1/status", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["protocol_version"], "v1");
        assert_eq!(body["server"], "ready");

        server.abort();
    }

    /// Test that GET /api/v1/state returns GameUnavailable before loading.
    #[tokio::test]
    async fn test_state_unavailable() {
        let state = make_test_state();
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/api/v1/state", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["protocol_version"], "v1");
        assert!(body["error"]["code"]
            .as_str()
            .map(|c| c == "GameUnavailable")
            .unwrap_or(false));

        server.abort();
    }

    /// Test that auth rejects missing / bad credentials.
    #[tokio::test]
    async fn test_auth_required() {
        let state = make_test_state();
        // Set a non-empty token so auth is enforced.
        let mut state2 = state.clone();
        state2.token = "test-secret-token".to_string();
        let router = build_router(state2);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        // Missing auth header.
        let resp = client
            .get(format!("http://{}/api/v1/status", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Bad token.
        let resp = client
            .get(format!("http://{}/api/v1/status", addr))
            .header("Authorization", "Bearer wrong-token")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Correct token.
        let resp = client
            .get(format!("http://{}/api/v1/status", addr))
            .header("Authorization", "Bearer test-secret-token")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        server.abort();
    }

    /// Test GET /api/v1/content returns content catalog.
    #[tokio::test]
    async fn test_content_endpoint() {
        let state = make_test_state();
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/api/v1/content", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["protocol_version"], "v1");
        assert!(body["content"].is_object());

        server.abort();
    }

    /// Test GET /api/v1/state/ships returns error when game not loaded.
    #[tokio::test]
    async fn test_collection_unavailable() {
        let state = make_test_state();
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        // When game is not loaded, all collections return 503.
        let resp = client
            .get(format!("http://{}/api/v1/state/ships", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        // Unknown collection also returns 503 when game isn't loaded.
        let resp = client
            .get(format!("http://{}/api/v1/state/unknown_collection", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        server.abort();
    }

    /// Test POST /api/v1/command — NewGame command is accepted and applied.
    #[tokio::test]
    async fn test_command_newgame() {
        // Create the actor and API state, then spawn BOTH the actor and server.
        let catalog = load_test_content();
        let (mut actor, mailbox_tx, snapshot_rx, status_rx) =
            crate::actor::SimulationActor::new(catalog);
        let state = AppState {
            mailbox_tx,
            snapshot_rx,
            status_rx,
            token: String::new(),
            content: actor.content.clone(),
        };
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let actor_handle = tokio::spawn(async move {
            actor.run().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let envelope = serde_json::json!({
            "id": "test-cmd-001",
            "command": {
                "type": "newGame",
                "scenario_id": "default"
            }
        });
        let resp = client
            .post(format!("http://{}/api/v1/command", addr))
            .header("Content-Type", "application/json")
            .body(envelope.to_string())
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        eprintln!("DEBUG command: status={}, body={:?}", status, body_text);
        // The actor accepts the NewGame command and applies it.
        assert_eq!(
            status,
            StatusCode::OK,
            "expected 200, got {}: {}",
            status,
            body_text
        );
        if let Ok(body) = serde_json::from_str::<Value>(&body_text) {
            assert_eq!(body["status"], "applied");
        }

        server.abort();
        actor_handle.abort();
    }

    /// Test error envelope format on 404 — unknown route.
    /// The fallback handler returns NotFound error envelope.
    #[tokio::test]
    async fn test_404_error_envelope() {
        let state = make_test_state();
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/api/v1/nonexistent", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["protocol_version"], "v1");
        assert_eq!(body["error"]["code"], "NotFound");
        assert!(body["error"]["message"].is_string());

        server.abort();
    }
}
