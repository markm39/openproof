use anyhow::{Context, Result};
use openproof_protocol::{IngestLibrarySeedResult, VerifiedCorpusDeclKind};
use openproof_store::AppStore;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::packages::{
    collect_seed_packages, decl_namespace, environment_fingerprint, package_revision_set_hash,
    resolve_package_for_module,
};
use crate::manifest::read_lake_manifest;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "recordType", rename_all = "camelCase")]
enum ExportRecord {
    #[serde(rename = "module")]
    Module {
        #[serde(rename = "moduleName")]
        module_name: String,
        #[serde(rename = "sourcePath")]
        source_path: Option<String>,
        #[serde(default)]
        imports: Vec<String>,
    },
    #[serde(rename = "declaration")]
    Declaration {
        #[serde(rename = "declName")]
        decl_name: String,
        #[serde(rename = "moduleName")]
        module_name: String,
        #[serde(rename = "declKind")]
        decl_kind: String,
        #[serde(rename = "isTheoremLike")]
        is_theorem_like: Option<bool>,
        #[serde(rename = "isInstance")]
        is_instance: Option<bool>,
        #[serde(rename = "isUnsafe")]
        is_unsafe: Option<bool>,
        #[serde(rename = "levelParams")]
        level_params: Option<Vec<String>>,
        #[serde(rename = "typePretty")]
        type_pretty: String,
        #[serde(rename = "docString")]
        doc_string: Option<String>,
        #[serde(rename = "searchText")]
        #[allow(dead_code)]
        search_text: Option<String>,
    },
}

fn hash_text(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(parts.join("\n"));
    format!("{:x}", hasher.finalize())
}

fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && seen.insert(v.clone()))
        .collect()
}

/// Run the full lake-based library seed ingestion pipeline.
///
/// This spawns `lake build` followed by `lake exe openproof-corpus-export`,
/// reads JSONL output line by line, and upserts declarations into the store.
pub async fn run_library_seed_ingestion(
    store: &AppStore,
    lean_project_dir: &Path,
    force: bool,
) -> Result<IngestLibrarySeedResult> {
    let (_, manifest_text) = read_lake_manifest(lean_project_dir)?;
    let packages = collect_seed_packages(lean_project_dir)?;
    let fingerprint = environment_fingerprint(
        &lean_project_dir.to_string_lossy(),
        &manifest_text,
        &packages,
    );
    let revision_hash = package_revision_set_hash(&packages);

    // Check if we can skip
    if !force {
        let already_done = tokio::task::spawn_blocking({
            let store = store.clone();
            let fp = fingerprint.clone();
            let rh = revision_hash.clone();
            move || store.has_completed_library_seed(&fp, &rh)
        })
        .await??;

        if already_done {
            let summary = tokio::task::spawn_blocking({
                let store = store.clone();
                move || store.get_corpus_summary()
            })
            .await??;
            return Ok(IngestLibrarySeedResult {
                run_id: None,
                skipped: true,
                package_count: packages.len(),
                module_count: summary.library_seed_count, // approximate
                declaration_count: summary.library_seed_count,
                environment_fingerprint: fingerprint,
            });
        }
    }

    // Start ingestion run
    let run_id = tokio::task::spawn_blocking({
        let store = store.clone();
        let fp = fingerprint.clone();
        let rh = revision_hash.clone();
        move || store.start_ingestion_run("library-seed", &fp, &rh)
    })
    .await??;

    // Phase 1: lake build
    let root_modules: Vec<String> = unique_strings(
        &packages
            .iter()
            .flat_map(|p| p.root_modules.clone())
            .collect::<Vec<_>>(),
    );
    let mut build_targets = vec!["openproof-corpus-export".to_string()];
    for pkg in &packages {
        if pkg.package_name != "lean4" {
            build_targets.extend(pkg.root_modules.clone());
        }
    }
    let build_targets = unique_strings(&build_targets);

    let mut build_cmd = Command::new("lake");
    build_cmd
        .arg("build")
        .args(&build_targets)
        .current_dir(lean_project_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    let build_output = build_cmd
        .output()
        .await
        .context("failed to spawn lake build")?;
    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        let msg = if stderr.trim().is_empty() {
            format!(
                "lake build failed with exit code {}",
                build_output.status.code().unwrap_or(-1)
            )
        } else {
            stderr.chars().take(20000).collect::<String>()
        };
        tokio::task::spawn_blocking({
            let store = store.clone();
            let run_id = run_id.clone();
            let msg = msg.clone();
            move || store.finish_ingestion_run(&run_id, "failed", &serde_json::json!({}), Some(&msg))
        })
        .await??;
        anyhow::bail!("{msg}");
    }

    // Phase 2: lake exe openproof-corpus-export
    let mut export_cmd = Command::new("lake");
    export_cmd
        .arg("exe")
        .arg("openproof-corpus-export")
        .args(&root_modules)
        .current_dir(lean_project_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = export_cmd
        .spawn()
        .context("failed to spawn lake exe openproof-corpus-export")?;

    let stdout = child.stdout.take().context("missing stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    // Upsert packages first
    {
        let store = store.clone();
        let pkgs = packages.clone();
        tokio::task::spawn_blocking(move || {
            for pkg in &pkgs {
                store.upsert_corpus_package(
                    &pkg.package_name,
                    pkg.package_revision.as_deref(),
                    &pkg.source_type,
                    pkg.source_url.as_deref(),
                    &pkg.manifest,
                    &pkg.root_modules,
                )?;
            }
            Ok::<_, anyhow::Error>(())
        })
        .await??;
    }

    let mut module_count = 0usize;
    let mut declaration_count = 0usize;
    let mut declaration_batch: Vec<DeclarationRecord> = Vec::new();
    let batch_size = 2000;

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: ExportRecord = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(_) => continue,
        };

        match record {
            ExportRecord::Module {
                module_name,
                source_path,
                imports,
            } => {
                let pkg = match resolve_package_for_module(&packages, &module_name) {
                    Some(p) => p,
                    None => continue,
                };
                module_count += 1;
                let store = store.clone();
                let mn = module_name.clone();
                let pn = pkg.package_name.clone();
                let pr = pkg.package_revision.clone();
                let sp = source_path.clone();
                let imp = unique_strings(&imports);
                let fp = fingerprint.clone();
                tokio::task::spawn_blocking(move || {
                    store.upsert_corpus_module(&mn, &pn, pr.as_deref(), sp.as_deref(), &imp, &fp, 0)
                })
                .await??;
            }
            ExportRecord::Declaration {
                decl_name,
                module_name,
                decl_kind,
                is_theorem_like,
                is_instance,
                is_unsafe,
                level_params,
                type_pretty,
                doc_string,
                ..
            } => {
                let pkg = match resolve_package_for_module(&packages, &module_name) {
                    Some(p) => p,
                    None => continue,
                };
                let kind = VerifiedCorpusDeclKind::from_str_normalized(&decl_kind);
                if matches!(kind, VerifiedCorpusDeclKind::Unknown) {
                    continue;
                }
                declaration_count += 1;
                declaration_batch.push(DeclarationRecord {
                    decl_name,
                    module_name,
                    package_name: pkg.package_name.clone(),
                    package_revision: pkg.package_revision.clone(),
                    decl_kind: kind,
                    type_pretty,
                    doc_string,
                    is_theorem_like: is_theorem_like.unwrap_or(false),
                    is_instance: is_instance.unwrap_or(false),
                    is_unsafe: is_unsafe.unwrap_or(false),
                    level_params: level_params.unwrap_or_default(),
                    fingerprint: fingerprint.clone(),
                });

                if declaration_batch.len() >= batch_size {
                    flush_declarations(store, &mut declaration_batch).await?;
                }
            }
        }
    }

    // Flush remaining
    if !declaration_batch.is_empty() {
        flush_declarations(store, &mut declaration_batch).await?;
    }

    let exit_status = child.wait().await?;
    if !exit_status.success() {
        let msg = format!(
            "lake exe openproof-corpus-export failed with exit code {}",
            exit_status.code().unwrap_or(-1)
        );
        tokio::task::spawn_blocking({
            let store = store.clone();
            let run_id = run_id.clone();
            let msg = msg.clone();
            move || store.finish_ingestion_run(&run_id, "failed", &serde_json::json!({}), Some(&msg))
        })
        .await??;
        anyhow::bail!("{msg}");
    }

    // Rebuild search index
    {
        let store = store.clone();
        tokio::task::spawn_blocking(move || store.rebuild_corpus_search_index())
            .await??;
    }

    // Finish run
    let stats = serde_json::json!({
        "packageCount": packages.len(),
        "moduleCount": module_count,
        "declarationCount": declaration_count,
        "environmentFingerprint": fingerprint,
        "rootModules": root_modules,
    });
    tokio::task::spawn_blocking({
        let store = store.clone();
        let run_id = run_id.clone();
        move || store.finish_ingestion_run(&run_id, "completed", &stats, None)
    })
    .await??;

    Ok(IngestLibrarySeedResult {
        run_id: Some(run_id),
        skipped: false,
        package_count: packages.len(),
        module_count,
        declaration_count,
        environment_fingerprint: fingerprint,
    })
}

struct DeclarationRecord {
    decl_name: String,
    module_name: String,
    package_name: String,
    package_revision: Option<String>,
    decl_kind: VerifiedCorpusDeclKind,
    type_pretty: String,
    doc_string: Option<String>,
    is_theorem_like: bool,
    is_instance: bool,
    is_unsafe: bool,
    level_params: Vec<String>,
    fingerprint: String,
}

async fn flush_declarations(store: &AppStore, batch: &mut Vec<DeclarationRecord>) -> Result<()> {
    let items: Vec<DeclarationRecord> = batch.drain(..).collect();
    let store = store.clone();
    tokio::task::spawn_blocking(move || {
        let conn = store.connect_for_bulk()?;
        let tx = conn.unchecked_transaction()?;
        for item in &items {
            let identity_key = format!(
                "{}@{}/{}/{}",
                item.package_name,
                item.package_revision.as_deref().unwrap_or("local"),
                item.module_name,
                item.decl_name
            );
            let verification_run_id = format!(
                "verrun_seed_{}",
                hash_text(&[
                    &item.package_name,
                    item.package_revision.as_deref().unwrap_or("local"),
                    &item.module_name,
                ])
            );
            let artifact_content = format!(
                "-- package: {}{}\nimport {}\n#check {}",
                item.package_name,
                item.package_revision
                    .as_ref()
                    .map(|r| format!("@{r}"))
                    .unwrap_or_default(),
                item.module_name,
                item.decl_name
            );
            let namespace = decl_namespace(&item.decl_name);
            let search_text = [
                &item.decl_name,
                &item.type_pretty,
                &item.module_name,
                &item.package_name,
            ]
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
            let metadata = serde_json::json!({
                "isUnsafe": item.is_unsafe,
                "levelParams": item.level_params,
            });

            store.upsert_library_seed_declaration_tx(
                &tx,
                &identity_key,
                &item.decl_name,
                &item.type_pretty,
                &artifact_content,
                &verification_run_id,
                &item.module_name,
                &item.package_name,
                item.package_revision.as_deref(),
                item.decl_kind.as_str(),
                item.doc_string.as_deref(),
                namespace.as_deref(),
                &search_text,
                item.is_theorem_like,
                item.is_instance,
                &item.fingerprint,
                &metadata,
            )?;
        }
        tx.commit()?;
        Ok::<_, anyhow::Error>(())
    })
    .await?
}
