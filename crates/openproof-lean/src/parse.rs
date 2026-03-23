//! Parse Lean source files to extract declarations and build proof trees.

use chrono::Utc;
use openproof_protocol::{ProofNode, ProofNodeKind, ProofNodeStatus};

/// A declaration extracted from a Lean source file.
#[derive(Debug, Clone)]
pub struct LeanDeclaration {
    pub kind: &'static str, // "theorem", "lemma", "def", "axiom"
    pub name: String,
    pub signature: String, // everything after the name up to := or where
    pub body: String,      // the full declaration text
    pub line: usize,
}

/// Parse a Lean source file and extract all top-level declarations.
/// Returns declarations in source order.
pub fn parse_lean_declarations(content: &str) -> Vec<LeanDeclaration> {
    let mut decls = Vec::new();
    let keywords = ["theorem", "lemma", "def", "noncomputable def", "axiom"];
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        let matched_kw = keywords.iter().find(|&&kw| {
            trimmed.starts_with(kw) && trimmed[kw.len()..].starts_with(|c: char| c.is_whitespace())
        });

        if let Some(&kw) = matched_kw {
            let canonical_kind = match kw {
                "noncomputable def" => "def",
                other => other,
            };

            let after_kw = trimmed[kw.len()..].trim();
            let name = after_kw
                .split(|c: char| c.is_whitespace() || c == '(' || c == ':' || c == '{' || c == '[')
                .next()
                .unwrap_or("")
                .to_string();

            if name.is_empty() || name.starts_with('-') {
                i += 1;
                continue;
            }

            let decl_start = i;
            let mut signature = String::new();
            let mut body_lines = vec![lines[i].to_string()];

            let mut j = i + 1;
            let mut _found_body = trimmed.contains(":=") || trimmed.contains(" by") || trimmed.contains(" where");
            while j < lines.len() {
                let next = lines[j];
                let next_trimmed = next.trim();

                if !next_trimmed.is_empty()
                    && !next.starts_with(' ')
                    && !next.starts_with('\t')
                    && keywords.iter().any(|&kw| {
                        next_trimmed.starts_with(kw) && next_trimmed[kw.len()..].starts_with(|c: char| c.is_whitespace())
                    })
                {
                    break;
                }

                if next_trimmed.starts_with("section")
                    || next_trimmed.starts_with("namespace")
                    || next_trimmed == "end"
                    || next_trimmed.starts_with("end ")
                    || next_trimmed.starts_with("#")
                {
                    break;
                }

                body_lines.push(next.to_string());
                if next_trimmed.contains(":=") || next_trimmed.contains(" by") || next_trimmed.contains(" where") {
                    _found_body = true;
                }
                j += 1;
            }

            let full_text = body_lines.join("\n");
            if let Some(name_pos) = full_text.find(&name) {
                let after_name = &full_text[name_pos + name.len()..];
                let sig_end = after_name
                    .find(":=")
                    .or_else(|| after_name.find(" by\n"))
                    .or_else(|| after_name.find(" by "))
                    .or_else(|| after_name.find(" where"))
                    .unwrap_or(after_name.len());
                signature = after_name[..sig_end].trim().to_string();
            }

            decls.push(LeanDeclaration {
                kind: canonical_kind,
                name,
                signature,
                body: full_text,
                line: decl_start + 1,
            });

            i = j;
        } else {
            i += 1;
        }
    }

    decls
}

/// Convert parsed Lean declarations into ProofNode entries for the proof tree.
/// The first theorem/lemma becomes the root; subsequent ones are children.
pub fn declarations_to_proof_nodes(
    decls: &[LeanDeclaration],
    session_id: &str,
) -> Vec<ProofNode> {
    let now = Utc::now().to_rfc3339();
    let mut nodes = Vec::new();
    let mut root_id: Option<String> = None;

    for (i, decl) in decls.iter().enumerate() {
        let kind = match decl.kind {
            "theorem" => ProofNodeKind::Theorem,
            "lemma" => ProofNodeKind::Lemma,
            _ => ProofNodeKind::Artifact,
        };

        let id = format!("lean_{session_id}_{}", decl.name);
        let is_root = i == 0 && matches!(kind, ProofNodeKind::Theorem);

        if is_root {
            root_id = Some(id.clone());
        }

        let parent_id = if is_root {
            None
        } else {
            root_id.clone()
        };

        let depth = if is_root { 0 } else { 1 };

        nodes.push(ProofNode {
            id,
            kind,
            label: decl.name.clone(),
            statement: decl.signature.clone(),
            content: decl.body.clone(),
            status: ProofNodeStatus::Pending,
            parent_id,
            depends_on: Vec::new(),
            depth,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    }

    nodes
}
