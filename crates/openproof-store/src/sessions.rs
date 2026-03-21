use anyhow::{Context, Result};
use chrono::Utc;
use openproof_protocol::{CloudPolicy, ProofSessionState, SessionSnapshot, TranscriptEntry};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;

use crate::extract::{
    default_proof_state, extract_cloud_policy, extract_proof_state, extract_transcript,
};
use crate::store::AppStore;

impl AppStore {
    pub fn list_sessions(&self) -> Result<Vec<SessionSnapshot>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, updated_at, workspace_root, workspace_label, transcript_json, cloud_json, proof_json
            FROM sessions
            ORDER BY updated_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let transcript_json: String = row.get(5)?;
            let transcript =
                serde_json::from_str::<Vec<TranscriptEntry>>(&transcript_json).unwrap_or_default();
            let cloud_json: String = row.get(6)?;
            let cloud = serde_json::from_str::<CloudPolicy>(&cloud_json).unwrap_or_default();
            let proof_json: String = row.get(7)?;
            let proof = serde_json::from_str::<ProofSessionState>(&proof_json)
                .unwrap_or_else(|_| default_proof_state());
            Ok(SessionSnapshot {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                workspace_root: row.get(3)?,
                workspace_label: row.get(4)?,
                cloud,
                transcript,
                proof,
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn session_count(&self) -> Result<usize> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
        Ok(count.max(0) as usize)
    }

    pub fn latest_session(&self) -> Result<Option<SessionSnapshot>> {
        Ok(self.list_sessions()?.into_iter().next())
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<SessionSnapshot>> {
        let conn = self.connect()?;
        self.session_by_id(&conn, session_id)
    }

    pub fn append_entry(&self, session_id: &str, entry: &TranscriptEntry) -> Result<()> {
        let conn = self.connect()?;
        let mut session = self
            .session_by_id(&conn, session_id)?
            .with_context(|| format!("missing session {session_id}"))?;
        session.updated_at = entry.created_at.clone();
        session.transcript.push(entry.clone());
        self.upsert_session(&conn, &session)
    }

    pub fn save_session(&self, session: &SessionSnapshot) -> Result<()> {
        let conn = self.connect()?;
        self.upsert_session(&conn, session)
    }

    pub fn import_legacy_sessions(&self) -> Result<openproof_protocol::LegacyImportSummary> {
        let mut summary = openproof_protocol::LegacyImportSummary::default();
        if !self.paths.legacy_sessions_dir.exists() {
            self.ensure_default_session()?;
            return Ok(summary);
        }

        let entries = fs::read_dir(&self.paths.legacy_sessions_dir)
            .with_context(|| format!("reading {}", self.paths.legacy_sessions_dir.display()))?;
        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => {
                    summary.failed += 1;
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                summary.skipped += 1;
                continue;
            }
            match self.import_legacy_session_file(&path) {
                Ok(imported) => {
                    if imported {
                        summary.imported += 1;
                    } else {
                        summary.skipped += 1;
                    }
                }
                Err(_) => summary.failed += 1,
            }
        }
        self.ensure_default_session()?;
        Ok(summary)
    }

    pub(crate) fn ensure_default_session(&self) -> Result<()> {
        let conn = self.connect()?;
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(());
        }
        let now = Utc::now().to_rfc3339();
        let session = SessionSnapshot {
            id: format!("rust_session_{}", Utc::now().timestamp_millis()),
            title: "OpenProof Rust Session".to_string(),
            updated_at: now,
            workspace_root: None,
            workspace_label: Some("openproof".to_string()),
            cloud: CloudPolicy::default(),
            transcript: Vec::new(),
            proof: default_proof_state(),
        };
        self.upsert_session(&conn, &session)
    }

    fn import_legacy_session_file(&self, path: &std::path::Path) -> Result<bool> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("parsing {}", path.display()))?;
        let Some(id) = value.get("id").and_then(Value::as_str).map(str::to_string) else {
            return Ok(false);
        };
        let title = value
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| "Imported OpenProof Session".to_string());
        let updated_at = value
            .get("updatedAt")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let workspace_root = value
            .get("workspace")
            .and_then(|item| item.get("root"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let workspace_label = value
            .get("workspace")
            .and_then(|item| item.get("label"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let transcript = extract_transcript(&value);
        let snapshot = SessionSnapshot {
            id,
            title,
            updated_at,
            workspace_root,
            workspace_label,
            cloud: extract_cloud_policy(&value),
            transcript,
            proof: extract_proof_state(&value),
        };

        let conn = self.connect()?;
        self.upsert_session(&conn, &snapshot)?;
        Ok(true)
    }

    pub(crate) fn session_by_id(
        &self,
        conn: &Connection,
        session_id: &str,
    ) -> Result<Option<SessionSnapshot>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, updated_at, workspace_root, workspace_label, transcript_json, cloud_json, proof_json
            FROM sessions
            WHERE id = ?
            "#,
        )?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let transcript_json: String = row.get(5)?;
            let transcript =
                serde_json::from_str::<Vec<TranscriptEntry>>(&transcript_json).unwrap_or_default();
            let cloud_json: String = row.get(6)?;
            let cloud = serde_json::from_str::<CloudPolicy>(&cloud_json).unwrap_or_default();
            let proof_json: String = row.get(7)?;
            let proof = serde_json::from_str::<ProofSessionState>(&proof_json)
                .unwrap_or_else(|_| default_proof_state());
            return Ok(Some(SessionSnapshot {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                workspace_root: row.get(3)?,
                workspace_label: row.get(4)?,
                cloud,
                transcript,
                proof,
            }));
        }
        Ok(None)
    }

    pub(crate) fn upsert_session(&self, conn: &Connection, session: &SessionSnapshot) -> Result<()> {
        let transcript_json = serde_json::to_string(&session.transcript)?;
        let cloud_json = serde_json::to_string(&session.cloud)?;
        let proof_json = serde_json::to_string(&session.proof)?;
        conn.execute(
            r#"
            INSERT INTO sessions (id, title, updated_at, workspace_root, workspace_label, transcript_json, cloud_json, proof_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                updated_at = excluded.updated_at,
                workspace_root = excluded.workspace_root,
                workspace_label = excluded.workspace_label,
                transcript_json = excluded.transcript_json,
                cloud_json = excluded.cloud_json,
                proof_json = excluded.proof_json
            "#,
            params![
                session.id,
                session.title,
                session.updated_at,
                session.workspace_root,
                session.workspace_label,
                transcript_json,
                cloud_json,
                proof_json
            ],
        )?;
        Ok(())
    }
}
