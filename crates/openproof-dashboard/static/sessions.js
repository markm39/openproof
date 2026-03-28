import React, { useState, useCallback } from "https://esm.sh/react@18.3.1";
import htm from "https://esm.sh/htm@3.1.1";

const h = htm.bind(React.createElement);

export function SessionSidebar({ sessions, selectedId, onSelect, onChanged }) {
  const [managing, setManaging] = useState(false);
  const [checked, setChecked] = useState(new Set());
  const [editingId, setEditingId] = useState(null);
  const [editTitle, setEditTitle] = useState("");
  const [confirmDelete, setConfirmDelete] = useState(null); // null | {type, ids}

  const toggleCheck = useCallback((id) => {
    setChecked((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const toggleAll = useCallback(() => {
    setChecked((prev) => {
      if (prev.size === sessions.length) return new Set();
      return new Set(sessions.map((s) => s.id));
    });
  }, [sessions]);

  const startRename = useCallback((id, title) => {
    setEditingId(id);
    setEditTitle(title);
  }, []);

  const commitRename = useCallback(async () => {
    if (!editingId || !editTitle.trim()) return;
    try {
      await fetch("/api/session", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: editingId, title: editTitle.trim() }),
      });
      onChanged();
    } catch {}
    setEditingId(null);
    setEditTitle("");
  }, [editingId, editTitle, onChanged]);

  const cancelRename = useCallback(() => {
    setEditingId(null);
    setEditTitle("");
  }, []);

  const requestDelete = useCallback((ids) => {
    setConfirmDelete({ ids });
  }, []);

  const executeDelete = useCallback(async () => {
    if (!confirmDelete) return;
    const { ids } = confirmDelete;
    try {
      if (ids.length === 1) {
        await fetch(`/api/session?id=${encodeURIComponent(ids[0])}`, { method: "DELETE" });
      } else {
        await fetch("/api/sessions/bulk-delete", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ ids }),
        });
      }
      // If we deleted the selected session, clear selection
      if (ids.includes(selectedId)) {
        const remaining = sessions.filter((s) => !ids.includes(s.id));
        onSelect(remaining[0]?.id || null);
      }
      setChecked((prev) => {
        const next = new Set(prev);
        for (const id of ids) next.delete(id);
        return next;
      });
      onChanged();
    } catch {}
    setConfirmDelete(null);
  }, [confirmDelete, sessions, selectedId, onSelect, onChanged]);

  const checkedCount = checked.size;

  return h`
    <div className="sidebar">
      <div className="sidebar-header">
        <span className="sidebar-title" style=${{ borderBottom: "none", flex: 1 }}>Sessions</span>
        <button className="btn-small btn-manage"
          onClick=${() => { setManaging(!managing); setChecked(new Set()); setEditingId(null); }}>
          ${managing ? "Done" : "Manage"}
        </button>
      </div>

      ${managing && sessions.length > 0 ? h`
        <div className="session-toolbar">
          <label className="session-check-label">
            <input type="checkbox"
              checked=${checkedCount === sessions.length && sessions.length > 0}
              onChange=${toggleAll} />
            <span>All</span>
          </label>
          ${checkedCount > 0 ? h`
            <button className="btn-small btn-danger"
              onClick=${() => requestDelete([...checked])}>
              Delete (${checkedCount})
            </button>
          ` : null}
          ${checkedCount > 0 ? h`
            <span className="session-toolbar-count">${checkedCount} selected</span>
          ` : null}
        </div>
      ` : null}

      ${confirmDelete ? h`
        <div className="delete-confirm">
          <div>Delete ${confirmDelete.ids.length} session${confirmDelete.ids.length > 1 ? "s" : ""}?</div>
          <div className="delete-confirm-sub">This cannot be undone.</div>
          <div className="delete-confirm-actions">
            <button className="btn-small" onClick=${() => setConfirmDelete(null)}>Cancel</button>
            <button className="btn-small btn-danger" onClick=${executeDelete}>Delete</button>
          </div>
        </div>
      ` : null}

      <div className="session-list">
        ${sessions.map((s) => h`
          <div key=${s.id}
            className=${`session-item ${selectedId === s.id ? "session-item-active" : ""}`}>
            ${managing ? h`
              <input type="checkbox" className="session-check"
                checked=${checked.has(s.id)}
                onChange=${() => toggleCheck(s.id)} />
            ` : null}
            <div className="session-item-body"
              onClick=${() => editingId !== s.id && onSelect(s.id)}>
              ${editingId === s.id ? h`
                <input className="session-edit-input"
                  value=${editTitle}
                  onInput=${(e) => setEditTitle(e.target.value)}
                  onKeyDown=${(e) => {
                    if (e.key === "Enter") commitRename();
                    if (e.key === "Escape") cancelRename();
                  }}
                  onClick=${(e) => e.stopPropagation()}
                  ref=${(el) => el && el.focus()} />
              ` : h`
                <strong>${s.title}</strong>
              `}
              <small>${s.transcriptEntries || 0} entries \u00b7 ${s.proofNodes || 0} nodes</small>
            </div>
            ${managing && editingId !== s.id ? h`
              <div className="session-item-actions">
                <button className="btn-icon" title="Rename"
                  onClick=${(e) => { e.stopPropagation(); startRename(s.id, s.title); }}>
                  \u270E
                </button>
                <button className="btn-icon btn-icon-danger" title="Delete"
                  onClick=${(e) => { e.stopPropagation(); requestDelete([s.id]); }}>
                  \u2715
                </button>
              </div>
            ` : null}
            ${editingId === s.id ? h`
              <div className="session-item-actions">
                <button className="btn-icon" title="Save" onClick=${commitRename}>\u2713</button>
                <button className="btn-icon" title="Cancel" onClick=${cancelRename}>\u2715</button>
              </div>
            ` : null}
          </div>
        `)}
      </div>
    </div>
  `;
}
