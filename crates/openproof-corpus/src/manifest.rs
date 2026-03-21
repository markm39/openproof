use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LakeManifestPackage {
    pub name: String,
    pub rev: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LakeManifest {
    #[serde(default)]
    pub packages: Vec<LakeManifestPackage>,
}

/// Read and parse the lake-manifest.json from a Lean project directory.
pub fn read_lake_manifest(lean_project_dir: &Path) -> Result<(LakeManifest, String)> {
    let manifest_path = lean_project_dir.join("lake-manifest.json");
    let raw = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let manifest: LakeManifest =
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", manifest_path.display()))?;
    Ok((manifest, raw))
}
