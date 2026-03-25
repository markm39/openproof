use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub(crate) fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| anyhow::anyhow!("opening {}: {e}", db_path.display()))?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 30000;
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            workspace_root TEXT,
            workspace_label TEXT,
            transcript_json TEXT NOT NULL,
            cloud_json TEXT NOT NULL DEFAULT '{}',
            proof_json TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at DESC);
        CREATE TABLE IF NOT EXISTS verified_artifacts (
            id TEXT PRIMARY KEY,
            artifact_hash TEXT NOT NULL UNIQUE,
            label TEXT NOT NULL,
            content TEXT NOT NULL,
            imports_json TEXT NOT NULL DEFAULT '[]',
            namespace TEXT,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS verification_runs (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            target_kind TEXT NOT NULL,
            target_id TEXT,
            target_label TEXT,
            target_node_id TEXT,
            artifact_id TEXT,
            ok INTEGER NOT NULL,
            code INTEGER,
            stdout TEXT NOT NULL,
            stderr TEXT NOT NULL,
            error TEXT,
            scratch_path TEXT NOT NULL,
            rendered_scratch TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS verified_corpus_items (
            id TEXT PRIMARY KEY,
            statement_hash TEXT NOT NULL,
            identity_key TEXT NOT NULL UNIQUE,
            cluster_id TEXT,
            cluster_role TEXT,
            equivalence_confidence REAL NOT NULL DEFAULT 1,
            label TEXT NOT NULL,
            statement TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            artifact_id TEXT NOT NULL,
            verification_run_id TEXT NOT NULL,
            visibility TEXT NOT NULL,
            decl_name TEXT,
            module_name TEXT,
            package_name TEXT,
            package_revision TEXT,
            decl_kind TEXT NOT NULL,
            doc_string TEXT,
            search_text TEXT NOT NULL,
            origin TEXT NOT NULL,
            environment_fingerprint TEXT,
            is_theorem_like INTEGER NOT NULL DEFAULT 1,
            is_instance INTEGER NOT NULL DEFAULT 0,
            is_library_seed INTEGER NOT NULL DEFAULT 0,
            namespace TEXT,
            imports_json TEXT NOT NULL DEFAULT '[]',
            metadata_json TEXT NOT NULL DEFAULT '{}',
            source_session_id TEXT,
            source_node_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_verified_corpus_items_updated_at ON verified_corpus_items(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_verified_corpus_items_visibility ON verified_corpus_items(visibility, updated_at DESC);
        CREATE TABLE IF NOT EXISTS verified_corpus_clusters (
            id TEXT PRIMARY KEY,
            cluster_key TEXT NOT NULL UNIQUE,
            canonical_item_id TEXT,
            label TEXT NOT NULL,
            statement_preview TEXT NOT NULL,
            member_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_verified_corpus_clusters_key ON verified_corpus_clusters(cluster_key);
        CREATE TABLE IF NOT EXISTS attempt_logs (
            id TEXT PRIMARY KEY,
            attempt_hash TEXT NOT NULL UNIQUE,
            session_id TEXT,
            target_hash TEXT NOT NULL,
            target_label TEXT NOT NULL,
            target_statement TEXT NOT NULL,
            attempt_kind TEXT NOT NULL,
            target_node_id TEXT,
            failure_class TEXT NOT NULL,
            snippet TEXT NOT NULL,
            rendered_scratch TEXT NOT NULL,
            diagnostic TEXT NOT NULL,
            imports_json TEXT NOT NULL DEFAULT '[]',
            metadata_json TEXT NOT NULL DEFAULT '{}',
            occurrence_count INTEGER NOT NULL DEFAULT 1,
            first_seen_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_attempt_logs_target ON attempt_logs(target_hash, last_seen_at DESC);
        CREATE TABLE IF NOT EXISTS sync_queue (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            queue_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sync_queue_status ON sync_queue(status, updated_at DESC);
        "#,
    )?;
    ensure_column(
        &conn,
        "sessions",
        "cloud_json",
        "ALTER TABLE sessions ADD COLUMN cloud_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        &conn,
        "sessions",
        "proof_json",
        "ALTER TABLE sessions ADD COLUMN proof_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        &conn,
        "verified_corpus_items",
        "cluster_id",
        "ALTER TABLE verified_corpus_items ADD COLUMN cluster_id TEXT",
    )?;
    ensure_column(
        &conn,
        "verified_corpus_items",
        "cluster_role",
        "ALTER TABLE verified_corpus_items ADD COLUMN cluster_role TEXT",
    )?;
    ensure_column(
        &conn,
        "verified_corpus_items",
        "equivalence_confidence",
        "ALTER TABLE verified_corpus_items ADD COLUMN equivalence_confidence REAL NOT NULL DEFAULT 1",
    )?;
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_verified_corpus_items_cluster ON verified_corpus_items(cluster_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_verified_corpus_clusters_key ON verified_corpus_clusters(cluster_key);

        CREATE TABLE IF NOT EXISTS corpus_packages (
            id TEXT PRIMARY KEY,
            package_name TEXT NOT NULL UNIQUE,
            package_revision TEXT,
            source_type TEXT NOT NULL,
            source_url TEXT,
            manifest_json TEXT NOT NULL DEFAULT '{}',
            root_modules_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS corpus_modules (
            id TEXT PRIMARY KEY,
            module_name TEXT NOT NULL UNIQUE,
            package_name TEXT NOT NULL,
            package_revision TEXT,
            source_path TEXT,
            imports_json TEXT NOT NULL DEFAULT '[]',
            environment_fingerprint TEXT,
            declaration_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_corpus_modules_package ON corpus_modules(package_name, module_name);

        CREATE TABLE IF NOT EXISTS ingestion_runs (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            environment_fingerprint TEXT NOT NULL,
            package_revision_set_hash TEXT NOT NULL,
            status TEXT NOT NULL,
            stats_json TEXT NOT NULL DEFAULT '{}',
            error TEXT,
            started_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_ingestion_runs_kind ON ingestion_runs(kind, updated_at DESC);

        CREATE TABLE IF NOT EXISTS remote_corpus_cache (
            id TEXT PRIMARY KEY,
            cache_key TEXT NOT NULL,
            identity_key TEXT NOT NULL,
            item_json TEXT NOT NULL,
            score REAL NOT NULL DEFAULT 0,
            cached_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_remote_corpus_cache_key ON remote_corpus_cache(cache_key, last_seen_at DESC);

        -- Knowledge graph: edges between corpus items
        CREATE TABLE IF NOT EXISTS corpus_edges (
            id TEXT PRIMARY KEY,
            from_item_key TEXT NOT NULL,
            to_item_key TEXT NOT NULL,
            edge_type TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_corpus_edges_from ON corpus_edges(from_item_key);
        CREATE INDEX IF NOT EXISTS idx_corpus_edges_to ON corpus_edges(to_item_key);

        -- Domain tags for corpus items
        CREATE TABLE IF NOT EXISTS corpus_tags (
            item_key TEXT NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY (item_key, tag)
        );
        CREATE INDEX IF NOT EXISTS idx_corpus_tags_tag ON corpus_tags(tag);

        -- Proof provenance: which items helped prove what
        CREATE TABLE IF NOT EXISTS proof_provenance (
            proof_item_key TEXT NOT NULL,
            used_item_key TEXT NOT NULL,
            context TEXT,
            created_at TEXT NOT NULL,
            PRIMARY KEY (proof_item_key, used_item_key)
        );
        "#,
    )?;

    // Migration: old schema used session_json blob; new schema uses separate columns.
    ensure_column(
        &conn,
        "sessions",
        "transcript_json",
        "ALTER TABLE sessions ADD COLUMN transcript_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        &conn,
        "sessions",
        "cloud_json",
        "ALTER TABLE sessions ADD COLUMN cloud_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        &conn,
        "sessions",
        "proof_json",
        "ALTER TABLE sessions ADD COLUMN proof_json TEXT NOT NULL DEFAULT '{}'",
    )?;

    // If old session_json column exists with data, trigger a re-import by
    // clearing the sessions table. The legacy import on startup will re-read
    // the JSON files and populate the new columns correctly.
    {
        let has_session_json: bool = conn
            .prepare("PRAGMA table_info(sessions)")
            .ok()
            .map(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))
                    .ok()
                    .map(|cols| cols.filter_map(Result::ok).any(|c| c == "session_json"))
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if has_session_json {
            // Old schema detected. Drop all sessions so they get re-imported.
            let _ = conn.execute("DELETE FROM sessions", []);
        }
    }

    Ok(conn)
}

pub(crate) fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }
    conn.execute_batch(ddl)?;
    Ok(())
}
