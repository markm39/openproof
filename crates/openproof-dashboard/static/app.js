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

  if (nodes.length === 0) {
    return h`<div className="graph-container">No proof nodes to visualize</div>`;
  }

  // Simple SVG tree layout
  const width = Math.max(600, nodes.length * 160);
  const height = 400;
  const nodeW = 120;
  const nodeH = 40;
  const gapX = 160;
  const startX = 40;
  const startY = 40;

  const positioned = nodes.map((n, i) => ({
    ...n,
    x: startX + i * gapX,
    y: startY + (n.kind === "theorem" ? 0 : n.kind === "lemma" ? 100 : 200),
  }));

  const statusColor = (s) => {
    if (s === "verified") return "#22c55e";
    if (s === "proving") return "#eab308";
    if (s === "failed") return "#ef4444";
    return "#525252";
  };

  return h`
    <div className="graph-canvas">
      <svg width=${width} height=${height} xmlns="http://www.w3.org/2000/svg">
        ${positioned.map((n, i) => {
          if (i > 0) {
            const prev = positioned[i - 1];
            return h`<line key=${"e" + i}
              x1=${prev.x + nodeW / 2} y1=${prev.y + nodeH}
              x2=${n.x + nodeW / 2} y2=${n.y}
              stroke="#333" strokeWidth="1.5" />`;
          }
          return null;
        })}
        ${positioned.map((n) => h`
          <g key=${n.id}>
            <rect x=${n.x} y=${n.y} width=${nodeW} height=${nodeH}
              rx="6" fill="#1a1a1a" stroke=${statusColor(n.status)} strokeWidth="2" />
            <text x=${n.x + nodeW / 2} y=${n.y + 16} textAnchor="middle"
              fill="#e5e5e5" fontSize="11" fontWeight="600" fontFamily="var(--sans)">
              ${n.label.length > 14 ? n.label.slice(0, 12) + ".." : n.label}
            </text>
            <text x=${n.x + nodeW / 2} y=${n.y + 30} textAnchor="middle"
              fill="#737373" fontSize="9" fontFamily="var(--mono)">
              ${n.kind} \u00b7 ${n.status}
            </text>
          </g>
        `)}
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
