//! Read-only HTTP handlers for the dashboard.

use axum::{
    extract::{Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use openproof_lean::detect_lean_health;
use openproof_model::load_auth_summary;
use openproof_protocol::{DashboardSessionSummary, DashboardStatusResponse, HealthReport};
use std::{process::Command, sync::Arc};

use crate::tex::generate_tex;
use crate::{DashboardState, APP_JS, GRAPH_JS, INDEX_HTML, SESSIONS_JS, STYLES_CSS};

pub(crate) fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(crate) async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub(crate) async fn app_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
}

pub(crate) async fn styles_css() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], STYLES_CSS)
}

pub(crate) async fn sessions_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        SESSIONS_JS,
    )
}

pub(crate) async fn graph_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        GRAPH_JS,
    )
}

pub(crate) async fn status(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<DashboardStatusResponse>, (StatusCode, String)> {
    let summaries = state
        .store
        .list_session_summaries()
        .map_err(internal_error)?;
    let auth = load_auth_summary().unwrap_or_default();
    let lean = detect_lean_health(&state.lean_project_dir).unwrap_or_default();
    let payload = DashboardStatusResponse {
        local_db_path: state.store.db_path().display().to_string(),
        auth,
        lean,
        session_count: summaries.len(),
        active_session_id: summaries.first().map(|s| s.id.clone()),
        sessions: summaries,
    };
    Ok(Json(payload))
}

pub(crate) async fn health(
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

pub(crate) async fn sessions(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<Vec<DashboardSessionSummary>>, (StatusCode, String)> {
    let summaries = state
        .store
        .list_session_summaries()
        .map_err(internal_error)?;
    Ok(Json(summaries))
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct SessionQuery {
    pub id: Option<String>,
}

pub(crate) async fn session(
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

pub(crate) async fn session_summaries(
    State(state): State<Arc<DashboardState>>,
) -> Result<Json<Vec<DashboardSessionSummary>>, (StatusCode, String)> {
    let summaries = state
        .store
        .list_session_summaries()
        .map_err(internal_error)?;
    Ok(Json(summaries))
}

pub(crate) async fn paper_tex(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match query.id.as_deref() {
        Some(id) => state.store.get_session(id).map_err(internal_error)?,
        None => state.store.latest_session().map_err(internal_error)?,
    };
    let Some(session) = session else {
        return Ok((StatusCode::NOT_FOUND, "No session").into_response());
    };
    // Read Paper.tex directly from workspace (source of truth)
    let ws_dir = state.store.workspace_dir(&session.id);
    let paper_file = std::fs::read_to_string(ws_dir.join("Paper.tex")).unwrap_or_default();
    let tex = if !paper_file.trim().is_empty() {
        paper_file
    } else {
        generate_tex(&session)
    };
    Ok(([(CONTENT_TYPE, "text/plain; charset=utf-8")], tex).into_response())
}

pub(crate) async fn paper_pdf(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match query.id.as_deref() {
        Some(id) => state.store.get_session(id).map_err(internal_error)?,
        None => state.store.latest_session().map_err(internal_error)?,
    };
    let Some(session) = session else {
        return Ok((StatusCode::NOT_FOUND, "No session").into_response());
    };
    let tex = generate_tex(&session);

    // Compile in a temp directory.
    let tmp = std::env::temp_dir().join("openproof-paper");
    let _ = std::fs::create_dir_all(&tmp);
    let tex_path = tmp.join("paper.tex");
    std::fs::write(&tex_path, &tex).map_err(|e| internal_error(e.into()))?;

    let output = Command::new("lualatex")
        .args(["-interaction=nonstopmode", "-halt-on-error", "paper.tex"])
        .current_dir(&tmp)
        .output()
        .map_err(|e| internal_error(anyhow::anyhow!("lualatex failed to start: {e}")))?;

    let pdf_path = tmp.join("paper.pdf");
    if !pdf_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stdout);
        return Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("lualatex failed:\n{stderr}"),
        )
            .into_response());
    }

    let pdf_bytes = std::fs::read(&pdf_path).map_err(|e| internal_error(e.into()))?;
    Ok(([(CONTENT_TYPE, "application/pdf")], pdf_bytes).into_response())
}

pub(crate) async fn workspace_files(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match query.id.as_deref() {
        Some(id) => state.store.get_session(id).map_err(internal_error)?,
        None => state.store.latest_session().map_err(internal_error)?,
    };
    let Some(session) = session else {
        return Ok("[]".to_string().into_response());
    };
    let ws_dir = state.store.workspace_dir(&session.id);
    let mut result = String::from("[");
    let mut first = true;
    if let Ok(entries) = state.store.list_workspace_files(&session.id) {
        for (path, _size) in entries {
            if path.ends_with(".lean") && !path.contains("history/") {
                let content = std::fs::read_to_string(ws_dir.join(&path)).unwrap_or_default();
                if !first {
                    result.push(',');
                }
                first = false;
                let escaped = content
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                result.push_str(&format!(
                    "{{\"path\":\"{path}\",\"content\":\"{escaped}\"}}"
                ));
            }
        }
    }
    result.push(']');
    Ok(([(CONTENT_TYPE, "application/json")], result).into_response())
}
