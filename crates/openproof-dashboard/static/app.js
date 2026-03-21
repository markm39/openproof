import React, { useEffect, useMemo, useState } from "https://esm.sh/react@18.3.1";
import { createRoot } from "https://esm.sh/react-dom@18.3.1/client";
import htm from "https://esm.sh/htm@3.1.1";

const html = htm.bind(React.createElement);
const POLL_MS = 2500;

function formatTime(value) {
  if (!value) return "n/a";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? String(value) : date.toLocaleString();
}

function roleTone(role) {
  switch (String(role || "").toLowerCase()) {
    case "assistant":
      return "assistant";
    case "user":
      return "user";
    case "system":
      return "system";
    default:
      return "notice";
  }
}

function tone(ok) {
  return ok ? "good" : "warn";
}

function App() {
  const [status, setStatus] = useState(null);
  const [selectedId, setSelectedId] = useState(null);
  const [session, setSession] = useState(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function refreshStatus() {
      try {
        const response = await fetch("/api/status");
        if (!response.ok) throw new Error(`status ${response.status}`);
        const payload = await response.json();
        if (cancelled) return;
        setStatus(payload);
        setSelectedId((current) => current || payload.activeSessionId || payload.sessions?.[0]?.id || null);
        setError("");
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    }

    refreshStatus();
    const handle = setInterval(refreshStatus, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(handle);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    if (!selectedId) {
      setSession(null);
      return () => {
        cancelled = true;
      };
    }
    async function refreshSession() {
      try {
        const response = await fetch(`/api/session?id=${encodeURIComponent(selectedId)}`);
        if (!response.ok) throw new Error(`session ${response.status}`);
        const payload = await response.json();
        if (!cancelled) {
          setSession(payload);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    }
    refreshSession();
    const handle = setInterval(refreshSession, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(handle);
    };
  }, [selectedId]);

  const transcript = useMemo(() => session?.transcript || [], [session]);

  return html`
    <main className="shell">
      <section className="hero">
        <div>
          <p className="eyebrow">OpenProof Native Dashboard</p>
          <h1>Rust local runtime status</h1>
          <p className="lede">
            Native shell, native store, native dashboard server. This view is served directly by the Rust app.
          </p>
        </div>
        <div className="hero__meta">
          <div className="chip" data-tone=${status ? tone(status.lean?.ok) : "muted"}>
            Lean ${status?.lean?.leanVersion || "loading"}
          </div>
          <div className="chip" data-tone=${status?.auth?.loggedIn ? "good" : "warn"}>
            ${status?.auth?.loggedIn ? status?.auth?.email || "logged in" : "logged out"}
          </div>
        </div>
      </section>

      <section className="metrics">
        <article className="card">
          <span className="metric-label">Local DB</span>
          <strong>${status?.localDbPath || "loading"}</strong>
        </article>
        <article className="card">
          <span className="metric-label">Sessions</span>
          <strong>${status?.sessionCount ?? "…"}</strong>
        </article>
        <article className="card">
          <span className="metric-label">Plan</span>
          <strong>${status?.auth?.plan || "n/a"}</strong>
        </article>
      </section>

      ${error
        ? html`<section className="error-banner">${error}</section>`
        : null}

      <section className="workspace">
        <aside className="panel panel--sidebar">
          <div className="panel__header">
            <h2>Sessions</h2>
            <span>${status?.sessions?.length || 0}</span>
          </div>
          <div className="session-list">
            ${(status?.sessions || []).map(
              (item) => html`
                <button
                  className=${`session-card ${selectedId === item.id ? "session-card--active" : ""}`}
                  onClick=${() => setSelectedId(item.id)}
                >
                  <strong>${item.title}</strong>
                  <span>${item.workspaceLabel || "workspace"}</span>
                  <small>
                    ${item.transcriptEntries} entries · ${item.proofNodes || 0} proof nodes · ${item.activeNodeLabel || "no focus"} · ${formatTime(item.updatedAt)}
                  </small>
                  ${item.lastExcerpt
                    ? html`<p>${item.lastExcerpt}</p>`
                    : null}
                </button>
              `
            )}
          </div>
        </aside>

        <section className="panel panel--main">
          <div className="panel__header">
            <div>
              <h2>${session?.title || "No session selected"}</h2>
              <span>
                ${session?.workspaceLabel || session?.workspace_label || "workspace"}
                ${session?.proof
                  ? ` · ${session.proof.phase || "idle"} · ${session.proof.activeNodeId || session.proof.active_node_id || "no focus"}`
                  : ""}
              </span>
            </div>
            <div className="muted">${session ? formatTime(session.updatedAt || session.updated_at) : ""}</div>
          </div>
          ${session?.proof
            ? html`
                <div className="card" style=${{ marginBottom: "16px" }}>
                  <strong>Proof status</strong>
                  <pre>${session.proof.statusLine || session.proof.status_line || "Ready."}</pre>
                  <pre>${[
                    `problem: ${session.proof.problem || "none"}`,
                    `formal target: ${session.proof.formalTarget || session.proof.formal_target || "none"}`,
                    `accepted target: ${session.proof.acceptedTarget || session.proof.accepted_target || "none"}`,
                    `search: ${session.proof.searchStatus || session.proof.search_status || "-"}`,
                    `assumptions: ${session.proof.assumptions?.length ? session.proof.assumptions.join(" | ") : "none"}`,
                    `active branch: ${session.proof.activeBranchId || session.proof.active_branch_id || "none"}`,
                    `branches: ${(session.proof.branches || []).length}`,
                    `agents: ${(session.proof.agents || []).length}`,
                  ].join("\n")}</pre>
                  ${(session.proof.pendingQuestion || session.proof.pending_question)
                    ? html`
                        <pre>${[
                          `question: ${(session.proof.pendingQuestion || session.proof.pending_question).prompt}`,
                          `status: ${(session.proof.pendingQuestion || session.proof.pending_question).status}`,
                          `answer: ${(session.proof.pendingQuestion || session.proof.pending_question).answerText || (session.proof.pendingQuestion || session.proof.pending_question).answer_text || "none"}`,
                          ...(((session.proof.pendingQuestion || session.proof.pending_question).options || []).map((option) => {
                            const recommended =
                              ((session.proof.pendingQuestion || session.proof.pending_question).recommendedOptionId ||
                                (session.proof.pendingQuestion || session.proof.pending_question).recommended_option_id) === option.id;
                            return `${option.id}${recommended ? " [recommended]" : ""}: ${option.label}${option.formalTarget || option.formal_target ? ` :: ${option.formalTarget || option.formal_target}` : ""}`;
                          })),
                        ].join("\n")}</pre>
                      `
                    : null}
                  ${(session.proof.paperNotes || session.proof.paper_notes)?.length
                    ? html`
                        <pre>${["paper notes:", ...((session.proof.paperNotes || session.proof.paper_notes).map((note, index) => `${index + 1}. ${note}`))].join("\n")}</pre>
                      `
                    : null}
                  <pre>${(session.proof.nodes || [])
                    .map(
                      (node) =>
                        `${node.id}  ${node.status}  ${node.label} :: ${node.statement}${node.content ? `\n${node.content}` : ""}`
                    )
                    .join("\n") || "No proof nodes yet."}</pre>
                  <pre>${(session.proof.branches || [])
                    .map(
                      (branch) =>
                        `${branch.id}  ${branch.role}  ${branch.status}  ${branch.title}${branch.leanSnippet || branch.lean_snippet ? `\n${branch.leanSnippet || branch.lean_snippet}` : ""}`
                    )
                    .join("\n") || "No branches yet."}</pre>
                  <pre>${(session.proof.agents || [])
                    .map(
                      (agent) =>
                        `${agent.id}  ${agent.role}  ${agent.status}  ${agent.title}\n${(agent.tasks || [])
                          .map((task) => `  ${task.id}  ${task.status}  ${task.title}`)
                          .join("\n")}`
                    )
                    .join("\n") || "No agents yet."}</pre>
                </div>
              `
            : null}
          <div className="transcript">
            ${transcript.length
              ? transcript.map(
                  (entry) => html`
                    <article className="entry" data-role=${roleTone(entry.role)}>
                      <header>
                        <span className="entry__role">${entry.role}</span>
                        <span className="entry__time">${formatTime(entry.createdAt || entry.created_at)}</span>
                      </header>
                      ${entry.title ? html`<h3>${entry.title}</h3>` : null}
                      <pre>${entry.content}</pre>
                    </article>
                  `
                )
              : html`<div className="empty">No transcript loaded yet.</div>`}
          </div>
        </section>
      </section>
    </main>
  `;
}

createRoot(document.getElementById("root")).render(html`<${App} />`);
