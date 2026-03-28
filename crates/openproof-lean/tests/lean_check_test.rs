//! Integration tests for Pantograph fast paths: lean_check, lean_verify, lean_eval.
//!
//! Requires Pantograph and Lean with Mathlib. Ignored if Pantograph is not available.

use openproof_lean::proof_tree::SessionProver;
use openproof_lean::tools::{execute_tool, ToolContext};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn lean_project_dir() -> &'static Path {
    // cargo test runs from the package root (crates/openproof-lean/),
    // but the lean project is at the workspace root's lean/ directory.
    Path::new("../../lean")
}

fn spawn_prover() -> Option<Arc<Mutex<SessionProver>>> {
    match SessionProver::spawn(lean_project_dir()) {
        Ok(p) => Some(Arc::new(Mutex::new(p))),
        Err(e) => {
            eprintln!("SessionProver::spawn failed: {e:#}");
            None
        }
    }
}

#[test]
#[ignore] // Requires Pantograph + Mathlib (~18s startup)
fn pantograph_single_expr() {
    let prover = spawn_prover().expect("Pantograph not available");
    let ctx = ToolContext {
        project_dir: lean_project_dir(),
        workspace_dir: Path::new("/tmp"),
        imports: &[],
        lsp_mcp: None,
        prover: Some(prover),
    };

    let start = Instant::now();
    let result = execute_tool("lean_check", r#"{"expr": "deriv_add"}"#, &ctx);
    let elapsed = start.elapsed();

    assert!(result.success, "lean_check failed: {}", result.content);
    assert!(
        result.content.contains("deriv_add"),
        "output should contain expression name: {}",
        result.content
    );
    assert!(
        result.content.contains("deriv"),
        "output should contain type info: {}",
        result.content
    );
    // Pantograph path should be fast (< 2s, typically <100ms)
    assert!(
        elapsed.as_secs() < 2,
        "Pantograph inspect took too long: {elapsed:?}"
    );
}

#[test]
#[ignore] // Requires Pantograph + Mathlib (~18s startup)
fn pantograph_batch_exprs() {
    let prover = spawn_prover().expect("Pantograph not available");
    let ctx = ToolContext {
        project_dir: lean_project_dir(),
        workspace_dir: Path::new("/tmp"),
        imports: &[],
        lsp_mcp: None,
        prover: Some(prover),
    };

    let start = Instant::now();
    let result = execute_tool(
        "lean_check",
        r#"{"exprs": ["deriv_add", "Nat.Prime.dvd_mul", "List.map"]}"#,
        &ctx,
    );
    let elapsed = start.elapsed();

    assert!(
        result.success,
        "batch lean_check failed: {}",
        result.content
    );
    assert!(
        result.content.contains("deriv_add"),
        "should contain deriv_add"
    );
    assert!(
        result.content.contains("Nat.Prime.dvd_mul"),
        "should contain Nat.Prime.dvd_mul"
    );
    assert!(
        result.content.contains("List.map"),
        "should contain List.map"
    );
    // 3 expressions via Pantograph should still be fast
    assert!(
        elapsed.as_secs() < 3,
        "Batch Pantograph inspect took too long: {elapsed:?}"
    );
}

#[test]
#[ignore] // Requires Pantograph + Mathlib (~18s startup)
fn pantograph_unknown_expr() {
    let prover = spawn_prover().expect("Pantograph not available");
    let ctx = ToolContext {
        project_dir: lean_project_dir(),
        workspace_dir: Path::new("/tmp"),
        imports: &[],
        lsp_mcp: None,
        prover: Some(prover),
    };

    let result = execute_tool(
        "lean_check",
        r#"{"expr": "totally_fake_lemma_xyz_12345"}"#,
        &ctx,
    );
    // Should not crash; returns success=false with descriptive message
    assert!(!result.success);
    assert!(
        result.content.contains("not found") || result.content.contains("error"),
        "should indicate not found: {}",
        result.content
    );
}

#[test]
fn lean_check_missing_args() {
    let ctx = ToolContext {
        project_dir: lean_project_dir(),
        workspace_dir: Path::new("/tmp"),
        imports: &[],
        lsp_mcp: None,
        prover: None,
    };

    let result = execute_tool("lean_check", r#"{}"#, &ctx);
    assert!(!result.success);
    assert!(
        result.content.contains("missing"),
        "should report missing args: {}",
        result.content
    );
}
