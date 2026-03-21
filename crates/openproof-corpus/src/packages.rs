use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::manifest::read_lake_manifest;

#[derive(Debug, Clone)]
pub struct PackageSeedInfo {
    pub package_name: String,
    pub package_revision: Option<String>,
    pub source_type: String,
    pub source_url: Option<String>,
    pub manifest: serde_json::Value,
    pub root_modules: Vec<String>,
    pub package_dir: PathBuf,
}

fn hash_text(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(parts.join("\n"));
    format!("{:x}", hasher.finalize())
}

fn preferred_root_module(package_name: &str) -> Option<&'static str> {
    match package_name {
        "mathlib" => Some("Mathlib"),
        "batteries" => Some("Batteries"),
        "aesop" => Some("Aesop"),
        "proofwidgets" => Some("ProofWidgets"),
        "importGraph" => Some("ImportGraph"),
        "plausible" => Some("Plausible"),
        "Cli" => Some("Cli"),
        "LeanSearchClient" => Some("LeanSearchClient"),
        "Qq" => Some("Qq"),
        "OpenProof" => Some("OpenProof"),
        _ => None,
    }
}

fn is_known_source_type(value: Option<&str>) -> &str {
    match value {
        Some("git") => "git",
        Some("local") => "local",
        _ => "unknown",
    }
}

fn detect_root_modules(package_dir: &Path, package_name: &str) -> Result<Vec<String>> {
    let entries = match std::fs::read_dir(package_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };
    let mut modules: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                && e.file_name().to_string_lossy().ends_with(".lean")
                && !e.file_name().to_string_lossy().starts_with('.')
        })
        .map(|e| {
            e.file_name()
                .to_string_lossy()
                .trim_end_matches(".lean")
                .to_string()
        })
        .collect();
    modules.sort();
    if let Some(preferred) = preferred_root_module(package_name) {
        if modules.contains(&preferred.to_string()) {
            return Ok(vec![preferred.to_string()]);
        }
    }
    Ok(modules)
}

fn detect_lean_toolchain_package(lean_project_dir: &Path) -> Option<PackageSeedInfo> {
    let output = Command::new("lean")
        .args(["--print-prefix"])
        .current_dir(lean_project_dir)
        .output()
        .ok()?;
    let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if prefix.is_empty() {
        return None;
    }
    let source_dir = PathBuf::from(&prefix).join("src").join("lean");
    let roots: Vec<String> = ["Init", "Lean", "Std"]
        .iter()
        .filter(|root| source_dir.join(root).is_dir())
        .map(|root| root.to_string())
        .collect();
    if roots.is_empty() {
        return None;
    }
    Some(PackageSeedInfo {
        package_name: "lean4".to_string(),
        package_revision: Some(
            PathBuf::from(&prefix)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        ),
        source_type: "local".to_string(),
        source_url: None,
        manifest: serde_json::json!({ "toolchainPrefix": prefix }),
        root_modules: roots,
        package_dir: source_dir,
    })
}

/// Collect all seed packages from the Lean project.
pub fn collect_seed_packages(lean_project_dir: &Path) -> Result<Vec<PackageSeedInfo>> {
    let (manifest, _raw) = read_lake_manifest(lean_project_dir)?;
    let mut packages = Vec::new();

    for pkg in &manifest.packages {
        let package_dir = lean_project_dir
            .join(".lake")
            .join("packages")
            .join(&pkg.name);
        let root_modules = detect_root_modules(&package_dir, &pkg.name)?;
        packages.push(PackageSeedInfo {
            package_name: pkg.name.clone(),
            package_revision: pkg.rev.clone(),
            source_type: is_known_source_type(pkg.package_type.as_deref()).to_string(),
            source_url: pkg.url.clone(),
            manifest: serde_json::to_value(pkg).unwrap_or_default(),
            root_modules,
            package_dir,
        });
    }

    // Add the local OpenProof package
    let openproof_root_modules = detect_root_modules(lean_project_dir, "OpenProof")?;
    packages.push(PackageSeedInfo {
        package_name: "OpenProof".to_string(),
        package_revision: None,
        source_type: "local".to_string(),
        source_url: None,
        manifest: serde_json::Value::Object(Default::default()),
        root_modules: openproof_root_modules,
        package_dir: lean_project_dir.to_path_buf(),
    });

    // Add the lean4 toolchain package
    if let Some(toolchain) = detect_lean_toolchain_package(lean_project_dir) {
        packages.push(toolchain);
    }

    // Only keep packages that have root modules
    packages.retain(|pkg| !pkg.root_modules.is_empty());
    Ok(packages)
}

/// Resolve which package owns a given module name.
pub fn resolve_package_for_module<'a>(
    packages: &'a [PackageSeedInfo],
    module_name: &str,
) -> Option<&'a PackageSeedInfo> {
    let mut matches: Vec<&PackageSeedInfo> = packages
        .iter()
        .filter(|pkg| {
            pkg.root_modules
                .iter()
                .any(|root| root_matches_module(root, module_name))
        })
        .collect();
    matches.sort_by(|a, b| {
        let a_best = a
            .root_modules
            .iter()
            .filter(|r| root_matches_module(r, module_name))
            .map(|r| r.len())
            .max()
            .unwrap_or(0);
        let b_best = b
            .root_modules
            .iter()
            .filter(|r| root_matches_module(r, module_name))
            .map(|r| r.len())
            .max()
            .unwrap_or(0);
        b_best.cmp(&a_best)
    });
    matches.first().copied()
}

fn root_matches_module(root_module: &str, module_name: &str) -> bool {
    module_name == root_module || module_name.starts_with(&format!("{root_module}."))
}

/// Compute environment fingerprint from package names and revisions.
pub fn environment_fingerprint(
    lean_project_dir: &str,
    manifest_text: &str,
    packages: &[PackageSeedInfo],
) -> String {
    let mut parts = vec![lean_project_dir.trim().to_string(), manifest_text.trim().to_string()];
    let mut pkg_parts: Vec<String> = packages
        .iter()
        .flat_map(|pkg| {
            let mut items = vec![format!(
                "{}@{}",
                pkg.package_name,
                pkg.package_revision.as_deref().unwrap_or("local")
            )];
            for root in &pkg.root_modules {
                items.push(format!("{}:{root}", pkg.package_name));
            }
            items
        })
        .collect();
    pkg_parts.sort();
    parts.extend(pkg_parts);
    let refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    hash_text(&refs)
}

/// Compute a hash of just the package name + revision pairs.
pub fn package_revision_set_hash(packages: &[PackageSeedInfo]) -> String {
    let mut items: Vec<String> = packages
        .iter()
        .map(|pkg| {
            format!(
                "{}@{}",
                pkg.package_name,
                pkg.package_revision.as_deref().unwrap_or("local")
            )
        })
        .collect();
    items.sort();
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    hash_text(&refs)
}

pub fn decl_namespace(decl_name: &str) -> Option<String> {
    let text = decl_name.trim();
    text.rfind('.').and_then(|pos| {
        if pos > 0 {
            Some(text[..pos].to_string())
        } else {
            None
        }
    })
}
