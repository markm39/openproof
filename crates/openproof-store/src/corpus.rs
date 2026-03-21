use anyhow::{Context, Result};
use chrono::Utc;
use openproof_protocol::{
    CorpusSummary, LeanVerificationSummary, ProofNodeKind, SessionSnapshot, ShareMode, SyncSummary,
};
use rusqlite::params;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::store::AppStore;

// ---------------------------------------------------------------------------
// Private utility functions
// ---------------------------------------------------------------------------

pub(crate) fn next_store_id(prefix: &str) -> String {
    format!("{prefix}_{}", Utc::now().timestamp_millis())
}

pub(crate) fn stable_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub(crate) fn sanitize_identity_segment(input: &str) -> String {
    let mut value = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    while value.contains("__") {
        value = value.replace("__", "_");
    }
    value.trim_matches('_').to_string()
}

pub(crate) fn share_mode_to_str(mode: ShareMode) -> &'static str {
    match mode {
        ShareMode::Local => "local",
        ShareMode::Community => "community",
        ShareMode::Private => "private",
    }
}

fn classify_failure(result: &LeanVerificationSummary) -> String {
    let combined = format!(
        "{}\n{}\n{}",
        result.stderr,
        result.stdout,
        result.error.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();
    if combined.contains("unknown constant") || combined.contains("unknown identifier") {
        "unknown-identifier".to_string()
    } else if combined.contains("type mismatch") {
        "type-mismatch".to_string()
    } else if combined.contains("application type mismatch") {
        "application-type-mismatch".to_string()
    } else if combined.contains("unsolved goals") {
        "unsolved-goals".to_string()
    } else if combined.contains("sorry") {
        "sorry-placeholder".to_string()
    } else if combined.contains("timeout") {
        "timeout".to_string()
    } else if let Some(error) = result.error.as_ref().filter(|value| !value.trim().is_empty()) {
        error.trim().to_string()
    } else {
        "lean-error".to_string()
    }
}

pub(crate) fn summarize_lean_diagnostic(result: &LeanVerificationSummary) -> String {
    let primary = if !result.stderr.trim().is_empty() {
        result.stderr.trim()
    } else if !result.stdout.trim().is_empty() {
        result.stdout.trim()
    } else {
        result.error.as_deref().unwrap_or("Lean verification failed.")
    };
    primary.lines().take(12).collect::<Vec<_>>().join("\n")
}

#[derive(Debug, Clone)]
struct CorpusClusterRecord {
    id: String,
    cluster_key: String,
    canonical_item_id: String,
    label: String,
    statement_preview: String,
    member_count: usize,
    created_at: String,
    updated_at: String,
}

fn normalize_statement_for_cluster(statement: &str) -> String {
    statement
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" :", ":")
        .replace(": ", ":")
        .replace(" ,", ",")
        .replace(", ", ",")
        .replace(" (", "(")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" = ", "=")
        .replace(" + ", "+")
        .replace(" - ", "-")
        .replace(" * ", "*")
        .replace(" / ", "/")
        .trim()
        .to_string()
}

fn compute_corpus_cluster_key(
    statement: &str,
    decl_kind: &str,
    is_theorem_like: bool,
    content_hash: &str,
) -> String {
    if !is_theorem_like && !content_hash.trim().is_empty() {
        return stable_hash(&format!("artifact::{decl_kind}::{content_hash}"));
    }
    stable_hash(&format!(
        "{}::{}::{}",
        if is_theorem_like {
            "theorem-like"
        } else {
            "declaration"
        },
        decl_kind,
        normalize_statement_for_cluster(statement)
    ))
}

fn preview_text(value: &str, limit: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    trimmed.chars().take(limit).collect::<String>()
}

// ---------------------------------------------------------------------------
// Library seed ingestion helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct LibrarySeedItem {
    pub kind: String,
    pub decl_name: String,
    pub statement: String,
    pub doc_string: Option<String>,
}

impl LibrarySeedItem {
    pub fn search_text(&self, module_name: &str, package_name: &str) -> String {
        [
            self.decl_name.as_str(),
            self.statement.as_str(),
            module_name,
            package_name,
            self.doc_string.as_deref().unwrap_or(""),
        ]
        .join(" ")
    }
}

pub(crate) fn collect_lean_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).with_context(|| format!("reading {}", path.display()))? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry.file_type()?.is_dir() {
                stack.push(entry_path);
            } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("lean") {
                files.push(entry_path);
            }
        }
    }
    Ok(files)
}

pub(crate) fn lean_module_name(relative_path: &Path) -> String {
    let without_extension = relative_path.with_extension("");
    without_extension
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn extract_library_seed_items(source: &str) -> Vec<LibrarySeedItem> {
    let mut items = Vec::new();
    let lines = source.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut pending_doc: Option<String> = None;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed.starts_with("/--") {
            let mut doc = trimmed.trim_start_matches("/--").trim().to_string();
            while !doc.contains("-/") && index + 1 < lines.len() {
                index += 1;
                doc.push(' ');
                doc.push_str(lines[index].trim());
            }
            pending_doc = Some(doc.replace("-/", "").trim().to_string());
            index += 1;
            continue;
        }
        if let Some((kind, decl_name)) = parse_decl_header(trimmed) {
            let mut header = trimmed.to_string();
            while !header.contains(":=")
                && !header.contains(" where")
                && !header.ends_with("where")
                && index + 1 < lines.len()
            {
                let next = lines[index + 1].trim();
                if next.is_empty()
                    || parse_decl_header(next).is_some()
                    || next.starts_with("/--")
                {
                    break;
                }
                header.push(' ');
                header.push_str(next);
                index += 1;
                if header.contains(":=") || header.contains(" where") || header.ends_with("where") {
                    break;
                }
            }
            let statement = header
                .split(":=")
                .next()
                .unwrap_or(header.as_str())
                .trim()
                .to_string();
            if !decl_name.contains("._") && !decl_name.starts_with('_') {
                items.push(LibrarySeedItem {
                    kind,
                    decl_name,
                    statement,
                    doc_string: pending_doc.take(),
                });
            } else {
                pending_doc = None;
            }
        } else {
            pending_doc = None;
        }
        index += 1;
    }
    items
}

fn parse_decl_header(line: &str) -> Option<(String, String)> {
    let mut tokens = line.split_whitespace();
    let mut first = tokens.next()?;
    while matches!(
        first,
        "private" | "protected" | "noncomputable" | "unsafe" | "partial"
    ) {
        first = tokens.next()?;
    }
    let kind = match first {
        "theorem" | "lemma" | "def" | "instance" | "class" | "structure" | "inductive"
        | "abbrev" => first,
        _ => return None,
    };
    let raw_name = tokens.next()?.trim();
    let decl_name = raw_name
        .trim_matches(|ch: char| matches!(ch, '(' | '{' | '['))
        .trim_end_matches(':')
        .trim_end_matches(":=")
        .trim_end_matches(',')
        .trim()
        .to_string();
    if decl_name.is_empty() {
        None
    } else {
        Some((kind.to_string(), decl_name))
    }
}

// ---------------------------------------------------------------------------
// AppStore impl: corpus operations
// ---------------------------------------------------------------------------

impl AppStore {
    pub fn record_verification_result(
        &self,
        session: &SessionSnapshot,
        result: &LeanVerificationSummary,
    ) -> Result<()> {
        let Some(active_node_id) = session.proof.active_node_id.as_deref() else {
            return Ok(());
        };
        let Some(node) = session
            .proof
            .nodes
            .iter()
            .find(|node| node.id == active_node_id)
        else {
            return Ok(());
        };

        let conn = self.connect()?;
        let tx = conn.unchecked_transaction()?;
        let now = result.checked_at.clone();
        let content_hash = stable_hash(&node.content);
        let artifact_id = format!("artifact_{}", content_hash);
        tx.execute(
            r#"
            INSERT INTO verified_artifacts
            (id, artifact_hash, label, content, imports_json, namespace, metadata_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(artifact_hash) DO UPDATE SET
                label = excluded.label,
                content = excluded.content,
                imports_json = excluded.imports_json,
                namespace = excluded.namespace,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at
            "#,
            params![
                artifact_id,
                content_hash,
                node.label,
                node.content,
                serde_json::to_string(&session.proof.imports)?,
                Option::<String>::None,
                serde_json::to_string(&serde_json::json!({
                    "workspaceRoot": session.workspace_root,
                    "workspaceLabel": session.workspace_label
                }))?,
                now,
                now,
            ],
        )?;

        let verification_run_id = next_store_id("verification");
        tx.execute(
            r#"
            INSERT INTO verification_runs
            (id, session_id, target_kind, target_id, target_label, target_node_id, artifact_id, ok, code, stdout, stderr, error, scratch_path, rendered_scratch, created_at)
            VALUES (?, ?, 'node', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                verification_run_id,
                session.id,
                node.id,
                node.label,
                node.id,
                artifact_id,
                if result.ok { 1 } else { 0 },
                result.code,
                result.stdout,
                result.stderr,
                result.error,
                result.scratch_path,
                result.rendered_scratch,
                now,
            ],
        )?;

        if result.ok {
            let identity_key = format!(
                "user-verified/{}/{}/{}",
                sanitize_identity_segment(session.id.as_str()),
                sanitize_identity_segment(node.label.as_str()),
                stable_hash(node.statement.as_str())
            );
            let visibility = share_mode_to_str(session.cloud.share_mode);
            tx.execute(
                r#"
                INSERT INTO verified_corpus_items
                (id, statement_hash, identity_key, label, statement, content_hash, artifact_id, verification_run_id, visibility, decl_kind, search_text, origin, namespace, imports_json, metadata_json, source_session_id, source_node_id, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'user-verified', ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(identity_key) DO UPDATE SET
                    label = excluded.label,
                    statement = excluded.statement,
                    content_hash = excluded.content_hash,
                    artifact_id = excluded.artifact_id,
                    verification_run_id = excluded.verification_run_id,
                    visibility = excluded.visibility,
                    search_text = excluded.search_text,
                    imports_json = excluded.imports_json,
                    metadata_json = excluded.metadata_json,
                    updated_at = excluded.updated_at
                "#,
                params![
                    next_store_id("corpus"),
                    stable_hash(node.statement.as_str()),
                    identity_key,
                    node.label,
                    node.statement,
                    content_hash,
                    artifact_id,
                    verification_run_id,
                    visibility,
                    match node.kind {
                        ProofNodeKind::Lemma => "lemma",
                        ProofNodeKind::Theorem => "theorem",
                        ProofNodeKind::Artifact => "artifact",
                        ProofNodeKind::Attempt => "attempt",
                        ProofNodeKind::Conjecture => "conjecture",
                    },
                    format!("{} {} {}", node.label, node.statement, node.content),
                    Option::<String>::None,
                    serde_json::to_string(&session.proof.imports)?,
                    serde_json::to_string(&serde_json::json!({
                        "kind": format!("{:?}", node.kind),
                        "workspaceRoot": session.workspace_root,
                        "workspaceLabel": session.workspace_label
                    }))?,
                    session.id,
                    node.id,
                    now,
                    now,
                ],
            )?;

            if session.cloud.sync_enabled && session.cloud.share_mode != ShareMode::Local {
                let payload = serde_json::json!({
                    "visibilityScope": visibility,
                    "items": [{
                        "identityKey": format!(
                            "user-verified/{}/{}/{}",
                            sanitize_identity_segment(session.id.as_str()),
                            sanitize_identity_segment(node.label.as_str()),
                            stable_hash(node.statement.as_str())
                        ),
                        "label": node.label,
                        "statement": node.statement,
                        "artifactId": artifact_id,
                        "artifactContent": node.content,
                        "visibility": visibility,
                    }]
                });
                tx.execute(
                    r#"
                    INSERT INTO sync_queue (id, session_id, queue_type, payload_json, status, created_at, updated_at)
                    VALUES (?, ?, 'corpus.contribute', ?, 'pending', ?, ?)
                    "#,
                    params![
                        next_store_id("sync"),
                        session.id,
                        serde_json::to_string(&payload)?,
                        now,
                        now,
                    ],
                )?;
            }
        } else {
            let failure_class = classify_failure(result);
            let attempt_hash = stable_hash(
                format!("{}::{}::{}", node.statement, node.content, failure_class).as_str(),
            );
            tx.execute(
                r#"
                INSERT INTO attempt_logs
                (id, attempt_hash, session_id, target_hash, target_label, target_statement, attempt_kind, target_node_id, failure_class, snippet, rendered_scratch, diagnostic, imports_json, metadata_json, occurrence_count, first_seen_at, last_seen_at)
                VALUES (?, ?, ?, ?, ?, ?, 'node', ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
                ON CONFLICT(attempt_hash) DO UPDATE SET
                    diagnostic = excluded.diagnostic,
                    rendered_scratch = excluded.rendered_scratch,
                    last_seen_at = excluded.last_seen_at,
                    occurrence_count = attempt_logs.occurrence_count + 1
                "#,
                params![
                    next_store_id("attempt"),
                    attempt_hash,
                    session.id,
                    stable_hash(node.statement.as_str()),
                    node.label,
                    node.statement,
                    node.id,
                    failure_class,
                    node.content,
                    result.rendered_scratch,
                    summarize_lean_diagnostic(result),
                    serde_json::to_string(&session.proof.imports)?,
                    serde_json::to_string(&serde_json::json!({
                        "workspaceRoot": session.workspace_root,
                        "workspaceLabel": session.workspace_label
                    }))?,
                    now,
                    now,
                ],
            )?;
        }

        tx.commit()?;
        if result.ok {
            let _ = self.rebuild_verified_corpus_clusters()?;
        }
        Ok(())
    }

    pub fn get_corpus_summary(&self) -> Result<CorpusSummary> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT
                (SELECT COUNT(*) FROM verified_corpus_items) AS local_entry_count,
                (SELECT COUNT(*) FROM verified_corpus_items) AS verified_entry_count,
                (SELECT COUNT(*) FROM verified_corpus_clusters) AS cluster_count,
                (SELECT COUNT(*) FROM verified_corpus_items WHERE cluster_role = 'member') AS duplicate_member_count,
                (SELECT COUNT(*) FROM attempt_logs) AS attempt_log_count,
                (SELECT COUNT(*) FROM verified_corpus_items WHERE is_library_seed = 1) AS library_seed_count,
                (SELECT COUNT(*) FROM verified_corpus_items WHERE origin = 'user-verified') AS user_verified_count,
                (SELECT MAX(updated_at) FROM verified_corpus_items) AS latest_updated_at
            "#,
        )?;
        Ok(stmt.query_row([], |row| {
            Ok(CorpusSummary {
                local_entry_count: row.get::<_, i64>(0)?.max(0) as usize,
                verified_entry_count: row.get::<_, i64>(1)?.max(0) as usize,
                cluster_count: row.get::<_, i64>(2)?.max(0) as usize,
                duplicate_member_count: row.get::<_, i64>(3)?.max(0) as usize,
                attempt_log_count: row.get::<_, i64>(4)?.max(0) as usize,
                library_seed_count: row.get::<_, i64>(5)?.max(0) as usize,
                user_verified_count: row.get::<_, i64>(6)?.max(0) as usize,
                latest_updated_at: row.get(7)?,
            })
        })?)
    }

    pub fn rebuild_verified_corpus_clusters(&self) -> Result<CorpusSummary> {
        let conn = self.connect()?;
        let tx = conn.unchecked_transaction()?;
        let mut stmt = tx.prepare(
            r#"
            SELECT id, label, statement, decl_kind, is_theorem_like, content_hash, created_at, updated_at
            FROM verified_corpus_items
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        drop(stmt);

        tx.execute("DELETE FROM verified_corpus_clusters", [])?;
        tx.execute(
            "UPDATE verified_corpus_items SET cluster_id = NULL, cluster_role = NULL, equivalence_confidence = 1",
            [],
        )?;

        let mut clusters: BTreeMap<String, CorpusClusterRecord> = BTreeMap::new();
        let update_item = tx.prepare(
            r#"
            UPDATE verified_corpus_items
            SET cluster_id = ?, cluster_role = ?, equivalence_confidence = ?
            WHERE id = ?
            "#,
        )?;
        let insert_cluster = tx.prepare(
            r#"
            INSERT INTO verified_corpus_clusters
            (id, cluster_key, canonical_item_id, label, statement_preview, member_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;
        let mut update_item = update_item;
        let mut insert_cluster = insert_cluster;

        for (id, label, statement, decl_kind, is_theorem_like, content_hash, created_at, updated_at) in
            items
        {
            let cluster_key = compute_corpus_cluster_key(
                &statement,
                &decl_kind,
                is_theorem_like != 0,
                &content_hash,
            );
            let cluster =
                clusters
                    .entry(cluster_key.clone())
                    .or_insert_with(|| CorpusClusterRecord {
                        id: format!("cluster_{}", &cluster_key[..cluster_key.len().min(24)]),
                        cluster_key: cluster_key.clone(),
                        canonical_item_id: id.clone(),
                        label: if label.trim().is_empty() {
                            "cluster".to_string()
                        } else {
                            label.clone()
                        },
                        statement_preview: preview_text(&statement, 512),
                        member_count: 0,
                        created_at: created_at.clone(),
                        updated_at: updated_at.clone(),
                    });
            cluster.member_count += 1;
            if updated_at > cluster.updated_at {
                cluster.updated_at = updated_at.clone();
            }
            let role = if cluster.canonical_item_id == id {
                "canonical"
            } else {
                "member"
            };
            update_item.execute(params![
                cluster.id,
                role,
                if role == "canonical" { 1.0 } else { 0.92 },
                id
            ])?;
        }

        for cluster in clusters.values() {
            insert_cluster.execute(params![
                cluster.id,
                cluster.cluster_key,
                cluster.canonical_item_id,
                cluster.label,
                cluster.statement_preview,
                cluster.member_count as i64,
                cluster.created_at,
                cluster.updated_at
            ])?;
        }

        drop(update_item);
        drop(insert_cluster);
        tx.commit()?;
        self.get_corpus_summary()
    }

    pub fn get_sync_summary(&self) -> Result<SyncSummary> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) AS pending_count,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS failed_count,
                SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END) AS sent_count
            FROM sync_queue
            "#,
        )?;
        Ok(stmt.query_row([], |row| {
            Ok(SyncSummary {
                pending_count: row.get::<_, Option<i64>>(0)?.unwrap_or(0).max(0) as usize,
                failed_count: row.get::<_, Option<i64>>(1)?.unwrap_or(0).max(0) as usize,
                sent_count: row.get::<_, Option<i64>>(2)?.unwrap_or(0).max(0) as usize,
            })
        })?)
    }

    pub fn search_verified_corpus(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>> {
        let conn = self.connect()?;
        let pattern = format!("%{}%", query.trim());
        let mut stmt = conn.prepare(
            r#"
            SELECT label, statement, visibility
            FROM verified_corpus_items
            WHERE search_text LIKE ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn ingest_default_library_seeds(&self, lean_root: &Path) -> Result<Vec<(String, usize)>> {
        let mut results = Vec::new();
        let mathlib_root = lean_root
            .join(".lake")
            .join("packages")
            .join("mathlib")
            .join("Mathlib");
        if mathlib_root.exists() {
            let count = self.ingest_library_seed_package("mathlib", &mathlib_root, None)?;
            results.push(("mathlib".to_string(), count));
        }
        let openproof_root = lean_root.join("OpenProof");
        if openproof_root.exists() {
            let count = self.ingest_library_seed_package("openproof", &openproof_root, None)?;
            results.push(("openproof".to_string(), count));
        }
        if !results.is_empty() {
            let _ = self.rebuild_verified_corpus_clusters()?;
        }
        Ok(results)
    }

    pub fn pending_sync_jobs(&self) -> Result<Vec<(String, String)>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, payload_json
            FROM sync_queue
            WHERE status = 'pending'
            ORDER BY created_at ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row?);
        }
        Ok(jobs)
    }

    pub fn mark_sync_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_queue SET status = ?, updated_at = ? WHERE id = ?",
            params![status, Utc::now().to_rfc3339(), job_id],
        )?;
        Ok(())
    }

    fn ingest_library_seed_package(
        &self,
        package_name: &str,
        source_root: &Path,
        package_revision: Option<&str>,
    ) -> Result<usize> {
        let files = collect_lean_files(source_root)?;
        if files.is_empty() {
            return Ok(0);
        }
        let conn = self.connect()?;
        let tx = conn.unchecked_transaction()?;
        let now = Utc::now().to_rfc3339();
        let mut inserted = 0usize;
        for file in files {
            let relative = file.strip_prefix(source_root).with_context(|| {
                format!(
                    "stripping {} from {}",
                    source_root.display(),
                    file.display()
                )
            })?;
            let module_name = lean_module_name(relative);
            let contents = fs::read_to_string(&file)
                .with_context(|| format!("reading {}", file.display()))?;
            for item in extract_library_seed_items(&contents) {
                let artifact_hash = stable_hash(&item.statement);
                let artifact_id = format!("seed_artifact_{artifact_hash}");
                let verification_run_id = format!("seed_verification_{artifact_hash}");
                let identity_key = format!(
                    "library-seed/{}/{}/{}",
                    sanitize_identity_segment(package_name),
                    sanitize_identity_segment(&module_name),
                    sanitize_identity_segment(&item.decl_name)
                );
                tx.execute(
                    r#"
                    INSERT INTO verified_artifacts
                    (id, artifact_hash, label, content, imports_json, namespace, metadata_json, created_at, updated_at)
                    VALUES (?, ?, ?, ?, '[]', NULL, ?, ?, ?)
                    ON CONFLICT(artifact_hash) DO UPDATE SET
                        label = excluded.label,
                        content = excluded.content,
                        metadata_json = excluded.metadata_json,
                        updated_at = excluded.updated_at
                    "#,
                    params![
                        artifact_id.as_str(),
                        artifact_hash.as_str(),
                        item.decl_name.as_str(),
                        item.statement.as_str(),
                        serde_json::to_string(&serde_json::json!({
                            "moduleName": &module_name,
                            "packageName": package_name,
                            "sourcePath": file.display().to_string(),
                            "docString": item.doc_string.clone(),
                        }))?,
                        now,
                        now,
                    ],
                )?;
                tx.execute(
                    r#"
                    INSERT OR IGNORE INTO verification_runs
                    (id, session_id, target_kind, target_id, target_label, target_node_id, artifact_id, ok, code, stdout, stderr, error, scratch_path, rendered_scratch, created_at)
                    VALUES (?, NULL, 'library_seed', ?, ?, NULL, ?, 1, NULL, '', '', NULL, ?, ?, ?)
                    "#,
                    params![
                        verification_run_id.as_str(),
                        identity_key.as_str(),
                        item.decl_name.as_str(),
                        artifact_id.as_str(),
                        file.display().to_string(),
                        item.statement.as_str(),
                        now,
                    ],
                )?;
                tx.execute(
                    r#"
                    INSERT INTO verified_corpus_items
                    (id, statement_hash, identity_key, label, statement, content_hash, artifact_id, verification_run_id, visibility, decl_name, module_name, package_name, package_revision, decl_kind, doc_string, search_text, origin, environment_fingerprint, is_theorem_like, is_instance, is_library_seed, namespace, imports_json, metadata_json, source_session_id, source_node_id, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'library-seed', ?, ?, ?, ?, ?, ?, ?, 'library-seed', NULL, ?, ?, 1, NULL, '[]', ?, NULL, NULL, ?, ?)
                    ON CONFLICT(identity_key) DO UPDATE SET
                        label = excluded.label,
                        statement = excluded.statement,
                        content_hash = excluded.content_hash,
                        artifact_id = excluded.artifact_id,
                        verification_run_id = excluded.verification_run_id,
                        decl_name = excluded.decl_name,
                        module_name = excluded.module_name,
                        package_name = excluded.package_name,
                        package_revision = excluded.package_revision,
                        decl_kind = excluded.decl_kind,
                        doc_string = excluded.doc_string,
                        search_text = excluded.search_text,
                        is_theorem_like = excluded.is_theorem_like,
                        is_instance = excluded.is_instance,
                        is_library_seed = excluded.is_library_seed,
                        metadata_json = excluded.metadata_json,
                        updated_at = excluded.updated_at
                    "#,
                    params![
                        next_store_id("corpus"),
                        stable_hash(&item.statement),
                        identity_key.as_str(),
                        item.decl_name.as_str(),
                        item.statement.as_str(),
                        artifact_hash.as_str(),
                        artifact_id.as_str(),
                        verification_run_id.as_str(),
                        item.decl_name.as_str(),
                        module_name.as_str(),
                        package_name,
                        package_revision,
                        item.kind.as_str(),
                        item.doc_string.clone(),
                        item.search_text(&module_name, package_name),
                        if matches!(item.kind.as_str(), "theorem" | "lemma") {
                            1
                        } else {
                            0
                        },
                        if item.kind == "instance" { 1 } else { 0 },
                        serde_json::to_string(&serde_json::json!({
                            "sourcePath": file.display().to_string(),
                            "packageName": package_name,
                            "moduleName": &module_name,
                        }))?,
                        now,
                        now,
                    ],
                )?;
                inserted = inserted.saturating_add(1);
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn upsert_corpus_package(
        &self,
        package_name: &str,
        package_revision: Option<&str>,
        source_type: &str,
        source_url: Option<&str>,
        manifest: &serde_json::Value,
        root_modules: &[String],
    ) -> Result<()> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let id = format!("pkg_{}", stable_hash(package_name));
        conn.execute(
            r#"
            INSERT INTO corpus_packages
            (id, package_name, package_revision, source_type, source_url, manifest_json, root_modules_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(package_name) DO UPDATE SET
                package_revision = excluded.package_revision,
                source_type = excluded.source_type,
                source_url = excluded.source_url,
                manifest_json = excluded.manifest_json,
                root_modules_json = excluded.root_modules_json,
                updated_at = excluded.updated_at
            "#,
            params![
                id,
                package_name,
                package_revision,
                source_type,
                source_url,
                serde_json::to_string(manifest)?,
                serde_json::to_string(root_modules)?,
                now,
                now
            ],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_corpus_module(
        &self,
        module_name: &str,
        package_name: &str,
        package_revision: Option<&str>,
        source_path: Option<&str>,
        imports: &[String],
        environment_fingerprint: &str,
        declaration_count: usize,
    ) -> Result<()> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let id = format!("mod_{}", stable_hash(module_name));
        conn.execute(
            r#"
            INSERT INTO corpus_modules
            (id, module_name, package_name, package_revision, source_path, imports_json, environment_fingerprint, declaration_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(module_name) DO UPDATE SET
                package_name = excluded.package_name,
                package_revision = excluded.package_revision,
                source_path = excluded.source_path,
                imports_json = excluded.imports_json,
                environment_fingerprint = excluded.environment_fingerprint,
                declaration_count = excluded.declaration_count,
                updated_at = excluded.updated_at
            "#,
            params![
                id,
                module_name,
                package_name,
                package_revision,
                source_path,
                serde_json::to_string(imports)?,
                environment_fingerprint,
                declaration_count as i64,
                now,
                now
            ],
        )?;
        Ok(())
    }

    pub fn start_ingestion_run(
        &self,
        kind: &str,
        fingerprint: &str,
        revision_hash: &str,
    ) -> Result<String> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let id = next_store_id("ingest");
        conn.execute(
            r#"
            INSERT INTO ingestion_runs (id, kind, environment_fingerprint, package_revision_set_hash, status, stats_json, error, started_at, updated_at)
            VALUES (?, ?, ?, ?, 'running', '{}', NULL, ?, ?)
            "#,
            params![id, kind, fingerprint, revision_hash, now, now],
        )?;
        Ok(id)
    }

    pub fn finish_ingestion_run(
        &self,
        run_id: &str,
        status: &str,
        stats: &serde_json::Value,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            UPDATE ingestion_runs
            SET status = ?, stats_json = ?, error = ?, updated_at = ?, completed_at = ?
            WHERE id = ?
            "#,
            params![status, serde_json::to_string(stats)?, error, &now, &now, run_id],
        )?;
        Ok(())
    }

    pub fn has_completed_library_seed(
        &self,
        fingerprint: &str,
        revision_hash: &str,
    ) -> Result<bool> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row(
            r#"
            SELECT COUNT(*) FROM ingestion_runs
            WHERE kind = 'library-seed'
              AND environment_fingerprint = ?
              AND package_revision_set_hash = ?
              AND status = 'completed'
            "#,
            params![fingerprint, revision_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_library_seed_declaration_tx(
        &self,
        tx: &rusqlite::Transaction<'_>,
        identity_key: &str,
        decl_name: &str,
        statement: &str,
        artifact_content: &str,
        verification_run_id: &str,
        module_name: &str,
        package_name: &str,
        package_revision: Option<&str>,
        decl_kind: &str,
        doc_string: Option<&str>,
        namespace: Option<&str>,
        search_text: &str,
        is_theorem_like: bool,
        is_instance: bool,
        fingerprint: &str,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let artifact_hash = stable_hash(statement);
        let artifact_id = format!("seed_artifact_{artifact_hash}");

        tx.execute(
            r#"
            INSERT INTO verified_artifacts
            (id, artifact_hash, label, content, imports_json, namespace, metadata_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, '[]', NULL, ?, ?, ?)
            ON CONFLICT(artifact_hash) DO UPDATE SET
                label = excluded.label,
                content = excluded.content,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at
            "#,
            params![
                artifact_id,
                artifact_hash,
                decl_name,
                artifact_content,
                serde_json::to_string(metadata)?,
                &now,
                &now,
            ],
        )?;

        tx.execute(
            r#"
            INSERT OR IGNORE INTO verification_runs
            (id, session_id, target_kind, target_id, target_label, target_node_id, artifact_id, ok, code, stdout, stderr, error, scratch_path, rendered_scratch, created_at)
            VALUES (?, NULL, 'library_seed', ?, ?, NULL, ?, 1, NULL, '', '', NULL, '', ?, ?)
            "#,
            params![
                verification_run_id,
                identity_key,
                decl_name,
                artifact_id,
                statement,
                &now
            ],
        )?;

        tx.execute(
            r#"
            INSERT INTO verified_corpus_items
            (id, statement_hash, identity_key, label, statement, content_hash, artifact_id, verification_run_id, visibility, decl_name, module_name, package_name, package_revision, decl_kind, doc_string, search_text, origin, environment_fingerprint, is_theorem_like, is_instance, is_library_seed, namespace, imports_json, metadata_json, source_session_id, source_node_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'library-seed', ?, ?, ?, ?, ?, ?, ?, 'library-seed', ?, ?, ?, 1, ?, '[]', ?, NULL, NULL, ?, ?)
            ON CONFLICT(identity_key) DO UPDATE SET
                label = excluded.label,
                statement = excluded.statement,
                content_hash = excluded.content_hash,
                artifact_id = excluded.artifact_id,
                verification_run_id = excluded.verification_run_id,
                decl_name = excluded.decl_name,
                module_name = excluded.module_name,
                package_name = excluded.package_name,
                package_revision = excluded.package_revision,
                decl_kind = excluded.decl_kind,
                doc_string = excluded.doc_string,
                search_text = excluded.search_text,
                is_theorem_like = excluded.is_theorem_like,
                is_instance = excluded.is_instance,
                is_library_seed = excluded.is_library_seed,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at
            "#,
            params![
                next_store_id("corpus"),
                stable_hash(statement),
                identity_key,
                decl_name,
                statement,
                &artifact_hash,
                &artifact_id,
                verification_run_id,
                decl_name,
                module_name,
                package_name,
                package_revision,
                decl_kind,
                doc_string,
                search_text,
                fingerprint,
                if is_theorem_like { 1 } else { 0 },
                if is_instance { 1 } else { 0 },
                namespace,
                serde_json::to_string(metadata)?,
                &now,
                &now,
            ],
        )?;
        Ok(())
    }

    pub fn rebuild_corpus_search_index(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            r#"
            DROP TABLE IF EXISTS verified_corpus_search;
            CREATE VIRTUAL TABLE IF NOT EXISTS verified_corpus_search USING fts5(
                item_id UNINDEXED,
                identity_key,
                label,
                statement,
                search_text,
                decl_name,
                module_name,
                package_name,
                doc_string,
                namespace,
                imports_text,
                tokenize = 'porter unicode61'
            );
            INSERT INTO verified_corpus_search
            (item_id, identity_key, label, statement, search_text, decl_name, module_name, package_name, doc_string, namespace, imports_text)
            SELECT
                id,
                identity_key,
                label,
                statement,
                search_text,
                COALESCE(decl_name, ''),
                COALESCE(module_name, ''),
                COALESCE(package_name, ''),
                COALESCE(doc_string, ''),
                COALESCE(namespace, ''),
                COALESCE(REPLACE(REPLACE(imports_json, '[', ''), ']', ''), '')
            FROM verified_corpus_items;
            "#,
        )?;
        Ok(())
    }

    pub fn cache_remote_corpus_hits(
        &self,
        cache_key: &str,
        hits: &[openproof_protocol::CloudCorpusSearchHit],
    ) -> Result<()> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        for hit in hits {
            let id = format!(
                "rcache_{}_{}",
                stable_hash(cache_key),
                stable_hash(&hit.identity_key)
            );
            conn.execute(
                r#"
                INSERT INTO remote_corpus_cache (id, cache_key, identity_key, item_json, score, cached_at, last_seen_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    item_json = excluded.item_json,
                    score = excluded.score,
                    last_seen_at = excluded.last_seen_at
                "#,
                params![
                    id,
                    cache_key,
                    hit.identity_key,
                    serde_json::to_string(hit)?,
                    hit.score,
                    &now,
                    &now
                ],
            )?;
        }
        Ok(())
    }

    pub fn search_remote_corpus_cache(
        &self,
        query: &str,
        limit: usize,
        cache_key: &str,
    ) -> Result<Vec<openproof_protocol::CloudCorpusSearchHit>> {
        let conn = self.connect()?;
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let mut stmt = conn.prepare(
            r#"
            SELECT item_json, score
            FROM remote_corpus_cache
            WHERE cache_key = ? AND LOWER(identity_key) LIKE ?
            ORDER BY score DESC, last_seen_at DESC
            LIMIT ?
            "#,
        )?;
        let rows = stmt.query_map(params![cache_key, pattern, limit as i64], |row| {
            let json_str: String = row.get(0)?;
            Ok(json_str)
        })?;
        let mut results = Vec::new();
        for row in rows {
            let json_str = row?;
            if let Ok(hit) =
                serde_json::from_str::<openproof_protocol::CloudCorpusSearchHit>(&json_str)
            {
                results.push(hit);
            }
        }
        Ok(results)
    }

    pub fn list_sync_jobs_full(
        &self,
        limit: usize,
    ) -> Result<Vec<openproof_protocol::SyncQueueItem>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, queue_type, payload_json, status, created_at, updated_at
            FROM sync_queue
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(openproof_protocol::SyncQueueItem {
                id: row.get(0)?,
                session_id: row.get(1)?,
                queue_type: row.get(2)?,
                payload_json: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// Get recent failed attempts for a given target, for negative retrieval.
    /// Returns (failure_class, snippet_preview, diagnostic_preview) tuples.
    pub fn failed_attempts_for_target(
        &self,
        target_label: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT failure_class, snippet, diagnostic
            FROM attempt_logs
            WHERE target_label = ? OR target_statement LIKE ?
            ORDER BY last_seen_at DESC
            LIMIT ?
            "#,
        )?;
        let pattern = format!("%{}%", target_label);
        let rows = stmt.query_map(params![target_label, pattern, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)
                    .unwrap_or_default()
                    .chars()
                    .take(200)
                    .collect::<String>(),
                row.get::<_, String>(2)
                    .unwrap_or_default()
                    .chars()
                    .take(200)
                    .collect::<String>(),
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn list_verified_upload_candidates(
        &self,
        limit: usize,
        identity_keys: &[String],
        visibility: &str,
    ) -> Result<Vec<openproof_protocol::CloudCorpusSearchHit>> {
        let conn = self.connect()?;
        let mut results = Vec::new();
        if identity_keys.is_empty() {
            return Ok(results);
        }
        let placeholders = identity_keys
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            r#"
            SELECT c.id, c.identity_key, c.label, c.statement, c.content_hash, c.artifact_id,
                   c.verification_run_id, c.visibility, c.decl_name, c.module_name,
                   c.package_name, c.package_revision, c.decl_kind, c.doc_string,
                   c.search_text, c.origin, c.environment_fingerprint,
                   c.is_theorem_like, c.is_instance, c.is_library_seed,
                   c.namespace, c.imports_json, c.metadata_json,
                   c.created_at, c.updated_at,
                   COALESCE(a.content, '') as artifact_content
            FROM verified_corpus_items c
            LEFT JOIN verified_artifacts a ON a.id = c.artifact_id
            WHERE c.identity_key IN ({placeholders})
              AND c.visibility = ?
              AND c.origin != 'library-seed'
            ORDER BY c.updated_at DESC
            LIMIT ?
            "#,
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = identity_keys
            .iter()
            .map(|k| Box::new(k.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        param_values.push(Box::new(visibility.to_string()));
        param_values.push(Box::new(limit as i64));
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(refs.as_slice(), |row| {
            Ok(openproof_protocol::CloudCorpusSearchHit {
                id: row.get(0)?,
                identity_key: row.get(1)?,
                label: row.get(2)?,
                statement: row.get(3)?,
                content_hash: row.get(4)?,
                artifact_id: row.get(5)?,
                verification_run_id: row.get(6)?,
                visibility: row.get(7)?,
                decl_name: row.get(8)?,
                module_name: row.get(9)?,
                package_name: row.get(10)?,
                package_revision: row.get(11)?,
                decl_kind: row.get::<_, String>(12).unwrap_or_default(),
                doc_string: row.get(13)?,
                search_text: row.get::<_, String>(14).unwrap_or_default(),
                origin: row.get::<_, String>(15).unwrap_or_default(),
                environment_fingerprint: row.get(16)?,
                is_theorem_like: row.get::<_, i64>(17).unwrap_or(0) == 1,
                is_instance: row.get::<_, i64>(18).unwrap_or(0) == 1,
                is_library_seed: row.get::<_, i64>(19).unwrap_or(0) == 1,
                namespace: row.get(20)?,
                imports: serde_json::from_str::<Vec<String>>(
                    &row.get::<_, String>(21).unwrap_or_default(),
                )
                .unwrap_or_default(),
                metadata: serde_json::from_str::<serde_json::Value>(
                    &row.get::<_, String>(22).unwrap_or_default(),
                )
                .unwrap_or_default(),
                created_at: row.get::<_, String>(23).unwrap_or_default(),
                updated_at: row.get::<_, String>(24).unwrap_or_default(),
                artifact_content: row.get::<_, String>(25).unwrap_or_default(),
                score: 0.0,
                statement_hash: String::new(),
                cluster_id: None,
                cluster_role: None,
                equivalence_confidence: None,
                kind: String::new(),
                source_session_id: None,
                source_node_id: None,
            })
        })?;
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
