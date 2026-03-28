//! LSP-based best-first tactic search.
//!
//! Operates by recompiling the file for each tactic test (~100ms per tactic).
//! Prefer `pantograph_best_first_search` when Pantograph is available (~3ms).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{bail, Result};

use openproof_lean::goal_state::AttemptResult;
use openproof_lean::lsp_mcp::LeanLspMcp;
use openproof_protocol::{GoalStatus, ProofGoal};

use crate::cache::{hash_goals, TacticCache};
use crate::config::{SearchResult, TacticSearchConfig};
use crate::search::{GoalUpdateFn, ProposeFn};

/// A node in the LSP search tree.
#[derive(Debug, Clone)]
struct SearchNode {
    priority: u64,
    score: usize,
    tactics: Vec<String>,
    goals: Vec<String>,
    sorry_line: usize,
}

impl SearchNode {
    fn new(
        goals: Vec<String>,
        tactics: Vec<String>,
        sorry_line: usize,
        length_penalty: f64,
    ) -> Self {
        let score = goals.len();
        let priority = ((score as f64 + length_penalty * tactics.len() as f64) * 1000.0) as u64;
        Self {
            priority,
            score,
            tactics,
            goals,
            sorry_line,
        }
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Reverse(self.priority)
            .cmp(&Reverse(other.priority))
            .then_with(|| self.tactics.len().cmp(&other.tactics.len()))
    }
}

/// Run best-first search on a single sorry position using LSP.
pub fn best_first_search(
    lsp: &Mutex<LeanLspMcp>,
    propose_fn: &ProposeFn,
    file_path: &Path,
    sorry_line: usize,
    retrieval_context: &str,
    config: &TacticSearchConfig,
    on_goal_update: Option<&GoalUpdateFn>,
) -> Result<SearchResult> {
    let start = Instant::now();
    let mut expansions: usize = 0;
    let mut cache = TacticCache::new();
    let mut seen_states: HashSet<u64> = HashSet::new();
    let mut frontier: BinaryHeap<SearchNode> = BinaryHeap::new();

    // Get initial goal state
    let initial_goals = {
        let mut mcp = lsp.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let goal_state = mcp.get_goals(file_path, sorry_line, None)?;
        goal_state
            .goals_before
            .or(goal_state.goals)
            .unwrap_or_default()
    };

    if initial_goals.is_empty() {
        return Ok(SearchResult::Exhausted { expansions: 0 });
    }

    let initial_hash = hash_goals(&initial_goals);
    seen_states.insert(initial_hash);

    let root_goal_id = format!("lsp-{sorry_line}-root");
    frontier.push(SearchNode::new(
        initial_goals.clone(),
        vec![],
        sorry_line,
        config.length_penalty,
    ));

    if let Some(cb) = on_goal_update {
        cb(ProofGoal {
            id: root_goal_id.clone(),
            goal_text: initial_goals.join("\n"),
            status: GoalStatus::Open,
            sorry_line: Some(sorry_line),
            ..Default::default()
        });
    }

    let mut goal_counter: usize = 0;
    let mut best_partial = SearchNode {
        priority: u64::MAX,
        score: usize::MAX,
        tactics: vec![],
        goals: initial_goals,
        sorry_line,
    };

    while let Some(node) = frontier.pop() {
        if start.elapsed() > config.timeout {
            return Ok(SearchResult::Timeout {
                best_tactics: best_partial.tactics,
                remaining_goals: best_partial.score,
            });
        }
        if expansions >= config.max_expansions {
            return Ok(SearchResult::Exhausted { expansions });
        }
        if node.score < best_partial.score {
            best_partial = node.clone();
        }

        let goal_text = node.goals.first().map(|s| s.as_str()).unwrap_or("");
        if goal_text.is_empty() {
            continue;
        }

        let candidates = match propose_fn(goal_text, retrieval_context, config.beam_width) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if candidates.is_empty() {
            continue;
        }

        let mut to_screen: Vec<String> = Vec::new();
        let mut cached_results: Vec<(String, AttemptResult)> = Vec::new();

        for tactic in &candidates {
            if let Some(cached) = cache.get(goal_text, tactic) {
                cached_results.push((tactic.clone(), cached.clone()));
            } else {
                to_screen.push(tactic.clone());
            }
        }

        let mut screen_results: Vec<AttemptResult> = Vec::new();
        if !to_screen.is_empty() {
            let result = {
                let mut mcp = lsp.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
                if !mcp.is_alive() {
                    bail!("lean-lsp-mcp process died during search");
                }
                mcp.screen_tactics(file_path, node.sorry_line, None, &to_screen)?
            };
            for item in result.items {
                cache.insert(goal_text, &item.snippet, item.clone());
                screen_results.push(item);
            }
            expansions += to_screen.len();
        }

        let all_results: Vec<AttemptResult> = cached_results
            .into_iter()
            .map(|(_, r)| r)
            .chain(screen_results)
            .collect();

        for item in all_results {
            if !item.succeeded() {
                continue;
            }

            if item.is_solved() {
                let mut tactics = node.tactics.clone();
                tactics.push(item.snippet.clone());
                if let Some(cb) = on_goal_update {
                    goal_counter += 1;
                    cb(ProofGoal {
                        id: format!("lsp-{sorry_line}-{goal_counter}"),
                        goal_text: String::new(),
                        status: GoalStatus::Closed,
                        parent_goal_id: Some(root_goal_id.clone()),
                        tactic_applied: Some(item.snippet),
                        sorry_line: Some(sorry_line),
                        ..Default::default()
                    });
                }
                return Ok(SearchResult::Solved {
                    tactics,
                    file_content: String::new(),
                });
            }

            let new_goals = &item.goals;
            let goals_hash = hash_goals(new_goals);
            if config.dedup && !seen_states.insert(goals_hash) {
                continue;
            }

            let mut new_tactics = node.tactics.clone();
            new_tactics.push(item.snippet.clone());

            if let Some(cb) = on_goal_update {
                goal_counter += 1;
                cb(ProofGoal {
                    id: format!("lsp-{sorry_line}-{goal_counter}"),
                    goal_text: new_goals.join("\n"),
                    status: GoalStatus::Open,
                    parent_goal_id: Some(root_goal_id.clone()),
                    tactic_applied: Some(item.snippet.clone()),
                    sorry_line: Some(sorry_line),
                    ..Default::default()
                });
            }

            frontier.push(SearchNode::new(
                new_goals.clone(),
                new_tactics,
                node.sorry_line,
                config.length_penalty,
            ));
        }
    }

    if best_partial.score < usize::MAX && !best_partial.tactics.is_empty() {
        Ok(SearchResult::Partial {
            tactics: best_partial.tactics,
            remaining_goals: best_partial.score,
            file_content: String::new(),
        })
    } else {
        Ok(SearchResult::Exhausted { expansions })
    }
}
