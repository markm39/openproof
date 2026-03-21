use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use openproof_lean::detect_lean_health;
use openproof_model::load_auth_summary;
use openproof_protocol::{
    DashboardSessionSummary, DashboardStatusResponse, HealthReport, MessageRole, SessionSnapshot,
};
use openproof_store::AppStore;
use std::{net::SocketAddr, process::Command, sync::Arc};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_JS: &str = include_str!("../static/app.js");
const STYLES_CSS: &str = include_str!("../static/styles.css");

#[derive(Clone)]
struct DashboardState {
    store: AppStore,
    lean_project_dir: std::path::PathBuf,
}

pub struct DashboardServer {
    pub port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl DashboardServer {
    pub async fn close(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.handle.await;
        Ok(())
    }
}

pub async fn start_dashboard_server(
    store: AppStore,
    lean_project_dir: std::path::PathBuf,
    preferred_port: Option<u16>,
) -> Result<DashboardServer> {
    let state = Arc::new(DashboardState {
        store,
        lean_project_dir,
    });

    let router = Router::new()
        .route("/", get(index))
        .route("/app.js", get(app_js))
        .route("/styles.css", get(styles_css))
        .route("/api/status", get(status))
        .route("/api/health", get(health))
        .route("/api/sessions", get(sessions))
        .route("/api/session", get(session))
        .route("/api/raw-state", get(status))
        .with_state(state);

    let primary_port = preferred_port.unwrap_or(4821);
    let listener = match TcpListener::bind(("127.0.0.1", primary_port)).await {
        Ok(listener) => listener,
        Err(_) => TcpListener::bind(("127.0.0.1", 0)).await?,
    };
    let port = listener.local_addr()?.port();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });
        let _ = server.await;
    });

    Ok(DashboardServer {
        port,
        shutdown_tx: Some(shutdown_tx),
        handle,
    })
}

pub fn open_browser(url: &str) {
    let platform = std::env::consts::OS;
    let mut command = if platform == "macos" {
        let mut cmd = Command::new("open");
        cmd.arg(url);
        cmd
    } else if platform == "windows" {
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", "start", "", url]);
        cmd
    } else {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url);
        cmd
    };
    let _ = command.spawn();
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn app_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
}

async fn styles_css() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], STYLES_CSS)
}

async fn status(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<DashboardStatusResponse>, (StatusCode, String)> {
    let sessions = state.store.list_sessions().map_err(internal_error)?;
    let auth = load_auth_summary().unwrap_or_default();
    let lean = detect_lean_health(&state.lean_project_dir).unwrap_or_default();
    let payload = DashboardStatusResponse {
        local_db_path: state.store.db_path().display().to_string(),
        auth,
        lean,
        session_count: sessions.len(),
        active_session_id: sessions.first().map(|session| session.id.clone()),
        sessions: sessions.iter().map(build_session_summary).collect(),
    };
    Ok(Json(payload))
}

async fn health(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<HealthReport>, (StatusCode, String)> {
    let latest_session = state.store.latest_session().map_err(internal_error)?;
    let auth = load_auth_summary().unwrap_or_default();
    let lean = detect_lean_health(&state.lean_project_dir).unwrap_or_default();
    let payload = HealthReport {
        ok: lean.ok,
        local_db_path: state.store.db_path().display().to_string(),
        session_count: state.store.session_count().map_err(internal_error)?,
        latest_session_id: latest_session.map(|session| session.id),
        auth,
        lean,
    };
    Ok(Json(payload))
}

async fn sessions(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<Vec<DashboardSessionSummary>>, (StatusCode, String)> {
    let sessions = state
        .store
        .list_sessions()
        .map_err(internal_error)?
        .iter()
        .map(build_session_summary)
        .collect::<Vec<_>>();
    Ok(Json(sessions))
}

#[derive(Debug, serde::Deserialize)]
struct SessionQuery {
    id: Option<String>,
}

async fn session(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match query.id.as_deref() {
        Some(id) => state.store.get_session(id).map_err(internal_error)?,
        None => state.store.latest_session().map_err(internal_error)?,
    };
    match session {
        Some(session) => Ok(Json(session).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

fn build_session_summary(session: &SessionSnapshot) -> DashboardSessionSummary {
    let last_entry = session.transcript.last();
    let active_node_label = session
        .proof
        .active_node_id
        .as_deref()
        .and_then(|id| session.proof.nodes.iter().find(|node| node.id == id))
        .map(|node| node.label.clone());
    DashboardSessionSummary {
        id: session.id.clone(),
        title: session.title.clone(),
        updated_at: session.updated_at.clone(),
        workspace_label: session.workspace_label.clone(),
        transcript_entries: session.transcript.len(),
        proof_nodes: session.proof.nodes.len(),
        active_node_label,
        proof_phase: Some(session.proof.phase.clone()),
        last_role: last_entry.map(|entry| match entry.role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::System => "system".to_string(),
            MessageRole::Notice => "notice".to_string(),
        }),
        last_excerpt: last_entry.map(|entry| truncate(&entry.content, 180)),
    }
}

fn truncate(input: &str, limit: usize) -> String {
    let trimmed = input.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    trimmed
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub fn dashboard_url(port: u16) -> String {
    SocketAddr::from(([127, 0, 0, 1], port)).to_string()
}
