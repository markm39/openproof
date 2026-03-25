import React, { useEffect, useMemo, useState, useCallback } from "https://esm.sh/react@18.3.1";
import { createRoot } from "https://esm.sh/react-dom@18.3.1/client";
import htm from "https://esm.sh/htm@3.1.1";
import { ReactFlow, Background, Controls, MiniMap, Handle, Position } from "https://esm.sh/@xyflow/react@12.6.0?deps=react@18.3.1,react-dom@18.3.1";

const h = htm.bind(React.createElement);
const POLL_MS = 2000;

// ── Helpers ─────────────────────────────────────────────────────────────

function statusDot(status) {
  const s = String(status || "").toLowerCase();
  if (s === "verified") return "dot-verified";
  if (s === "proving") return "dot-proving";
  if (s === "failed") return "dot-failed";
  return "dot-pending";
}

function badgeClass(ok) {
  if (ok === true) return "badge badge-green";
  if (ok === false) return "badge badge-red";
  return "badge badge-yellow";
}

// ── App ─────────────────────────────────────────────────────────────────

function App() {
  const [sessions, setSessions] = useState([]);
  const [selectedId, setSelectedId] = useState(null);
  const [session, setSession] = useState(null);
  const [tab, setTab] = useState("overview");
  const [status, setStatus] = useState(null);

  // Poll sessions list
  useEffect(() => {
    let c = false;
    async function poll() {
      try {
        const r = await fetch("/api/status");
        const d = await r.json();
        if (c) return;
        setStatus(d);
        setSessions(d.sessions || []);
        setSelectedId((cur) => cur || d.activeSessionId || d.sessions?.[0]?.id || null);
      } catch {}
    }
    poll();
    const t = setInterval(poll, POLL_MS);
    return () => { c = true; clearInterval(t); };
  }, []);

  // Poll selected session
  useEffect(() => {
    let c = false;
    if (!selectedId) { setSession(null); return () => { c = true; }; }
    async function poll() {
      try {
        const r = await fetch(`/api/session?id=${encodeURIComponent(selectedId)}`);
        const d = await r.json();
        if (!c) setSession(d);
      } catch {}
    }
    poll();
    const t = setInterval(poll, POLL_MS);
    return () => { c = true; clearInterval(t); };
  }, [selectedId]);

  const proof = session?.proof;
  const leanOk = status?.lean?.ok;

  return h`
    <div className="header">
      <span className="header-brand">openproof</span>
      <span className="header-sep" />
      <span className="header-item"><strong>${session?.title || "no session"}</strong></span>
      <span className="header-sep" />
      <span className="header-item">${proof?.phase || "idle"}</span>
      <span className="header-sep" />
      <span className=${badgeClass(leanOk)}>Lean ${leanOk ? "ok" : "?"}</span>
      ${proof?.last_verification ? h`
        <span className=${badgeClass(proof.last_verification.ok)}>
          ${proof.last_verification.ok ? "verified" : "failed"}
        </span>
      ` : null}
    </div>

    <div className="layout">
      <div className="sidebar">
        <div className="sidebar-title">Sessions</div>
        ${sessions.map((s) => h`
          <button key=${s.id}
            className=${`session-item ${selectedId === s.id ? "session-item-active" : ""}`}
            onClick=${() => setSelectedId(s.id)}>
            <strong>${s.title}</strong>
            <small>${s.transcriptEntries || 0} entries \u00b7 ${s.proofNodes || 0} nodes</small>
          </button>
        `)}
      </div>

      <div className="main-area">
        <div className="tabs">
          ${["overview", "graph", "code", "activity", "paper"].map((t) => h`
            <button key=${t}
              className=${`tab ${tab === t ? "tab-active" : ""}`}
              onClick=${() => setTab(t)}>
              ${t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          `)}
        </div>
        <div className="tab-content">
          ${!session ? h`<div className="empty">Select a session</div>`
            : tab === "overview" ? h`<${OverviewTab} session=${session} />`
            : tab === "graph" ? h`<${GraphTab} session=${session} />`
            : tab === "code" ? h`<${CodeTab} sessionId=${selectedId} />`
            : tab === "activity" ? h`<${ActivityTab} session=${session} />`
            : h`<${PaperTab} sessionId=${selectedId} />`}
        </div>
      </div>
    </div>
  `;
}

// ── Overview Tab ─────────────────────────────────────────────────────────

function OverviewTab({ session }) {
  const proof = session?.proof;
  const nodes = proof?.nodes || [];
  const branches = proof?.branches || [];
  const verification = proof?.last_verification;

  return h`
    <div className="overview">
      <div className="overview-panel">
        <div className="panel-title">Proof Nodes (${nodes.length})</div>
        <div className="panel-body">
          ${nodes.length === 0 ? h`<div style=${{ padding: "12px", color: "var(--muted)" }}>No nodes yet</div>` : null}
          ${nodes.map((n) => h`
            <div key=${n.id} className="node-row">
              <div className=${`node-dot ${statusDot(n.status)}`} />
              <span className="node-kind">${n.kind || "node"}</span>
              <span className="node-label">${n.label}</span>
              <span className="node-statement">${n.statement}</span>
            </div>
          `)}
          ${verification ? h`
            <div className="verify-banner">
              <span className=${verification.ok ? "verify-pass" : "verify-fail"}>
                ${verification.ok ? "Lean verified" : "Lean failed"}
              </span>
              ${!verification.ok && verification.stderr ? h`
                <div className="verify-detail">${verification.stderr}</div>
              ` : null}
            </div>
          ` : null}
        </div>
      </div>

      <div className="overview-panel">
        <div className="panel-title">Branches (${branches.length})</div>
        <div className="panel-body">
          ${branches.length === 0 ? h`<div style=${{ padding: "12px", color: "var(--muted)" }}>No branches yet</div>` : null}
          ${branches.map((b) => h`
            <div key=${b.id} className="branch-card">
              <div className="branch-header">
                <span className="branch-role">${b.role}</span>
                <span className="branch-title">${b.title}</span>
                <span className=${badgeClass(b.status === "done")}>${b.status}</span>
              </div>
              ${b.lean_snippet || b.leanSnippet ? h`
                <pre className="branch-snippet">${b.lean_snippet || b.leanSnippet}</pre>
              ` : null}
              ${b.summary ? h`<div className="branch-status">${b.summary}</div>` : null}
            </div>
          `)}
        </div>
      </div>
    </div>
  `;
}

// ── Graph Tab (React Flow) ──────────────────────────────────────────────

const statusColor = (s) => {
  const st = String(s || "").toLowerCase();
  if (st === "verified" || st === "done") return "#22c55e";
  if (st === "proving" || st === "running") return "#eab308";
  if (st === "failed" || st === "error" || st === "blocked") return "#ef4444";
  return "#525252";
};

const roleColor = (r) => {
  const role = String(r || "").toLowerCase();
  if (role === "prover") return "#3b82f6";
  if (role === "repairer") return "#f59e0b";
  if (role === "planner") return "#8b5cf6";
  if (role === "retriever") return "#06b6d4";
  if (role === "critic") return "#ec4899";
  return "#6b7280";
};

const kindIcon = (k) => {
  const kind = String(k || "").toLowerCase();
  if (kind === "theorem") return "\u{1D4AF}";
  if (kind === "lemma") return "\u{2113}";
  if (kind === "def" || kind === "artifact") return "\u{1D49F}";
  if (kind === "axiom") return "\u{1D49C}";
  return "\u25CB";
};

function ProofNodeComponent({ data }) {
  const borderColor = statusColor(data.status);
  return h`
    <div style=${{
      background: "#1a1a1a",
      border: "2px solid " + borderColor,
      borderRadius: 8,
      padding: "8px 12px",
      minWidth: 160,
      maxWidth: 240,
      fontFamily: "system-ui, sans-serif",
    }}>
      <${Handle} type="target" position=${Position.Top} style=${{ background: "#555" }} />
      <div style=${{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
        <span style=${{ fontSize: 14 }}>${kindIcon(data.kind)}</span>
        <strong style=${{ color: "#e5e5e5", fontSize: 12 }}>${data.label}</strong>
      </div>
      <div style=${{ color: "#a3a3a3", fontSize: 10, marginBottom: 2 }}>
        ${data.kind || "node"} \u00b7 ${data.status}
      </div>
      ${data.statement ? h`
        <div style=${{ color: "#525252", fontSize: 9, fontFamily: "monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 220 }}>
          ${data.statement}
        </div>
      ` : null}
      <${Handle} type="source" position=${Position.Bottom} style=${{ background: "#555" }} />
    </div>
  `;
}

function BranchNodeComponent({ data }) {
  const color = roleColor(data.role);
  const hasSnippet = !!(data.lean_snippet || data.leanSnippet || "").trim();
  return h`
    <div style=${{
      background: "#111",
      border: "1.5px solid " + color,
      borderRadius: 5,
      padding: "6px 10px",
      minWidth: 130,
      opacity: hasSnippet ? 1 : 0.7,
      fontFamily: "system-ui, sans-serif",
    }}>
      <${Handle} type="target" position=${Position.Top} style=${{ background: color }} />
      <div style=${{ color, fontSize: 10, fontWeight: 600 }}>
        ${data.role}${data.hidden ? " (hidden)" : ""}
        ${hasSnippet ? h`<span style=${{ color: "#22c55e", marginLeft: 4 }}>\u25CF</span>` : null}
      </div>
      <div style=${{ color: "#737373", fontSize: 9 }}>
        ${String(data.status || "idle")} \u00b7 score ${(data.score || 0).toFixed(0)} \u00b7 ${data.attempt_count || data.attemptCount || 0} tries
      </div>
      ${data.summary ? h`
        <div style=${{ color: "#525252", fontSize: 8, marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 180 }}>
          ${data.summary}
        </div>
      ` : null}
    </div>
  `;
}

function GoalNodeComponent({ data }) {
  const colors = { open: "#f59e0b", in_progress: "#3b82f6", closed: "#22c55e", failed: "#ef4444" };
  const color = colors[data.status] || "#737373";
  const failCount = (data.failed_tactics || data.failedTactics || []).length;
  return h`
    <div style=${{
      background: "#0a0a0a",
      border: "1.5px dashed " + color,
      borderRadius: 6,
      padding: "6px 10px",
      minWidth: 140,
      maxWidth: 220,
      fontFamily: "system-ui, sans-serif",
    }}>
      <${Handle} type="target" position=${Position.Top} style=${{ background: color }} />
      <div style=${{ display: "flex", alignItems: "center", gap: 4, marginBottom: 2 }}>
        <span style=${{ color, fontSize: 10, fontWeight: 600 }}>
          ${data.status === "closed" ? "\u2713" : data.status === "failed" ? "\u2717" : "\u25CB"} goal
        </span>
        ${failCount > 0 ? h`
          <span style=${{ background: "#7f1d1d", color: "#fca5a5", fontSize: 8, padding: "1px 4px", borderRadius: 3 }}>
            ${failCount} failed
          </span>
        ` : null}
        ${data.attempts > 0 ? h`
          <span style=${{ color: "#525252", fontSize: 8 }}>${data.attempts} tried</span>
        ` : null}
      </div>
      <div style=${{ color: "#a3a3a3", fontSize: 9, fontFamily: "monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 200 }}>
        ${(data.goal_text || data.goalText || "").substring(0, 60)}
      </div>
      ${data.tactic_applied || data.tacticApplied ? h`
        <div style=${{ color: "#22c55e", fontSize: 8, marginTop: 2 }}>
          via: ${data.tactic_applied || data.tacticApplied}
        </div>
      ` : null}
      <${Handle} type="source" position=${Position.Bottom} style=${{ background: color }} />
    </div>
  `;
}

const nodeTypes = {
  proofNode: ProofNodeComponent,
  branchNode: BranchNodeComponent,
  goalNode: GoalNodeComponent,
};

function GraphTab({ session }) {
  const proof = session?.proof;
  const proofNodes = proof?.nodes || [];
  const allBranches = proof?.branches || [];
  const [showBranches, setShowBranches] = useState(false);
  const branches = showBranches ? allBranches : [];

  if (proofNodes.length === 0 && allBranches.length === 0) {
    return h`<div className="graph-container">No proof nodes to visualize</div>`;
  }

  // Build React Flow nodes and edges
  const { flowNodes, flowEdges } = useMemo(() => {
    const nodes = [];
    const edges = [];

    // Layout: if nodes have parent_id, use tree layout.
    // Otherwise use a 2-column grid for flat declarations.
    const hasTree = proofNodes.some(n => n.parent_id || n.parentId);

    if (hasTree) {
      // Tree layout by depth
      const byDepth = {};
      for (const n of proofNodes) {
        const d = n.depth || 0;
        if (!byDepth[d]) byDepth[d] = [];
        byDepth[d].push(n);
      }
      for (const n of proofNodes) {
        const d = n.depth || 0;
        const siblings = byDepth[d] || [];
        const idx = siblings.indexOf(n);
        const totalWidth = siblings.length * 220;
        const startX = -(totalWidth / 2) + 110;
        nodes.push({
          id: n.id, type: "proofNode",
          position: { x: startX + idx * 220, y: d * 120 },
          draggable: true,
          data: { ...n, _nodeColor: statusColor(n.status) },
        });
      }
    } else {
      // Grid layout for flat declarations (2 columns)
      const cols = 2;
      for (let i = 0; i < proofNodes.length; i++) {
        const n = proofNodes[i];
        const col = i % cols;
        const row = Math.floor(i / cols);
        nodes.push({
          id: n.id, type: "proofNode",
          position: { x: col * 280, y: row * 100 },
          draggable: true,
          data: { ...n, _nodeColor: statusColor(n.status) },
        });
      }
    }

    // Edges: parent edges, dependency edges, and flow edges
    let prevId = null;
    for (const n of proofNodes) {
      const parentId = n.parent_id || n.parentId;
      if (parentId) {
        edges.push({
          id: "tree-" + n.id, source: parentId, target: n.id,
          style: { stroke: "#3b82f6", strokeWidth: 2 },
          animated: n.status === "proving",
        });
      } else if (prevId && !hasTree) {
        // Flow edge for flat layouts (declaration order)
        edges.push({
          id: "flow-" + n.id, source: prevId, target: n.id,
          style: { stroke: "#333", strokeWidth: 1 },
          type: "smoothstep",
        });
      }
      prevId = n.id;

      // Dependency edges
      for (const depId of (n.depends_on || n.dependsOn || [])) {
        edges.push({
          id: "dep-" + n.id + "-" + depId, source: depId,
          target: n.id,
          style: { stroke: "#6b7280", strokeWidth: 1, strokeDasharray: "4 3" },
        });
      }
    }

    // Position branches below the tree
    const maxDepth = Math.max(0, ...proofNodes.map((n) => n.depth || 0));
    const branchY = (maxDepth + 1) * 100 + 40;
    const branchCols = {};

    for (const b of branches) {
      const focusId = b.focus_node_id || b.focusNodeId || proofNodes[0]?.id;
      if (!branchCols[focusId]) branchCols[focusId] = 0;
      const col = branchCols[focusId]++;

      const parentNode = nodes.find((n) => n.id === focusId);
      const baseX = parentNode ? parentNode.position.x : col * 160;

      const bId = "branch-" + b.id;
      nodes.push({
        id: bId,
        type: "branchNode",
        position: { x: baseX + col * 160, y: branchY },
        draggable: true,
        data: { ...b, _nodeColor: roleColor(b.role) },
      });

      if (focusId) {
        edges.push({
          id: "agent-" + b.id,
          source: focusId,
          target: bId,
          style: { stroke: roleColor(b.role), strokeWidth: 1, strokeDasharray: "4 3", opacity: 0.5 },
        });
      }
    }

    // Add proof goal nodes (from Pantograph proof tree)
    const proofGoals = proof?.proof_goals || proof?.proofGoals || [];
    const goalY = (maxDepth + 1) * 100 + (branches.length > 0 ? 180 : 40);
    for (let i = 0; i < proofGoals.length; i++) {
      const g = proofGoals[i];
      const gId = "goal-" + g.id;
      nodes.push({
        id: gId,
        type: "goalNode",
        position: { x: i * 200 - (proofGoals.length * 100), y: goalY + (g.parent_goal_id || g.parentGoalId ? 80 : 0) },
        draggable: true,
        data: g,
      });
      // Edge from parent goal
      const parentGoalId = g.parent_goal_id || g.parentGoalId;
      if (parentGoalId) {
        edges.push({
          id: "goaltree-" + g.id,
          source: "goal-" + parentGoalId,
          target: gId,
          style: { stroke: "#3b82f6", strokeWidth: 1, strokeDasharray: "2 2" },
          label: g.tactic_applied || g.tacticApplied || "",
          labelStyle: { fill: "#22c55e", fontSize: 8 },
        });
      }
      // Edge from proof node to first-level goals
      if (!parentGoalId && proofNodes.length > 0) {
        const activeNodeId = proof?.active_node_id || proof?.activeNodeId || proofNodes[0]?.id;
        edges.push({
          id: "nodegoal-" + g.id,
          source: activeNodeId,
          target: gId,
          style: { stroke: "#f59e0b", strokeWidth: 1, strokeDasharray: "3 3", opacity: 0.6 },
        });
      }
    }

    return { flowNodes: nodes, flowEdges: edges };
  }, [proofNodes, branches, proof]);

  // Use controlled mode: pass nodes/edges directly to ReactFlow.
  // No useNodesState/useEdgesState -- avoids the state-fighting bug
  // that causes nodes to flash then disappear on poll updates.

  const verification = proof?.last_verification;
  const attemptNum = proof?.attempt_number || proof?.attemptNumber || 0;

  return h`
    <div className="graph-canvas" style=${{ display: "flex", flexDirection: "column", height: "calc(100vh - 120px)" }}>
      <div className="graph-info">
        <span>Phase: <strong>${proof?.phase || "idle"}</strong></span>
        <span>\u00a0\u00b7\u00a0 Nodes: ${proofNodes.length}</span>
        <span>\u00a0\u00b7\u00a0 Goals: ${(proof?.proof_goals || proof?.proofGoals || []).length}</span>
        <span>\u00a0\u00b7\u00a0 Attempts: ${attemptNum}</span>
        <button onClick=${() => setShowBranches(!showBranches)} style=${{
          marginLeft: 12, padding: "2px 8px", fontSize: 10, cursor: "pointer",
          background: showBranches ? "#333" : "#1a1a1a", color: "#a3a3a3",
          border: "1px solid #333", borderRadius: 4,
        }}>${showBranches ? "Hide" : "Show"} agent branches (${allBranches.length})</button>
        ${verification ? h`
          <span>\u00a0\u00b7\u00a0
            <span style=${{ color: verification.ok ? "#22c55e" : "#ef4444" }}>
              ${verification.ok ? "Lean verified" : "Lean failed"}
            </span>
          </span>
        ` : null}
      </div>
      <div style=${{ flex: 1, width: "100%" }}>
        <${ReactFlow}
          nodes=${flowNodes}
          edges=${flowEdges}
          nodeTypes=${nodeTypes}
          fitView
          fitViewOptions=${{ padding: 0.3 }}
          minZoom=${0.2}
          maxZoom=${2}
          defaultViewport=${{ x: 0, y: 0, zoom: 0.8 }}
          proOptions=${{ hideAttribution: true }}
          style=${{ background: "#0a0a0a" }}
        >
          <${Background} color="#222" gap=${20} />
          <${Controls} position="bottom-right" />
          <${MiniMap}
            nodeColor=${(n) => n.data?._nodeColor || "#525252"}
            maskColor="rgba(0,0,0,0.7)"
            style=${{ background: "#111" }}
          />
        <//>
      </div>
    </div>
  `;
}

// ── Paper Tab ───────────────────────────────────────────────────────────

// Lean syntax highlighting (keywords, types, comments, strings)
const LEAN_KEYWORDS = new Set([
  "theorem", "lemma", "def", "abbrev", "instance", "class", "structure",
  "where", "by", "have", "let", "show", "suffices", "calc", "match", "with",
  "if", "then", "else", "do", "return", "for", "in", "open", "import",
  "namespace", "end", "section", "variable", "example", "noncomputable",
  "sorry", "exact", "apply", "intro", "intros", "rw", "simp", "omega",
  "ring", "norm_num", "linarith", "nlinarith", "aesop", "trivial",
  "constructor", "rcases", "obtain", "refine", "cases", "induction",
  "contradiction", "exfalso", "push_neg", "classical", "decide",
]);
const LEAN_TYPES = new Set([
  "Prop", "Type", "Sort", "Nat", "Int", "Bool", "String", "List", "Option",
  "True", "False", "And", "Or", "Not", "Iff", "Exists", "Finset", "Set",
]);

function highlightLean(line) {
  // Comment
  if (line.trimStart().startsWith("--") || line.trimStart().startsWith("/-")) {
    return h`<span style=${{ color: "#525252", fontStyle: "italic" }}>${line}</span>`;
  }
  // sorry gets red
  if (line.includes("sorry")) {
    const parts = line.split("sorry");
    const result = [];
    for (let i = 0; i < parts.length; i++) {
      if (i > 0) result.push(h`<span style=${{ color: "#ef4444", fontWeight: 600 }}>sorry</span>`);
      result.push(highlightTokens(parts[i]));
    }
    return h`<span>${result}</span>`;
  }
  return highlightTokens(line);
}

function highlightTokens(text) {
  return text.replace(/\b(\w+)\b/g, (match) => {
    if (LEAN_KEYWORDS.has(match)) return `\x01kw\x02${match}\x01/kw\x02`;
    if (LEAN_TYPES.has(match)) return `\x01ty\x02${match}\x01/ty\x02`;
    return match;
  }).split(/(\x01kw\x02[^\x01]*\x01\/kw\x02|\x01ty\x02[^\x01]*\x01\/ty\x02)/).map((part, i) => {
    if (part.startsWith("\x01kw\x02")) return h`<span key=${i} style=${{ color: "#c084fc" }}>${part.slice(4, -5)}</span>`;
    if (part.startsWith("\x01ty\x02")) return h`<span key=${i} style=${{ color: "#22d3ee" }}>${part.slice(4, -5)}</span>`;
    return part;
  });
}

function ActivityTab({ session }) {
  const proof = session?.proof;
  const activityLog = proof?.activity_log || proof?.activityLog || [];
  const goals = proof?.proof_goals || proof?.proofGoals || [];

  return h`
    <div style=${{ padding: 16, fontFamily: "monospace", fontSize: 12, maxHeight: "calc(100vh - 180px)", overflow: "auto" }}>
      <h3 style=${{ color: "#e5e5e5", margin: "0 0 12px", fontSize: 14 }}>Live Activity</h3>
      ${activityLog.length === 0 ? h`
        <div style=${{ color: "#525252" }}>No activity yet. Start a proof to see events here.</div>
      ` : h`
        <div style=${{ display: "flex", flexDirection: "column", gap: 4 }}>
          ${activityLog.slice().reverse().map((entry, i) => {
            const kindColors = { tool: "#06b6d4", verify: "#22c55e", search: "#f59e0b", error: "#ef4444" };
            const color = kindColors[entry.kind] || "#737373";
            const time = entry.timestamp ? new Date(entry.timestamp).toLocaleTimeString() : "";
            return h`
              <div key=${i} style=${{ display: "flex", gap: 8, alignItems: "baseline" }}>
                <span style=${{ color: "#525252", fontSize: 10, minWidth: 70 }}>${time}</span>
                <span style=${{ color, fontSize: 10, fontWeight: 600, minWidth: 40 }}>${entry.kind}</span>
                <span style=${{ color: "#a3a3a3" }}>${entry.message}</span>
              </div>
            `;
          })}
        </div>
      `}
      ${goals.length > 0 ? h`
        <h3 style=${{ color: "#e5e5e5", margin: "16px 0 8px", fontSize: 14 }}>Proof Goals (${goals.length})</h3>
        ${goals.map((g, i) => {
          const statusColors = { open: "#f59e0b", in_progress: "#3b82f6", closed: "#22c55e", failed: "#ef4444" };
          const color = statusColors[g.status] || "#737373";
          const failCount = (g.failed_tactics || g.failedTactics || []).length;
          return h`
            <div key=${i} style=${{ padding: "6px 8px", margin: "4px 0", border: "1px solid #333", borderRadius: 4, borderLeftColor: color, borderLeftWidth: 3 }}>
              <div style=${{ display: "flex", gap: 8, alignItems: "center" }}>
                <span style=${{ color, fontWeight: 600, fontSize: 11 }}>${g.status}</span>
                ${failCount > 0 ? h`<span style=${{ color: "#ef4444", fontSize: 10 }}>${failCount} failed tactics</span>` : null}
                ${g.attempts > 0 ? h`<span style=${{ color: "#525252", fontSize: 10 }}>${g.attempts} attempts</span>` : null}
              </div>
              <div style=${{ color: "#a3a3a3", fontSize: 11, marginTop: 2 }}>
                ${(g.goal_text || g.goalText || "").substring(0, 120)}
              </div>
              ${g.tactic_applied || g.tacticApplied ? h`
                <div style=${{ color: "#22c55e", fontSize: 10, marginTop: 2 }}>via: ${g.tactic_applied || g.tacticApplied}</div>
              ` : null}
            </div>
          `;
        })}
      ` : null}
    </div>
  `;
}

function CodeTab({ sessionId }) {
  const filesRef = React.useRef([]);
  const [filePaths, setFilePaths] = useState([]);
  const [selected, setSelected] = useState(null);
  const [content, setContent] = useState("");

  useEffect(() => {
    if (!sessionId) return;
    let c = false;
    async function poll() {
      try {
        const r = await fetch(`/api/workspace?id=${encodeURIComponent(sessionId)}`);
        const d = await r.json();
        if (c) return;
        const f = d.files || d || [];
        filesRef.current = f;
        // Only update file list if paths changed (avoids re-render on content-only changes)
        const newPaths = f.map(x => x.path).join(",");
        const oldPaths = filePaths.join(",");
        if (newPaths !== oldPaths) {
          setFilePaths(f.map(x => x.path));
        }
        // Update content for selected file without resetting scroll
        const sel = selected || (f.length > 0 ? f[0].path : null);
        if (!selected && f.length > 0) setSelected(f[0].path);
        const cur = f.find(x => x.path === sel);
        if (cur) setContent(cur.content || "");
      } catch {}
    }
    poll();
    const t = setInterval(poll, 4000);
    return () => { c = true; clearInterval(t); };
  }, [sessionId, selected]);

  const selectFile = useCallback((path) => {
    setSelected(path);
    const cur = filesRef.current.find(f => f.path === path);
    if (cur) setContent(cur.content || "");
  }, []);

  const lines = content.split("\n");

  return h`
    <div style=${{ display: "flex", height: "100%", gap: 0 }}>
      <div style=${{ width: 180, borderRight: "1px solid #262626", padding: "8px 0", overflow: "auto", flexShrink: 0 }}>
        ${filePaths.map(path => h`
          <button key=${path}
            onClick=${() => selectFile(path)}
            style=${{
              display: "block", width: "100%", textAlign: "left",
              padding: "6px 12px", border: "none", cursor: "pointer",
              background: selected === path ? "#1e293b" : "transparent",
              color: selected === path ? "#e5e5e5" : "#737373",
              fontSize: 12, fontFamily: "'SF Mono', 'Fira Code', monospace",
              borderLeft: selected === path ? "2px solid #3b82f6" : "2px solid transparent",
            }}>
            ${path}
          </button>
        `)}
      </div>
      <div style=${{ flex: 1, overflow: "auto", padding: 0 }}>
        ${content ? h`
          <pre style=${{
            margin: 0, padding: "12px 0", fontFamily: "'SF Mono', 'Fira Code', Consolas, monospace",
            fontSize: 13, lineHeight: 1.6, color: "#e5e5e5", background: "#0a0a0a",
            tabSize: 2, minHeight: "100%",
          }}>
            ${lines.map((line, i) => h`
              <div key=${i} style=${{ display: "flex", minHeight: "1.6em",
                background: line.includes("sorry") ? "rgba(239,68,68,0.08)" : "transparent",
              }}>
                <span style=${{
                  color: "#404040", width: 48, textAlign: "right", paddingRight: 16,
                  userSelect: "none", flexShrink: 0, borderRight: "1px solid #1a1a1a",
                  marginRight: 16,
                }}>${i + 1}</span>
                <code>${highlightLean(line)}</code>
              </div>
            `)}
          </pre>
        ` : h`<div style=${{ color: "#737373", padding: 20 }}>Select a file</div>`}
      </div>
    </div>
  `;
}

function PaperTab({ sessionId }) {
  const [view, setView] = useState("pdf"); // "pdf" or "tex"
  const [tex, setTex] = useState("");
  const [pdfUrl, setPdfUrl] = useState(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  const loadPdf = useCallback(async () => {
    if (!sessionId) return;
    setLoading(true);
    setError("");
    try {
      const r = await fetch(`/api/paper/pdf?id=${encodeURIComponent(sessionId)}`);
      if (!r.ok) {
        const text = await r.text();
        setError(text);
        setPdfUrl(null);
      } else {
        const blob = await r.blob();
        setPdfUrl(URL.createObjectURL(blob));
      }
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  }, [sessionId]);

  const loadTex = useCallback(async () => {
    if (!sessionId) return;
    try {
      const r = await fetch(`/api/paper/tex?id=${encodeURIComponent(sessionId)}`);
      setTex(await r.text());
    } catch {}
  }, [sessionId]);

  useEffect(() => { loadPdf(); loadTex(); }, [loadPdf, loadTex]);

  return h`
    <div className="paper-container">
      <div className="paper-toolbar">
        <button className=${view === "pdf" ? "active" : ""} onClick=${() => setView("pdf")}>
          Compiled PDF
        </button>
        <button className=${view === "tex" ? "active" : ""} onClick=${() => setView("tex")}>
          TeX Source
        </button>
        <button onClick=${loadPdf} style=${{ marginLeft: "auto" }}>Recompile</button>
      </div>
      <div className="paper-body">
        ${view === "pdf" ? (
          loading ? h`<div className="paper-loading">Compiling...</div>`
          : error ? h`<div className="paper-error">${error}</div>`
          : pdfUrl ? h`<embed src=${pdfUrl} type="application/pdf" />`
          : h`<div className="paper-loading">No PDF available</div>`
        ) : h`<textarea className="paper-source" value=${tex} readOnly />`}
      </div>
    </div>
  `;
}

// ── Mount ───────────────────────────────────────────────────────────────

createRoot(document.getElementById("root")).render(h`<${App} />`);
