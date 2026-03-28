import React, { useMemo, useState } from "https://esm.sh/react@18.3.1";
import htm from "https://esm.sh/htm@3.1.1";
import { ReactFlow, Background, Controls, MiniMap, Handle, Position } from "https://esm.sh/@xyflow/react@12.6.0?deps=react@18.3.1,react-dom@18.3.1";

const h = htm.bind(React.createElement);

// ── Color helpers ──────────────────────────────────────────────────────

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

// ── Node components ────────────────────────────────────────────────────

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

// ── GraphTab ───────────────────────────────────────────────────────────

export function GraphTab({ session }) {
  const proof = session?.proof;
  const proofNodes = proof?.nodes || [];
  const allBranches = proof?.branches || [];
  const [showBranches, setShowBranches] = useState(false);
  const branches = showBranches ? allBranches : [];

  if (proofNodes.length === 0 && allBranches.length === 0) {
    return h`<div className="graph-container">No proof nodes to visualize</div>`;
  }

  const { flowNodes, flowEdges } = useMemo(() => {
    const nodes = [];
    const edges = [];
    const hasTree = proofNodes.some(n => n.parent_id || n.parentId);

    if (hasTree) {
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
        edges.push({
          id: "flow-" + n.id, source: prevId, target: n.id,
          style: { stroke: "#333", strokeWidth: 1 },
          type: "smoothstep",
        });
      }
      prevId = n.id;

      for (const depId of (n.depends_on || n.dependsOn || [])) {
        edges.push({
          id: "dep-" + n.id + "-" + depId, source: depId,
          target: n.id,
          style: { stroke: "#6b7280", strokeWidth: 1, strokeDasharray: "4 3" },
        });
      }
    }

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
        id: bId, type: "branchNode",
        position: { x: baseX + col * 160, y: branchY },
        draggable: true,
        data: { ...b, _nodeColor: roleColor(b.role) },
      });
      if (focusId) {
        edges.push({
          id: "agent-" + b.id, source: focusId, target: bId,
          style: { stroke: roleColor(b.role), strokeWidth: 1, strokeDasharray: "4 3", opacity: 0.5 },
        });
      }
    }

    const proofGoals = proof?.proof_goals || proof?.proofGoals || [];
    const goalY = (maxDepth + 1) * 100 + (branches.length > 0 ? 180 : 40);
    for (let i = 0; i < proofGoals.length; i++) {
      const g = proofGoals[i];
      const gId = "goal-" + g.id;
      nodes.push({
        id: gId, type: "goalNode",
        position: { x: i * 200 - (proofGoals.length * 100), y: goalY + (g.parent_goal_id || g.parentGoalId ? 80 : 0) },
        draggable: true, data: g,
      });
      const parentGoalId = g.parent_goal_id || g.parentGoalId;
      if (parentGoalId) {
        edges.push({
          id: "goaltree-" + g.id, source: "goal-" + parentGoalId, target: gId,
          style: { stroke: "#3b82f6", strokeWidth: 1, strokeDasharray: "2 2" },
          label: g.tactic_applied || g.tacticApplied || "",
          labelStyle: { fill: "#22c55e", fontSize: 8 },
        });
      }
      if (!parentGoalId && proofNodes.length > 0) {
        const activeNodeId = proof?.active_node_id || proof?.activeNodeId || proofNodes[0]?.id;
        edges.push({
          id: "nodegoal-" + g.id, source: activeNodeId, target: gId,
          style: { stroke: "#f59e0b", strokeWidth: 1, strokeDasharray: "3 3", opacity: 0.6 },
        });
      }
    }

    return { flowNodes: nodes, flowEdges: edges };
  }, [proofNodes, branches, proof]);

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
