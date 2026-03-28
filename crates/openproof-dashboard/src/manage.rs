//! Session management endpoints: delete, rename, bulk delete.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::routes::{internal_error, SessionQuery};
use crate::DashboardState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct RenameRequest {
    pub id: String,
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct BulkDeleteRequest {
    pub ids: Vec<String>,
}

pub(crate) async fn delete_session(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(id) = query.id.as_deref() else {
        return Err((StatusCode::BAD_REQUEST, "missing id parameter".to_string()));
    };
    let deleted = state.store.delete_session(id).map_err(internal_error)?;
    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((StatusCode::NOT_FOUND, "session not found".to_string()))
    }
}

pub(crate) async fn rename_session(
    State(state): State<Arc<DashboardState>>,
    Json(body): Json<RenameRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let title = body.title.trim();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title cannot be empty".to_string()));
    }
    let updated = state
        .store
        .rename_session(&body.id, title)
        .map_err(internal_error)?;
    if updated {
        Ok(Json(serde_json::json!({"updated": true})))
    } else {
        Err((StatusCode::NOT_FOUND, "session not found".to_string()))
    }
}

pub(crate) async fn bulk_delete_sessions(
    State(state): State<Arc<DashboardState>>,
    Json(body): Json<BulkDeleteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "ids cannot be empty".to_string()));
    }
    let deleted = state
        .store
        .delete_sessions(&body.ids)
        .map_err(internal_error)?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}
