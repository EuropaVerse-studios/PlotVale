// Binder: the project outline. Collapse, double-click to open, right-click
// menu, HTML5 drag-and-drop reorder, inline rename, "+ Add" kind picker.

import { memo, useState } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import type { NodeDto, NodeKind } from "../lib/types";
import { canHaveText, isSystemKind } from "../lib/types";

const KIND_ICON: Record<NodeKind, string> = {
  draft_root: "📖",
  folder: "📁",
  scene: "📄",
  note: "📝",
  research_folder: "🧭",
  research_item: "🔖",
  character_sheet: "👤",
  location_sheet: "📍",
  trash: "🗑️",
};

const STATUS_DOT: Record<string, string> = {
  todo: "○",
  draft: "●",
  revised: "◆",
  final: "★",
};

const STATUS_CLASS: Record<string, string> = {
  todo: "st-todo",
  draft: "st-draft",
  revised: "st-revised",
  final: "st-final",
};

function Binder() {
  const { t } = useLocale();
  const store = useStore();
  const { nodes, draftRoot, researchRoot, trashId } = store;
  const [adding, setAdding] = useState(false);
  const [newName, setNewName] = useState("");

  if (!store.project) return null;

  async function createHere(kind: string) {
    const parent = store.selection ?? draftRoot;
    const title = newName.trim() === "" ? t(kindTitleKey(kind)) : newName.trim();
    setNewName("");
    setAdding(false);
    try {
      await store.createNode(parent, kind, title);
    } catch (e) {
      store.setStatus(`Error: ${e instanceof Error ? e.message : e}`);
    }
  }

  return (
    <aside className="binder">
      <div className="panel-head">
        <h2>{t("binder")}</h2>
        <div className="add-wrap">
          <button className="btn small" onClick={() => setAdding((v) => !v)}>
            + {t("dlg_new_title")}
          </button>
          {adding && (
            <div className="add-drop">
              <input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder={t("dlg_new_name")}
              />
              {[
                ["scene", t("menu_new_scene")],
                ["folder", t("menu_new_folder")],
                ["note", t("menu_new_note")],
                ["research_item", t("ctx_new_research")],
                ["character", t("ctx_new_character")],
                ["location", t("ctx_new_location")],
              ].map(([kind, label]) => (
                <button key={kind} className="menu-item" onClick={() => void createHere(kind)}>
                  {label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
      <div className="tree">
        {[draftRoot, researchRoot, trashId].map(
          (root) => nodes.get(root) && <NodeRow key={root} id={root} depth={0} />,
        )}
      </div>
    </aside>
  );
}

function kindTitleKey(kind: string): string {
  switch (kind) {
    case "scene":
      return "scene_label";
    case "folder":
      return "chapter_label";
    case "note":
      return "kind_note";
    case "research_item":
      return "kind_research_item";
    case "character":
      return "kind_character";
    case "location":
      return "kind_location";
    default:
      return "untitled";
  }
}

const NodeRow = memo(function NodeRow({ id, depth }: { id: string; depth: number }) {
  const store = useStore();
  const node = store.nodes.get(id);
  const [renaming, setRenaming] = useState(false);
  const [renameBuf, setRenameBuf] = useState("");
  const [ctx, setCtx] = useState<{ x: number; y: number } | null>(null);

  if (!node) return null;
  const selected = store.selection === id;
  const collapsed = store.collapsed.has(id);
  const hasChildren = node.children.length > 0;
  const inTrash = isInTrash(store.nodes, store.trashId, id);

  function open() {
    store.setSelection(id);
    if (canHaveText(node!.kind)) {
      store.setOpenDoc(id);
      store.setCenterTab("editor");
    } else if (hasChildren) {
      store.toggleCollapsed(id);
    }
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault();
    const dragged = e.dataTransfer.getData("text/plotvale-id");
    if (!dragged || dragged === id) return;
    const rect = (e.target as HTMLElement).closest(".node-row")?.getBoundingClientRect();
    const upper = rect ? e.clientY < rect.top + rect.height / 3 : false;
    if (upper && node!.parent) {
      const siblings = store.nodes.get(node!.parent)?.children ?? [];
      void store.moveNode(dragged, node!.parent, siblings.indexOf(id));
    } else if (id === store.trashId) {
      void store.trashNode(dragged);
    } else {
      const pos = (store.nodes.get(id)?.children.length ?? 0);
      // Dropping onto trashed items or from trash uses restore semantics instead.
      if (inTrash) return;
      void store.moveNode(dragged, id, pos).catch(() => {});
    }
  }

  return (
    <div>
      <div
        className={selected ? "node-row selected" : "node-row"}
        style={{ paddingLeft: 6 + depth * 16 }}
        draggable={!isSystemKind(node.kind)}
        onDragStart={(e) => e.dataTransfer.setData("text/plotvale-id", id)}
        onDragOver={(e) => e.preventDefault()}
        onDrop={onDrop}
        onClick={open}
        onDoubleClick={open}
        onContextMenu={(e) => {
          e.preventDefault();
          setCtx({ x: e.clientX, y: e.clientY });
        }}
      >
        {hasChildren ? (
          <button
            className="twisty"
            onClick={(e) => {
              e.stopPropagation();
              store.toggleCollapsed(id);
            }}
          >
            {collapsed ? "▸" : "▾"}
          </button>
        ) : (
          <span className="twisty" />
        )}
        <span className={`status-dot ${STATUS_CLASS[node.meta.status] ?? "st-todo"}`}>
          {STATUS_DOT[node.meta.status] ?? "○"}
        </span>
        <span className="kind-icon">{KIND_ICON[node.kind]}</span>
        {renaming ? (
          <input
            autoFocus
            value={renameBuf}
            onChange={(e) => setRenameBuf(e.target.value)}
            onBlur={() => setRenaming(false)}
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => {
              if (e.key === "Enter" && renameBuf.trim() !== "") {
                void store.renameNode(id, renameBuf.trim());
                setRenaming(false);
              } else if (e.key === "Escape") {
                setRenaming(false);
              }
            }}
          />
        ) : (
          <span className="node-title">{node.title}</span>
        )}
      </div>
      {ctx && (
        <CtxMenu
          x={ctx.x}
          y={ctx.y}
          node={node}
          inTrash={inTrash}
          onClose={() => setCtx(null)}
          onRename={() => {
            setRenameBuf(node.title);
            setRenaming(true);
          }}
        />
      )}
      {hasChildren && !collapsed && (
        <div>
          {node.children.map((child) => (
            <NodeRow key={child} id={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
});

function CtxMenu({
  x,
  y,
  node,
  inTrash,
  onClose,
  onRename,
}: {
  x: number;
  y: number;
  node: NodeDto;
  inTrash: boolean;
  onClose: () => void;
  onRename: () => void;
}) {
  const { t } = useLocale();
  const store = useStore();
  const id = node.id;

  function act(fn: () => Promise<unknown>) {
    onClose();
    void fn().catch((e: unknown) =>
      store.setStatus(`Error: ${e instanceof Error ? e.message : e}`),
    );
  }

  return (
    <>
      <div className="menu-scrim" onClick={onClose} onContextMenu={onClose} />
      <div className="ctx-menu" style={{ left: Math.min(x, window.innerWidth - 220), top: Math.min(y, window.innerHeight - 320) }}>
        {!inTrash && (
          <>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "scene", t("scene_label")))}>
              {t("ctx_new_scene")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "folder", t("chapter_label")))}>
              {t("ctx_new_folder")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "note", t("kind_note")))}>
              {t("ctx_new_note")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "research_item", t("kind_research_item")))}>
              {t("ctx_new_research")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "character", t("kind_character")))}>
              {t("ctx_new_character")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.createNode(id, "location", t("kind_location")))}>
              {t("ctx_new_location")}
            </button>
            <div className="menu-sep" />
          </>
        )}
        {!isSystemKind(node.kind) && (
          <button
            className="menu-item"
            onClick={() => {
              onRename();
              onClose();
            }}
          >
            {t("ctx_rename")}
          </button>
        )}
        {!isSystemKind(node.kind) && !inTrash && (
          <>
            <button className="menu-item" onClick={() => act(() => store.reorderNode(id, true))}>
              {t("ctx_move_up")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.reorderNode(id, false))}>
              {t("ctx_move_down")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.indentNode(id))}>
              {t("ctx_indent")}
            </button>
            <button className="menu-item" onClick={() => act(() => store.outdentNode(id))}>
              {t("ctx_outdent")}
            </button>
            <div className="menu-sep" />
            <button className="menu-item danger" onClick={() => act(() => store.trashNode(id))}>
              {t("ctx_trash")}
            </button>
          </>
        )}
        {inTrash && !isSystemKind(node.kind) && (
          <button className="menu-item" onClick={() => act(() => store.restoreNode(id))}>
            {t("ctx_restore")}
          </button>
        )}
        {(inTrash || node.kind === "trash") && (
          <>
            <button className="menu-item danger" onClick={() => act(() => store.deleteNode(id))}>
              {t("ctx_delete_forever")}
            </button>
          </>
        )}
        {node.kind === "trash" && (
          <button className="menu-item danger" onClick={() => act(() => store.emptyTrash())}>
            {t("ctx_empty_trash")}
          </button>
        )}
      </div>
    </>
  );
}

function isInTrash(nodes: Map<string, NodeDto>, trashId: string, id: string): boolean {
  let cur: string | null | undefined = id;
  while (cur) {
    if (cur === trashId) return true;
    cur = nodes.get(cur)?.parent;
  }
  return false;
}

export default memo(Binder);
