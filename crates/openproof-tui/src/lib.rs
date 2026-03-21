use openproof_core::{AppState, FocusPane};
use openproof_protocol::ProofNodeStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(40)])
        .split(frame.area());
    let sidebar = chunks[0];
    let main = chunks[1];
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(4),
        ])
        .split(main);
    let content_chunks = if state.show_proof_pane {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(34)])
            .split(main_chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40)])
            .split(main_chunks[0])
    };

    let items = state
        .sessions
        .iter()
        .map(|session| {
            let subtitle = session
                .workspace_label
                .clone()
                .unwrap_or_else(|| "workspace".to_string());
            let proof_summary = if session.proof.nodes.is_empty() {
                "no proof nodes".to_string()
            } else {
                let verified = session
                    .proof
                    .nodes
                    .iter()
                    .filter(|node| node.status == ProofNodeStatus::Verified)
                    .count();
                format!(
                    "{} nodes · {} verified · {}",
                    session.proof.nodes.len(),
                    verified,
                    session
                        .proof
                        .active_node_id
                        .as_deref()
                        .and_then(|id| session.proof.nodes.iter().find(|node| node.id == id))
                        .map(|node| node.label.clone())
                        .unwrap_or_else(|| "no focus".to_string())
                )
            };
            ListItem::new(vec![
                Line::from(Span::styled(
                    session.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(subtitle, Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled(proof_summary, Style::default().fg(Color::Gray))),
            ])
        })
        .collect::<Vec<_>>();
    let sessions = List::new(items)
        .block(block("Sessions", state.focus == FocusPane::Sessions))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("› ");
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_session));
    frame.render_stateful_widget(sessions, sidebar, &mut list_state);

    let transcript_lines = state
        .current_session()
        .map(|session| {
            if session.transcript.is_empty() {
                vec![Line::from("No transcript yet.")]
            } else {
                session
                    .transcript
                    .iter()
                    .flat_map(|entry| {
                        let role = match entry.role {
                            openproof_protocol::MessageRole::User => "you",
                            openproof_protocol::MessageRole::Assistant => "assistant",
                            openproof_protocol::MessageRole::System => "system",
                            openproof_protocol::MessageRole::Notice => "notice",
                        };
                        let title = entry.title.clone().unwrap_or_default();
                        let header = if title.is_empty() {
                            format!("[{}] {}", role, entry.created_at)
                        } else {
                            format!("[{}] {} · {}", role, title, entry.created_at)
                        };
                        [
                            Line::from(Span::styled(header, Style::default().fg(Color::Yellow))),
                            Line::from(entry.content.clone()),
                            Line::from(""),
                        ]
                    })
                    .collect::<Vec<_>>()
            }
        })
        .unwrap_or_else(|| vec![Line::from("No active session.")]);
    let transcript = Paragraph::new(Text::from(transcript_lines))
        .block(block("Transcript", state.focus == FocusPane::Transcript))
        .wrap(Wrap { trim: false })
        .scroll((state.transcript_scroll, 0));
    frame.render_widget(transcript, content_chunks[0]);

    if state.show_proof_pane {
        frame.render_widget(render_proof_pane(state), content_chunks[1]);
    }

    let composer = Paragraph::new(state.composer.as_str())
        .block(block("Composer", state.focus == FocusPane::Composer))
        .wrap(Wrap { trim: false });
    frame.render_widget(composer, main_chunks[1]);

    let proof_line = state
        .current_session()
        .map(|session| {
            let active = session
                .proof
                .active_node_id
                .as_deref()
                .and_then(|id| session.proof.nodes.iter().find(|node| node.id == id))
                .map(|node| format!("{} [{}]", node.label, format_status(node.status)))
                .unwrap_or_else(|| "none".to_string());
            format!(
                "proof: {} · focus: {} · nodes: {}",
                session.proof.phase,
                active,
                session.proof.nodes.len()
            )
        })
        .unwrap_or_else(|| "proof: n/a".to_string());

    let auth = if state.auth.logged_in {
        format!(
            "auth: {} ({})",
            state
                .auth
                .email
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            state
                .auth
                .plan
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        )
    } else {
        "auth: logged out".to_string()
    };
    let lean = if state.lean.ok {
        format!(
            "lean: {}",
            state
                .lean
                .lean_version
                .clone()
                .unwrap_or_else(|| "ok".to_string())
        )
    } else {
        "lean: loading".to_string()
    };
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" focus  "),
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(if state.has_open_question() {
                " clarify  "
            } else {
                " sessions/scroll  "
            }),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(if state.has_open_question() {
                " answer  "
            } else {
                " submit  "
            }),
            Span::styled("/proof", Style::default().fg(Color::Cyan)),
            Span::raw(" pane  "),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]),
        Line::from(auth),
        Line::from(format!(
            "{lean} · turn: {} · verify: {} · pending writes: {}",
            if state.turn_in_flight {
                "running"
            } else {
                "idle"
            },
            if state.verification_in_flight {
                "running"
            } else {
                "idle"
            },
            state.pending_writes,
        )),
        Line::from(proof_line),
        Line::from(state.status.clone()),
    ]))
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, main_chunks[2]);

    if state.has_open_question() {
        render_question_modal(frame, state);
    }
}

fn block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(Span::styled(title, style.add_modifier(Modifier::BOLD)))
}

fn render_proof_pane(state: &AppState) -> Paragraph<'_> {
    let lines = state
        .current_session()
        .map(|session| {
            let active = session
                .proof
                .active_node_id
                .as_deref()
                .and_then(|id| session.proof.nodes.iter().find(|node| node.id == id));
            let active_branch = session
                .proof
                .active_branch_id
                .as_deref()
                .and_then(|id| session.proof.branches.iter().find(|branch| branch.id == id));
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("phase: {}", session.proof.phase),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("status: {}", session.proof.status_line)),
                Line::from(format!(
                    "search: {}",
                    session
                        .proof
                        .search_status
                        .clone()
                        .unwrap_or_else(|| "-".to_string())
                )),
                Line::from(""),
                Line::from(Span::styled("targets", Style::default().fg(Color::Cyan))),
                Line::from(format!(
                    "problem: {}",
                    session
                        .proof
                        .problem
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                )),
                Line::from(format!(
                    "formal: {}",
                    session
                        .proof
                        .formal_target
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                )),
                Line::from(format!(
                    "accepted: {}",
                    session
                        .proof
                        .accepted_target
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                )),
                Line::from(""),
                Line::from(Span::styled("focus", Style::default().fg(Color::Cyan))),
            ];

            if let Some(node) = active {
                lines.push(Line::from(format!(
                    "{} [{}]",
                    node.label,
                    format_status(node.status)
                )));
                lines.push(Line::from(node.statement.clone()));
                if !node.content.trim().is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "candidate",
                        Style::default().fg(Color::Cyan),
                    )));
                    for line in node.content.lines().take(10) {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            } else {
                lines.push(Line::from("none"));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "branch",
                Style::default().fg(Color::Cyan),
            )));
            if let Some(branch) = active_branch {
                lines.push(Line::from(format!(
                    "{} [{}]",
                    branch.title,
                    format_agent_status(branch.status)
                )));
                if !branch.goal_summary.trim().is_empty() {
                    lines.push(Line::from(branch.goal_summary.clone()));
                }
                if !branch.search_status.trim().is_empty() {
                    lines.push(Line::from(format!("search: {}", branch.search_status)));
                }
                if !branch.lean_snippet.trim().is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "branch candidate",
                        Style::default().fg(Color::Cyan),
                    )));
                    for line in branch.lean_snippet.lines().take(8) {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            } else {
                lines.push(Line::from("none"));
            }

            lines.push(Line::from(""));
            if let Some(question) = &session.proof.pending_question {
                lines.push(Line::from(Span::styled(
                    "question",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(question.prompt.clone()));
                for option in question.options.iter().take(4) {
                    let recommended = question
                        .recommended_option_id
                        .as_ref()
                        .map(|value| value == &option.id)
                        .unwrap_or(false);
                    let mut text = format!(
                        "{}{}: {}",
                        option.id,
                        if recommended { " [rec]" } else { "" },
                        option.label
                    );
                    if !option.formal_target.trim().is_empty() {
                        text.push_str(&format!(" :: {}", option.formal_target.trim()));
                    }
                    lines.push(Line::from(text));
                }
                if let Some(answer) = &question.answer_text {
                    lines.push(Line::from(format!("answer: {}", answer)));
                }
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "nodes",
                Style::default().fg(Color::Cyan),
            )));
            if session.proof.nodes.is_empty() {
                lines.push(Line::from("no proof nodes"));
            } else {
                for node in session.proof.nodes.iter().take(10) {
                    lines.push(Line::from(format!(
                        "{} [{}] {}",
                        node.label,
                        format_status(node.status),
                        node.statement
                    )));
                }
            }

            if !session.proof.branches.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "branches",
                    Style::default().fg(Color::Cyan),
                )));
                for branch in session.proof.branches.iter().rev().take(4).rev() {
                    lines.push(Line::from(format!(
                        "{} [{}] {}",
                        agent_role_label(branch.role),
                        format_agent_status(branch.status),
                        branch.title
                    )));
                }
            }

            if !session.proof.agents.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "agents",
                    Style::default().fg(Color::Cyan),
                )));
                for agent in session.proof.agents.iter().rev().take(4).rev() {
                    lines.push(Line::from(format!(
                        "{} [{}] {}",
                        agent_role_label(agent.role),
                        format_agent_status(agent.status),
                        agent.title
                    )));
                }
            }

            if let Some(last) = &session.proof.last_verification {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "last verify",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(if last.ok {
                    "ok".to_string()
                } else {
                    format!("failed: {}", last.stderr.lines().next().unwrap_or("Lean error"))
                }));
            }
            if !session.proof.paper_notes.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "paper",
                    Style::default().fg(Color::Cyan),
                )));
                for note in session.proof.paper_notes.iter().rev().take(4).rev() {
                    lines.push(Line::from(note.clone()));
                }
            }
            lines
        })
        .unwrap_or_else(|| vec![Line::from("No active session.")]);

    Paragraph::new(Text::from(lines))
        .block(block("Proof", false))
        .wrap(Wrap { trim: false })
}

fn render_question_modal(frame: &mut Frame<'_>, state: &AppState) {
    let Some(question) = state.pending_question() else {
        return;
    };
    let area = centered_rect(78, 70, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(7),
            Constraint::Length(4),
        ])
        .split(area);

    let header = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "Clarification Required",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(question.prompt.clone()),
        Line::from("Select an option with ↑/↓ and press Enter to answer immediately."),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Question"))
    .wrap(Wrap { trim: false });
    frame.render_widget(header, chunks[0]);

    let items = question
        .options
        .iter()
        .map(|option| {
            let recommended = question
                .recommended_option_id
                .as_ref()
                .map(|value| value == &option.id)
                .unwrap_or(false);
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    option.id.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    option.label.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                if recommended {
                    Span::styled("  recommended", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw("")
                },
            ])];
            if !option.summary.trim().is_empty() {
                lines.push(Line::from(Span::styled(
                    option.summary.clone(),
                    Style::default().fg(Color::Gray),
                )));
            }
            if !option.formal_target.trim().is_empty() {
                lines.push(Line::from(option.formal_target.clone()));
            }
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .selected_question_option
            .min(question.options.len().saturating_sub(1)),
    ));
    let options = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    frame.render_stateful_widget(options, chunks[1], &mut list_state);

    let mut footer_lines = vec![Line::from(format!("status: {}", question.status))];
    if let Some(answer) = &question.answer_text {
        footer_lines.push(Line::from(format!("latest answer: {}", answer)));
    }
    let footer = Paragraph::new(Text::from(footer_lines))
        .block(Block::default().borders(Borders::ALL).title("Resolution"))
        .wrap(Wrap { trim: false });
    frame.render_widget(footer, chunks[2]);
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage(100 - height_percent - ((100 - height_percent) / 2)),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage(100 - width_percent - ((100 - width_percent) / 2)),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn format_status(status: ProofNodeStatus) -> &'static str {
    match status {
        ProofNodeStatus::Pending => "pending",
        ProofNodeStatus::Suggested => "suggested",
        ProofNodeStatus::Proving => "proving",
        ProofNodeStatus::Verifying => "verifying",
        ProofNodeStatus::Verified => "verified",
        ProofNodeStatus::Failed => "failed",
        ProofNodeStatus::Abandoned => "abandoned",
    }
}

fn format_agent_status(status: openproof_protocol::AgentStatus) -> &'static str {
    match status {
        openproof_protocol::AgentStatus::Idle => "idle",
        openproof_protocol::AgentStatus::Running => "running",
        openproof_protocol::AgentStatus::Blocked => "blocked",
        openproof_protocol::AgentStatus::Done => "done",
        openproof_protocol::AgentStatus::Error => "error",
    }
}

fn agent_role_label(role: openproof_protocol::AgentRole) -> &'static str {
    match role {
        openproof_protocol::AgentRole::Planner => "planner",
        openproof_protocol::AgentRole::Prover => "prover",
        openproof_protocol::AgentRole::Repairer => "repairer",
        openproof_protocol::AgentRole::Retriever => "retriever",
        openproof_protocol::AgentRole::Critic => "critic",
    }
}
