//! Tool execution for the LLM agent's coding tools.
//!
//! Each tool operates within a sandboxed session workspace directory.
//! Paths are validated to prevent escaping the workspace.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Maximum output size returned to the model (characters).
const MAX_OUTPUT_CHARS: usize = 8000;

/// Timeout for Lean commands.
const LEAN_TIMEOUT_SECS: u64 = 120;

/// Context needed to execute tools.
pub struct ToolContext<'a> {
    /// Path to the Lean project (contains lakefile.toml).
    pub project_dir: &'a Path,
    /// Path to the session workspace directory.
    pub workspace_dir: &'a Path,
    /// Current import list for the session.
    pub imports: &'a [String],
}

/// Result of executing a tool.
#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub success: bool,
    pub content: String,
}

/// Execute a tool by name with JSON arguments.
pub fn execute_tool(name: &str, arguments: &str, ctx: &ToolContext) -> ToolOutput {
    let args: Value = serde_json::from_str(arguments).unwrap_or(Value::Object(Default::default()));
    let result = match name {
        "lean_verify" => tool_lean_verify(&args, ctx),
        "lean_check" => tool_lean_check(&args, ctx),
        "lean_eval" => tool_lean_eval(&args, ctx),
        "lean_search_tactic" => tool_lean_search_tactic(&args, ctx),
        "file_read" => tool_file_read(&args, ctx),
        "file_write" => tool_file_write(&args, ctx),
        "file_patch" => tool_file_patch(&args, ctx),
        "workspace_ls" => tool_workspace_ls(ctx),
        "shell_run" => tool_shell_run(&args, ctx),
        _ => Err(anyhow::anyhow!("unknown tool: {name}")),
    };
    match result {
        Ok(output) => output,
        Err(err) => ToolOutput {
            success: false,
            content: truncate_output(&format!("Error: {err:#}")),
        },
    }
}

fn tool_lean_verify(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let file = args
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or("Scratch.lean");
    let target = sanitize_path(ctx.workspace_dir, file)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("reading {file}"))?;

    // If the content already has imports, use as-is. Otherwise prepend imports.
    let full_content = if content.trim_start().starts_with("import ") {
        content
    } else {
        let imports = if ctx.imports.is_empty() {
            vec!["Mathlib".to_string()]
        } else {
            ctx.imports.to_vec()
        };
        let mut lines: Vec<String> = imports.iter().map(|i| format!("import {i}")).collect();
        lines.push(String::new());
        lines.push(content);
        lines.join("\n")
    };

    let scratch_path = write_temp_file(&full_content)?;
    let (ok, output) = run_lean_command(ctx.project_dir, &scratch_path)?;
    Ok(ToolOutput {
        success: ok && !output.contains("declaration uses 'sorry'"),
        content: truncate_output(&output),
    })
}

fn tool_lean_check(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let expr = args
        .get("expr")
        .and_then(Value::as_str)
        .context("missing 'expr' argument")?;
    let imports = build_import_block(ctx.imports);
    let content = format!("{imports}\n#check {expr}\n");
    let scratch_path = write_temp_file(&content)?;
    let (ok, output) = run_lean_command(ctx.project_dir, &scratch_path)?;
    Ok(ToolOutput {
        success: ok,
        content: truncate_output(&output),
    })
}

fn tool_lean_eval(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let expr = args
        .get("expr")
        .and_then(Value::as_str)
        .context("missing 'expr' argument")?;
    let imports = build_import_block(ctx.imports);
    // #eval runs a Lean expression and prints the result
    let content = format!("{imports}\n#eval ({expr})\n");
    let scratch_path = write_temp_file(&content)?;
    let (ok, output) = run_lean_command(ctx.project_dir, &scratch_path)?;
    Ok(ToolOutput {
        success: ok,
        content: truncate_output(&output),
    })
}

fn tool_lean_search_tactic(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let tactic = args
        .get("tactic")
        .and_then(Value::as_str)
        .context("missing 'tactic' argument")?;
    let file = args
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or("Scratch.lean");
    let target_line = args.get("line").and_then(Value::as_u64);

    let target = sanitize_path(ctx.workspace_dir, file)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("reading {file}"))?;

    // Replace the sorry at the specified line (or first sorry) with the search tactic.
    let modified = if let Some(line_num) = target_line {
        replace_sorry_at_line(&content, line_num as usize, tactic)
    } else {
        replace_first_sorry(&content, tactic)
    };

    // Prepend imports if needed.
    let full_content = if modified.trim_start().starts_with("import ") {
        modified
    } else {
        let imports = build_import_block(ctx.imports);
        format!("{imports}\n{modified}")
    };

    let scratch_path = write_temp_file(&full_content)?;
    let (_ok, output) = run_lean_command(ctx.project_dir, &scratch_path)?;

    // Extract just the suggestions from the output.
    let mut suggestions = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Try this:") {
            suggestions.push(rest.trim().to_string());
        } else if trimmed.starts_with("[exact]")
            || trimmed.starts_with("[apply]")
            || trimmed.starts_with("[rw]")
        {
            if let Some(pos) = trimmed.find(']') {
                suggestions.push(trimmed[pos + 1..].trim().to_string());
            }
        }
    }

    if suggestions.is_empty() {
        Ok(ToolOutput {
            success: false,
            content: truncate_output(&format!("No suggestions found.\n\nFull output:\n{output}")),
        })
    } else {
        Ok(ToolOutput {
            success: true,
            content: truncate_output(&suggestions.join("\n")),
        })
    }
}

fn tool_file_read(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .context("missing 'path' argument")?;
    let target = sanitize_path(ctx.workspace_dir, path)?;
    let content = fs::read_to_string(&target)
        .with_context(|| format!("reading {path}"))?;
    // Add line numbers.
    let numbered: String = content
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:4}  {line}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(ToolOutput {
        success: true,
        content: truncate_output(&numbered),
    })
}

fn tool_file_write(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .context("missing 'path' argument")?;
    let content = args
        .get("content")
        .and_then(Value::as_str)
        .context("missing 'content' argument")?;
    let target = sanitize_path(ctx.workspace_dir, path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, content)
        .with_context(|| format!("writing {path}"))?;
    let size = content.len();
    Ok(ToolOutput {
        success: true,
        content: format!("Wrote {size} bytes to {path}"),
    })
}

fn tool_file_patch(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .context("missing 'path' argument")?;
    let patch_text = args
        .get("patch")
        .and_then(Value::as_str)
        .context("missing 'patch' argument")?;
    let target = sanitize_path(ctx.workspace_dir, path)?;
    let original = fs::read_to_string(&target)
        .with_context(|| format!("reading {path} for patching"))?;

    match crate::patch::apply_patch(&original, patch_text) {
        Some(result) => {
            fs::write(&target, &result.patched_content)
                .with_context(|| format!("writing patched {path}"))?;
            Ok(ToolOutput {
                success: true,
                content: format!(
                    "Patch applied to {path}: {} hunks, +{} -{} lines",
                    result.hunks_applied, result.lines_added, result.lines_removed
                ),
            })
        }
        None => Ok(ToolOutput {
            success: false,
            content: "Patch failed: could not match context lines in the file".to_string(),
        }),
    }
}

fn tool_workspace_ls(ctx: &ToolContext) -> Result<ToolOutput> {
    if !ctx.workspace_dir.exists() {
        return Ok(ToolOutput {
            success: true,
            content: "(empty workspace)".to_string(),
        });
    }
    let mut entries = Vec::new();
    walk_dir(ctx.workspace_dir, ctx.workspace_dir, &mut entries)?;
    entries.sort();
    if entries.is_empty() {
        Ok(ToolOutput {
            success: true,
            content: "(empty workspace)".to_string(),
        })
    } else {
        Ok(ToolOutput {
            success: true,
            content: entries.join("\n"),
        })
    }
}

// --- Helpers ---

fn sanitize_path(workspace_dir: &Path, relative: &str) -> Result<PathBuf> {
    let rel = Path::new(relative);
    anyhow::ensure!(rel.is_relative(), "path must be relative: {relative}");
    for component in rel.components() {
        if matches!(component, std::path::Component::ParentDir) {
            anyhow::bail!("path must not contain '..': {relative}");
        }
    }
    Ok(workspace_dir.join(rel))
}

fn write_temp_file(content: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "openproof-lean-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    let path = dir.join("Scratch.lean");
    fs::write(&path, content)?;
    Ok(path)
}

fn build_import_block(imports: &[String]) -> String {
    let list = if imports.is_empty() {
        vec!["Mathlib".to_string()]
    } else {
        imports.to_vec()
    };
    list.iter()
        .map(|i| format!("import {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_lean_command(project_dir: &Path, scratch_path: &Path) -> Result<(bool, String)> {
    let child = Command::new("lake")
        .arg("env")
        .arg("lean")
        .arg(scratch_path)
        .current_dir(project_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("spawning lake env lean")?;

    // Wait with timeout.
    let output = wait_with_timeout(child, Duration::from_secs(LEAN_TIMEOUT_SECS))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stderr}{stdout}").trim().to_string();
    let ok = output.status.success();
    Ok((ok, combined))
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output> {
    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                let stdout = child.stdout.map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default();
                let stderr = child.stderr.map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default();
                return Ok(std::process::Output { status, stdout, stderr });
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    anyhow::bail!("Lean command timed out after {}s", timeout.as_secs());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn replace_first_sorry(content: &str, tactic: &str) -> String {
    if let Some(pos) = content.find("sorry") {
        format!(
            "{}{}{}",
            &content[..pos],
            tactic,
            &content[pos + "sorry".len()..]
        )
    } else {
        content.to_string()
    }
}

fn replace_sorry_at_line(content: &str, target_line: usize, tactic: &str) -> String {
    let mut result = String::new();
    let mut replaced = false;
    for (i, line) in content.lines().enumerate() {
        if i + 1 == target_line && !replaced {
            if let Some(pos) = line.find("sorry") {
                result.push_str(&line[..pos]);
                result.push_str(tactic);
                result.push_str(&line[pos + "sorry".len()..]);
                result.push('\n');
                replaced = true;
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    if !replaced {
        return replace_first_sorry(content, tactic);
    }
    result
}

fn truncate_output(s: &str) -> String {
    if s.len() <= MAX_OUTPUT_CHARS {
        s.to_string()
    } else {
        let truncated = &s[..MAX_OUTPUT_CHARS];
        format!("{truncated}\n\n... (output truncated at {MAX_OUTPUT_CHARS} characters)")
    }
}

fn walk_dir(base: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries = fs::read_dir(current)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().map(|n| n == "history").unwrap_or(false) && path.is_dir() {
            continue;
        }
        if path.is_dir() {
            walk_dir(base, &path, out)?;
        } else {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            out.push(format!("{:<40} {:>8} bytes", relative.display(), size));
        }
    }
    Ok(())
}

/// Shell timeout for computation commands.
const SHELL_TIMEOUT_SECS: u64 = 30;

fn tool_shell_run(args: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .context("missing 'command' argument")?;

    // Use `timeout` command on macOS/Linux to enforce time limit
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("timeout {SHELL_TIMEOUT_SECS} sh -c {}", shell_escape(command)))
        .current_dir(ctx.workspace_dir)
        .output()
        .context("failed to run shell command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{stdout}\n--- stderr ---\n{stderr}")
    };

    // Exit code 124 = timeout killed the process
    let timed_out = output.status.code() == Some(124);
    Ok(ToolOutput {
        success: output.status.success() && !timed_out,
        content: if timed_out {
            format!("Command timed out after {SHELL_TIMEOUT_SECS}s\n{combined}")
        } else {
            truncate_output(&combined)
        },
    })
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
