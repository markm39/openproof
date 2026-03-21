import React, { useEffect, useMemo, useState, useCallback } from "https://esm.sh/react@18.3.1";
import { createRoot } from "https://esm.sh/react-dom@18.3.1/client";
import htm from "https://esm.sh/htm@3.1.1";

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
          ${["overview", "graph", "paper"].map((t) => h`
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

// ── Graph Tab ───────────────────────────────────────────────────────────

function GraphTab({ session }) {
  const proof = session?.proof;
  const nodes = proof?.nodes || [];
  const branches = proof?.branches || [];
  const agents = proof?.agents || [];

  if (nodes.length === 0 && branches.length === 0) {
    return h`<div className="graph-container">No proof nodes to visualize</div>`;
  }

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

  // Layout: tree structure based on parent_id / depth
  const nodeW = 160;
  const nodeH = 50;
  const branchW = 140;
  const branchH = 36;
  const gapX = 20;
  const gapY = 30;
  const startX = 30;
  const startY = 30;

  // Group nodes by depth for tree layout
  const maxDepth = Math.max(0, ...nodes.map((n) => n.depth || 0));
  const byDepth = {};
  for (const n of nodes) {
    const d = n.depth || 0;
    if (!byDepth[d]) byDepth[d] = [];
    byDepth[d].push(n);
  }

  // Position nodes: each depth level is a row
  const posNodes = nodes.map((n) => {
    const d = n.depth || 0;
    const siblings = byDepth[d] || [];
    const idx = siblings.indexOf(n);
    return {
      ...n,
      x: startX + idx * (nodeW + gapX),
      y: startY + d * (nodeH + gapY),
    };
  });

  // Position branches below the proof tree, grouped by focusNodeId
  const treeBottom = startY + (maxDepth + 1) * (nodeH + gapY);
  const posBranches = branches.map((b, i) => {
    const parentNode = posNodes.find((n) =>
      (b.focus_node_id || b.focusNodeId) === n.id
    ) || posNodes[0];
    const parentX = parentNode ? parentNode.x : startX;
    const parentY = parentNode ? parentNode.y + nodeH : treeBottom;
    const col = branches.filter((bb, j) =>
      j < i && ((bb.focus_node_id || bb.focusNodeId || "") === (b.focus_node_id || b.focusNodeId || ""))
    ).length;
    return {
      ...b,
      x: parentX + col * (branchW + 10),
      y: treeBottom + 10,
      parentX: parentX + nodeW / 2,
      parentY,
    };
  });

  const width = Math.max(700,
    Math.max(
      ...posNodes.map((n) => n.x + nodeW + 40),
      ...posBranches.map((b) => b.x + branchW + 40)
    )
  );
  const height = Math.max(300, startY + nodeH + gapY + 30 + branchH + 60);

  // Verification info
  const verification = proof?.last_verification;
  const attemptNum = proof?.attempt_number || proof?.attemptNumber || 0;
  const scratchPath = proof?.scratch_path || proof?.scratchPath || "";

  return h`
    <div className="graph-canvas">
      <div className="graph-info">
        <span>Phase: <strong>${proof?.phase || "idle"}</strong></span>
        <span>\u00a0\u00b7\u00a0 Nodes: ${nodes.length}</span>
        <span>\u00a0\u00b7\u00a0 Branches: ${branches.length}</span>
        <span>\u00a0\u00b7\u00a0 Attempts: ${attemptNum}</span>
        ${scratchPath ? h`<span>\u00a0\u00b7\u00a0 <code style=${{fontSize:"10px"}}>${scratchPath}</code></span>` : null}
      </div>
      <svg width=${width} height=${height} xmlns="http://www.w3.org/2000/svg">
        <!-- Parent-child edges in proof tree -->
        ${posNodes.filter((n) => n.parent_id || n.parentId).map((n) => {
          const parent = posNodes.find((p) => p.id === (n.parent_id || n.parentId));
          if (!parent) return null;
          return h`<line key=${"tree-" + n.id}
            x1=${parent.x + nodeW / 2} y1=${parent.y + nodeH}
            x2=${n.x + nodeW / 2} y2=${n.y}
            stroke="#3b82f6" strokeWidth="2" />`;
        })}
        <!-- Dependency edges (dashed) -->
        ${posNodes.flatMap((n) => (n.depends_on || n.dependsOn || []).map((depId) => {
          const dep = posNodes.find((p) => p.id === depId);
          if (!dep) return null;
          return h`<line key=${"dep-" + n.id + "-" + depId}
            x1=${dep.x + nodeW / 2} y1=${dep.y + nodeH}
            x2=${n.x + nodeW / 2} y2=${n.y}
            stroke="#6b7280" strokeWidth="1" strokeDasharray="4,3" />`;
        }))}
        <!-- Edges from nodes to agent branches -->
        ${posBranches.map((b, i) => h`
          <line key=${"edge" + i}
            x1=${b.parentX} y1=${b.parentY}
            x2=${b.x + branchW / 2} y2=${b.y}
            stroke=${roleColor(b.role)} strokeWidth="1.5" strokeDasharray="4,3" opacity="0.4" />
        `)}

        <!-- Proof nodes (top row) -->
        ${posNodes.map((n) => h`
          <g key=${n.id}>
            <rect x=${n.x} y=${n.y} width=${nodeW} height=${nodeH}
              rx="8" fill="#1a1a1a" stroke=${statusColor(n.status)} strokeWidth="2.5" />
            <text x=${n.x + nodeW / 2} y=${n.y + 18} textAnchor="middle"
              fill="#e5e5e5" fontSize="12" fontWeight="700" fontFamily="system-ui">
              ${n.label.length > 18 ? n.label.slice(0, 16) + ".." : n.label}
            </text>
            <text x=${n.x + nodeW / 2} y=${n.y + 33} textAnchor="middle"
              fill="#a3a3a3" fontSize="10" fontFamily="system-ui">
              ${n.kind || "node"} \u00b7 ${n.status}
            </text>
            <text x=${n.x + nodeW / 2} y=${n.y + 45} textAnchor="middle"
              fill="#525252" fontSize="8" fontFamily="monospace">
              ${(n.statement || "").slice(0, 30)}${(n.statement || "").length > 30 ? ".." : ""}
            </text>
          </g>
        `)}

        <!-- Branches (below nodes) -->
        ${posBranches.map((b) => {
          const hasSnippet = !!(b.lean_snippet || b.leanSnippet || "").trim();
          return h`
            <g key=${b.id}>
              <rect x=${b.x} y=${b.y} width=${branchW} height=${branchH}
                rx="5" fill="#111" stroke=${roleColor(b.role)} strokeWidth="1.5"
                opacity=${hasSnippet ? 1 : 0.6} />
              <text x=${b.x + branchW / 2} y=${b.y + 14} textAnchor="middle"
                fill=${roleColor(b.role)} fontSize="9" fontWeight="600" fontFamily="system-ui">
                ${b.role}${b.hidden ? " (hidden)" : ""}
              </text>
              <text x=${b.x + branchW / 2} y=${b.y + 26} textAnchor="middle"
                fill="#737373" fontSize="8" fontFamily="system-ui">
                ${String(b.status || "idle")} \u00b7 score ${(b.score || 0).toFixed(0)} \u00b7 ${b.attempt_count || b.attemptCount || 0} tries
              </text>
              ${hasSnippet ? h`
                <circle cx=${b.x + branchW - 8} cy=${b.y + 8} r="4"
                  fill="#22c55e" opacity="0.8" />
              ` : null}
            </g>
          `;
        })}

        <!-- Verification result banner -->
        ${verification ? h`
          <rect x=${startX} y=${height - 40} width=${width - 60} height="28"
            rx="5" fill=${verification.ok ? "#052e16" : "#450a0a"}
            stroke=${verification.ok ? "#22c55e" : "#ef4444"} strokeWidth="1" />
          <text x=${startX + 12} y=${height - 22}
            fill=${verification.ok ? "#86efac" : "#fca5a5"} fontSize="11" fontFamily="system-ui">
            ${verification.ok ? "Lean verified" : "Lean failed"}: ${
              verification.ok ? "Proof accepted"
                : (verification.stderr || "").split("\\n")[0].slice(0, 80)
            }
          </text>
        ` : null}
      </svg>
    </div>
  `;
}

// ── Paper Tab ───────────────────────────────────────────────────────────

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
