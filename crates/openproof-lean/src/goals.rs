//! Goal extraction and tactic suggestions from Lean compiler output.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::verify::write_temp_scratch;

/// Extract the goal states at each `sorry` in a Lean file.
/// Returns a list of (line_number, goal_description) pairs.
pub fn extract_sorry_goals(project_dir: &Path, content: &str) -> Result<Vec<(usize, String)>> {
    let scratch_path = write_temp_scratch(content)?;
    let output = Command::new("lake")
        .arg("env")
        .arg("lean")
        .arg(&scratch_path)
        .current_dir(project_dir)
        .output()
        .context("running lean for goal extraction")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");

    let mut goals = Vec::new();
    let mut current_line: Option<usize> = None;
    let mut current_goal = String::new();
    let mut in_goal = false;

    for line in combined.lines() {
        // Match patterns like "Scratch.lean:15:4: error: unsolved goals"
        // or "declaration uses 'sorry'"
        if line.contains("unsolved goals") || line.contains("uses 'sorry'") {
            if let Some(ln) = extract_line_number(line, &scratch_path) {
                if in_goal && current_line.is_some() {
                    goals.push((current_line.unwrap(), current_goal.trim().to_string()));
                }
                current_line = Some(ln);
                current_goal.clear();
                in_goal = true;
            }
        } else if in_goal {
            let trimmed = line.trim();
            if trimmed.is_empty() && !current_goal.is_empty() {
                if let Some(ln) = current_line {
                    goals.push((ln, current_goal.trim().to_string()));
                }
                current_goal.clear();
                current_line = None;
                in_goal = false;
            } else if !trimmed.is_empty() {
                current_goal.push_str(trimmed);
                current_goal.push('\n');
            }
        }
    }
    // Flush last goal
    if in_goal && current_line.is_some() && !current_goal.trim().is_empty() {
        goals.push((current_line.unwrap(), current_goal.trim().to_string()));
    }

    Ok(goals)
}

/// Run `exact?` / `apply?` / `rw?` at the first `sorry` and return Lean's suggestions.
pub fn run_tactic_suggestions(
    project_dir: &Path,
    content: &str,
    tactic: &str,
) -> Result<Vec<String>> {
    // Replace the first `sorry` with the search tactic
    let modified = if let Some(pos) = content.find("sorry") {
        format!("{}{}{}",
            &content[..pos],
            tactic,
            &content[pos + "sorry".len()..])
    } else {
        format!("{content}\n#check {tactic}")
    };

    let scratch_path = write_temp_scratch(&modified)?;
    let output = Command::new("lake")
        .arg("env")
        .arg("lean")
        .arg(&scratch_path)
        .current_dir(project_dir)
        .output()
        .context("running lean for tactic suggestions")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");

    let mut suggestions = Vec::new();
    for line in combined.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Try this:") {
            suggestions.push(rest.trim().to_string());
        } else if trimmed.starts_with("[exact]") || trimmed.starts_with("[apply]") || trimmed.starts_with("[rw]") {
            if let Some(pos) = trimmed.find(']') {
                suggestions.push(trimmed[pos + 1..].trim().to_string());
            }
        }
    }

    Ok(suggestions)
}

fn extract_line_number(error_line: &str, scratch_path: &Path) -> Option<usize> {
    let filename = scratch_path.file_name()?.to_str()?;
    let parts: Vec<&str> = error_line.split(':').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.contains(filename) || part.ends_with(".lean") {
            if let Some(line_str) = parts.get(i + 1) {
                return line_str.trim().parse().ok();
            }
        }
    }
    None
}

/// Extract grounding facts from Lean output: #check results, type signatures,
/// "Try this:" suggestions, and known-good lemma names that Lean reports.
pub fn extract_grounding_from_lean_output(stderr: &str, stdout: &str) -> Vec<String> {
    let combined = format!("{stderr}\n{stdout}");
    let mut facts = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("Try this:") {
            facts.push(format!("LEAN SUGGESTS: {}", rest.trim()));
        }
        if trimmed.starts_with("[exact]") || trimmed.starts_with("[apply]") || trimmed.starts_with("[rw]") {
            if let Some(pos) = trimmed.find(']') {
                facts.push(format!("LEAN SUGGESTS: {}", trimmed[pos + 1..].trim()));
            }
        }
        // Type signatures from #check
        if trimmed.contains(" : ") && !trimmed.contains("error") && !trimmed.starts_with('/') {
            let has_known_pattern = trimmed.contains('\u{2192}') || trimmed.contains('\u{2200}')
                || trimmed.contains('\u{2203}') || trimmed.contains("Prop")
                || (trimmed.contains("(") && trimmed.contains(":"));
            if has_known_pattern && trimmed.len() > 10 && trimmed.len() < 500 {
                facts.push(format!("LEAN REPORTS: {trimmed}"));
            }
        }
    }

    facts.dedup();
    facts
}
