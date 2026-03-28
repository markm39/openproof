import React, { useMemo, useState } from "https://esm.sh/react@18.3.1";
import htm from "https://esm.sh/htm@3.1.1";
import { ReactFlow, Background, Controls, MiniMap, Handle, Position } from "https://esm.sh/@xyflow/react@12.6.0?deps=react@18.3.1,react-dom@18.3.1";

const h = htm.bind(React.createElement);

// ── Colors ─────────────────────────────────────────────────────────────

const STATUS_COLORS = {
  open: "#f59e0b",
  in_progress: "#3b82f6",
  closed: "#22c55e",
  failed: "#ef4444",
};

const ATTRIBUTION_COLORS = {
  "agent:prover": "#3b82f6",
  "agent:repairer": "#f59e0b",
  "agent:planner": "#8b5cf6",
  "agent:retriever": "#06b6d4",
  "agent:critic": "#ec4899",
  bfs: "#22c55e",
};

const STATUS_ICONS = {
  open: "\u25CB",
  in_progress: "\u25D4",
  closed: "\u2713",
  failed: "\u2717",
};

// ── Tree layout ────────────────────────────────────────────────────────

const H_GAP = 240;
const V_GAP = 120;

function computeTreeLayout(goals) {
  const childrenMap = new Map();
  const goalMap = new Map();
  const roots = [];

  for (const g of goals) {
    goalMap.set(g.id, g);
    const pid = g.parent_goal_id || g.parentGoalId;
    if (pid) {
      if (!childrenMap.has(pid)) childrenMap.set(pid, []);
      childrenMap.get(pid).push(g.id);
    } else {
      roots.push(g.id);
    }
  }

  // Compute subtree widths (leaf = 1)
  const widths = new Map();
  function subtreeWidth(id) {
    if (widths.has(id)) return widths.get(id);
    const ch = childrenMap.get(id) || [];
    const w = ch.length === 0 ? 1 : ch.reduce((sum, c) => sum + subtreeWidth(c), 0);
    widths.set(id, w);
    return w;
  }
  for (const r of roots) subtreeWidth(r);

  // Assign positions
  const positions = new Map();
  function layout(id, x, depth) {
    positions.set(id, { x, y: depth * V_GAP });
    const ch = childrenMap.get(id) || [];
    if (ch.length === 0) return;
    const totalW = ch.reduce((s, c) => s + subtreeWidth(c), 0);
    let curX = x - ((totalW - 1) * H_GAP) / 2;
    for (const c of ch) {
      const w = subtreeWidth(c);
      layout(c, curX + ((w - 1) * H_GAP) / 2, depth + 1);
      curX += w * H_GAP;
    }
  }

  // Layout each root with horizontal offset between them
  let rootX = 0;
  for (const r of roots) {
    const w = subtreeWidth(r);
    layout(r, rootX + ((w - 1) * H_GAP) / 2, 0);
    rootX += w * H_GAP + H_GAP;
  }

  return { positions, childrenMap };
}

// ── GoalNode component ─────────────────────────────────────────────────

function GoalNodeComponent({ data }) {
  const [expanded, setExpanded] = useState(false);
  const status = String(data.status || "open").toLowerCase();
  const color = STATUS_COLORS[status] || "#737373";
  const isLeaf = !data._hasChildren;
  const isFrontier = isLeaf && status === "open";
  const isActive = status === "in_progress";
  const attribution = data.solved_by || data.solvedBy || (status === "closed" ? "bfs" : null);
  const attrColor = attribution ? (ATTRIBUTION_COLORS[attribution] || "#737373") : null;

  const goalText = data.goal_text || data.goalText || "";
  const failedTactics = data.failed_tactics || data.failedTactics || [];
  const attempts = data.attempts || 0;

  return h`
    <div onClick=${() => setExpanded(!expanded)} style=${{
      background: "#0f0f0f",
      border: "2px solid " + color,
      borderRadius: 8,
      padding: "8px 12px",
      minWidth: 180,
      maxWidth: expanded ? 400 : 260,
      fontFamily: "system-ui, sans-serif",
      cursor: "pointer",
      transition: "all 0.2s",
      animation: isFrontier ? "frontier-pulse 2s infinite" : isActive ? "active-pulse 1.5s infinite" : "none",
    }}>
      <${Handle} type="target" position=${Position.Top} style=${{ background: color }} />

      <div style=${{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
        <span style=${{ color, fontSize: 13, fontWeight: 700 }}>
          ${STATUS_ICONS[status] || "\u25CB"}
        </span>
        <span style=${{ color: "#a3a3a3", fontSize: 10, fontWeight: 500 }}>
          ${status}
        </span>
        ${attribution ? h`
          <span style=${{
            background: attrColor + "22",
            color: attrColor,
            fontSize: 8,
            fontWeight: 600,
            padding: "1px 6px",
            borderRadius: 3,
            marginLeft: "auto",
          }}>${attribution}</span>
        ` : null}
        ${isFrontier ? h`
          <span style=${{
            background: "#78350f",
            color: "#fbbf24",
            fontSize: 8,
            fontWeight: 600,
            padding: "1px 6px",
            borderRadius: 3,
            marginLeft: attribution ? 0 : "auto",
          }}>frontier</span>
        ` : null}
      </div>

      <div style=${{
        color: "#d4d4d4",
        fontSize: 10,
        fontFamily: "monospace",
        overflow: expanded ? "auto" : "hidden",
        textOverflow: expanded ? "unset" : "ellipsis",
        whiteSpace: expanded ? "pre-wrap" : "nowrap",
        maxWidth: expanded ? 380 : 240,
        maxHeight: expanded ? 200 : "1.4em",
        lineHeight: "1.4",
      }}>
        ${goalText || "(empty goal)"}
      </div>

      ${expanded && failedTactics.length > 0 ? h`
        <div style=${{ marginTop: 6, fontSize: 9, color: "#ef4444" }}>
          <div style=${{ fontWeight: 600, marginBottom: 2 }}>Failed tactics (${failedTactics.length}):</div>
          ${failedTactics.slice(0, 5).map((t, i) => h`
            <div key=${i} style=${{ color: "#a3a3a3", fontFamily: "monospace", paddingLeft: 8 }}>${t}</div>
          `)}
          ${failedTactics.length > 5 ? h`<div style=${{ color: "#525252", paddingLeft: 8 }}>...and ${failedTactics.length - 5} more</div>` : null}
        </div>
      ` : null}

      ${expanded && attempts > 0 ? h`
        <div style=${{ marginTop: 4, fontSize: 9, color: "#737373" }}>
          ${attempts} tactic${attempts !== 1 ? "s" : ""} attempted
        </div>
      ` : null}

      <${Handle} type="source" position=${Position.Bottom} style=${{ background: color }} />
    </div>
  `;
}

const nodeTypes = { goalNode: GoalNodeComponent };

// ── Stats bar ──────────────────────────────────────────────────────────

function StatsBar({ stats, proof }) {
  if (!stats) return null;
  const verification = proof?.last_verification;
  return h`
    <div className="graph-info">
      <span>Phase: <strong>${proof?.phase || "idle"}</strong></span>
      <span>\u00a0\u00b7\u00a0 Goals: ${stats.total}</span>
      <span>\u00a0\u00b7\u00a0 Closed: <span style=${{ color: "#22c55e" }}>${stats.closed}</span></span>
      <span>\u00a0\u00b7\u00a0 Open: <span style=${{ color: "#f59e0b" }}>${stats.open}</span></span>
      ${stats.failed > 0 ? h`<span>\u00a0\u00b7\u00a0 Failed: <span style=${{ color: "#ef4444" }}>${stats.failed}</span></span>` : null}
      ${stats.frontier > 0 ? h`<span>\u00a0\u00b7\u00a0 BFS frontier: <span style=${{ color: "#fbbf24" }}>${stats.frontier}</span></span>` : null}
      ${verification ? h`
        <span>\u00a0\u00b7\u00a0
          <span style=${{ color: verification.ok ? "#22c55e" : "#ef4444" }}>
            ${verification.ok ? "Lean verified" : "Lean failed"}
          </span>
        </span>
      ` : null}
    </div>
  `;
}

// ── GraphTab ───────────────────────────────────────────────────────────

export function GraphTab({ session }) {
  const proof = session?.proof;
  const goals = proof?.proof_goals || proof?.proofGoals || [];

  const { flowNodes, flowEdges, stats } = useMemo(() => {
    if (goals.length === 0) return { flowNodes: [], flowEdges: [], stats: null };

    const { positions, childrenMap } = computeTreeLayout(goals);

    const nodes = goals.map((g) => ({
      id: g.id,
      type: "goalNode",
      position: positions.get(g.id) || { x: 0, y: 0 },
      draggable: true,
      data: {
        ...g,
        _hasChildren: childrenMap.has(g.id) && childrenMap.get(g.id).length > 0,
        _nodeColor: STATUS_COLORS[String(g.status || "open").toLowerCase()] || "#737373",
      },
    }));

    const edges = goals
      .filter((g) => g.parent_goal_id || g.parentGoalId)
      .map((g) => {
        const tactic = g.tactic_applied || g.tacticApplied || "";
        const st = String(g.status || "open").toLowerCase();
        return {
          id: "tactic-" + g.id,
          source: g.parent_goal_id || g.parentGoalId,
          target: g.id,
          label: tactic.length > 40 ? tactic.substring(0, 37) + "..." : tactic,
          labelStyle: { fill: "#22c55e", fontSize: 9, fontFamily: "monospace" },
          style: { stroke: STATUS_COLORS[st] || "#525252", strokeWidth: 1.5 },
          type: "smoothstep",
          animated: st === "in_progress",
        };
      });

    // Compute stats
    let closed = 0, open = 0, failed = 0, inProgress = 0, frontier = 0;
    for (const g of goals) {
      const s = String(g.status || "open").toLowerCase();
      if (s === "closed") closed++;
      else if (s === "failed") failed++;
      else if (s === "in_progress") inProgress++;
      else open++;
      const isLeaf = !childrenMap.has(g.id) || childrenMap.get(g.id).length === 0;
      if (s === "open" && isLeaf) frontier++;
    }

    return {
      flowNodes: nodes,
      flowEdges: edges,
      stats: { total: goals.length, closed, open, failed, inProgress, frontier },
    };
  }, [goals]);

  if (goals.length === 0) {
    return h`<div className="graph-container">
      <div style=${{ textAlign: "center", color: "#737373" }}>
        <div style=${{ fontSize: 14, marginBottom: 8 }}>No proof goals yet</div>
        <div style=${{ fontSize: 11 }}>Start a proof to see the tactic tree</div>
      </div>
    </div>`;
  }

  return h`
    <div className="graph-canvas" style=${{ display: "flex", flexDirection: "column", height: "calc(100vh - 120px)" }}>
      <${StatsBar} stats=${stats} proof=${proof} />
      <div style=${{ flex: 1, width: "100%" }}>
        <${ReactFlow}
          nodes=${flowNodes}
          edges=${flowEdges}
          nodeTypes=${nodeTypes}
          fitView
          fitViewOptions=${{ padding: 0.3 }}
          minZoom=${0.1}
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
