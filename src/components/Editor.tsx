// Editor: plain textarea (native speed on 150k-word scenes) with a
// Markdown formatting toolbar, in-document find, and scrivenings mode.
// Saved text stays pure Markdown, like the classic UI.

import { memo, useEffect, useRef, useState } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { countWords } from "../lib/count";
import type { NodeDto } from "../lib/types";

/** Above this size the find bar warns the doc is shown plain and fast. */
const LARGE_DOC_CHARS = 100_000;

function Editor() {
  const store = useStore();
  const { nodes, selection, openDoc, scrivenings, project } = store;

  if (!project) return <div className="center-pane" />;

  if (scrivenings && selection) {
    const node = nodes.get(selection);
    const isContainer =
      node &&
      (node.kind === "draft_root" || node.kind === "folder" || node.kind === "research_folder");
    if (isContainer) {
      const ids = collectTexts(nodes, selection);
      if (ids.length === 0) return <div className="center-pane empty">{""}</div>;
      return (
        <div className="center-pane scriv">
          {ids.map((id) => (
            <DocEditor key={id} id={id} showTitle />
          ))}
        </div>
      );
    }
  }

  if (!openDoc) {
    return (
      <div className="center-pane empty">
        <EditorHint />
      </div>
    );
  }
  return (
    <div className="center-pane">
      <DocEditor key={`${openDoc}`} id={openDoc} showTitle={false} />
    </div>
  );
}

function EditorHint() {
  const { t } = useLocale();
  return <p className="hint">{t("editor_hint")}</p>;
}

function collectTexts(nodes: Map<string, NodeDto>, root: string): string[] {
  const out: string[] = [];
  const visit = (id: string): void => {
    const node = nodes.get(id);
    if (!node) return;
    if (
      node.kind === "scene" ||
      node.kind === "note" ||
      node.kind === "research_item" ||
      node.kind === "character_sheet" ||
      node.kind === "location_sheet"
    ) {
      out.push(id);
    }
    for (const child of node.children) visit(child);
  };
  for (const child of nodes.get(root)?.children ?? []) visit(child);
  return out;
}

const DocEditor = memo(function DocEditor({ id, showTitle }: { id: string; showTitle: boolean }) {
  const { t } = useLocale();
  const store = useStore();
  const node = store.nodes.get(id);
  const backendText = store.texts[id] ?? "";
  const rev = store.textRev[id] ?? 0;
  const [draft, setDraft] = useState(backendText);
  const [finding, setFinding] = useState(false);
  const [query, setQuery] = useState("");
  const areaRef = useRef<HTMLTextAreaElement>(null);
  const lastRev = useRef(rev);
  const lastId = useRef(id);

  // Resets the draft when switching documents or on external change
  // (snapshot restore, open). Keystrokes never reset the caret.
  useEffect(() => {
    if (lastId.current !== id || lastRev.current !== rev) {
      lastId.current = id;
      lastRev.current = rev;
      setDraft(backendText);
    }
  }, [id, rev]); // keystrokes only touch local draft + store text

  if (!node) return null;
  const words = countWords(draft).words;
  const large = draft.length > LARGE_DOC_CHARS;

  function commit(value: string) {
    setDraft(value);
    store.setText(id, value);
  }

  function wrapSelection(before: string, after: string) {
    const area = areaRef.current;
    if (!area) return;
    const { selectionStart: s, selectionEnd: e, value } = area;
    const selected = value.slice(s, e);
    const next = `${value.slice(0, s)}${before}${selected}${after}${value.slice(e)}`;
    commit(next);
    requestAnimationFrame(() => {
      area.focus();
      const caret = selected === "" ? s + before.length : s + before.length + selected.length + after.length;
      area.setSelectionRange(caret, caret);
    });
  }

  function linePrefix(prefix: string) {
    const area = areaRef.current;
    if (!area) return;
    const { selectionStart: s, value } = area;
    const ls = value.lastIndexOf("\n", s - 1) + 1;
    const le = value.indexOf("\n", s);
    const end = le === -1 ? value.length : le;
    const line = value.slice(ls, end);
    const next = line.startsWith(prefix)
      ? value.slice(0, ls) + line.slice(prefix.length) + value.slice(end)
      : value.slice(0, ls) + prefix + line + value.slice(end);
    commit(next);
    requestAnimationFrame(() => {
      area.focus();
      area.setSelectionRange(s + (line.startsWith(prefix) ? -prefix.length : prefix.length), s + (line.startsWith(prefix) ? -prefix.length : prefix.length));
    });
  }

  function find(dir: 1 | -1) {
    const area = areaRef.current;
    if (!area || query === "") return;
    const from = dir === 1 ? area.selectionEnd : area.selectionStart - query.length - 1;
    let pos = dir === 1 ? draft.indexOf(query, from) : draft.lastIndexOf(query, Math.max(0, from));
    if (pos === -1 && dir === 1) pos = draft.indexOf(query, 0);
    if (pos === -1 && dir === -1) pos = draft.lastIndexOf(query);
    if (pos !== -1) {
      area.focus();
      area.setSelectionRange(pos, pos + query.length);
      area.scrollTop = Math.max(0, area.scrollTop);
    }
  }

  function onKey(e: React.KeyboardEvent) {
    if (e.ctrlKey && !e.shiftKey && (e.key === "b" || e.key === "B")) {
      e.preventDefault();
      wrapSelection("**", "**");
    } else if (e.ctrlKey && !e.shiftKey && (e.key === "i" || e.key === "I")) {
      e.preventDefault();
      wrapSelection("*", "*");
    } else if (e.ctrlKey && e.key === "f") {
      e.preventDefault();
      setFinding(true);
    }
  }

  return (
    <section className="doc">
      {showTitle && (
        <button
          className="scriv-title"
          onClick={() => {
            store.setSelection(id);
            store.setOpenDoc(id);
          }}
        >
          ▾ {node.title}
        </button>
      )}
      <header className="doc-head">
        <h3>{node.title}</h3>
        <span className="small">
          {words} {t("status_words")}
        </span>
        {large && <span className="badge">{t("editor_large_plain")}</span>}
      </header>
      <div className="toolbar" role="toolbar">
        <ToolBtn tip={t("fmt_b")} label="B" onClick={() => wrapSelection("**", "**")} />
        <ToolBtn tip={t("fmt_i")} label="I" italic onClick={() => wrapSelection("*", "*")} />
        <ToolBtn tip={t("fmt_code")} label="<>" onClick={() => wrapSelection("`", "`")} />
        <ToolBtn tip={t("fmt_h1")} label="H1" onClick={() => linePrefix("# ")} />
        <ToolBtn tip={t("fmt_h2")} label="H2" onClick={() => linePrefix("## ")} />
        <ToolBtn tip={t("fmt_h3")} label="H3" onClick={() => linePrefix("### ")} />
        <ToolBtn tip={t("fmt_quote")} label="❝" onClick={() => linePrefix("> ")} />
        <ToolBtn tip={t("fmt_list")} label="•" onClick={() => linePrefix("- ")} />
        <ToolBtn tip={t("fmt_note")} label="[[]]" onClick={() => wrapSelection("[[note: ", "]]")} />
        <span className="spacer" />
        <ToolBtn tip={t("fmt_size_down")} label="A−" onClick={() => store.setFontSize(Math.max(12, store.fontSize - 1))} />
        <ToolBtn tip={t("fmt_size_up")} label="A+" onClick={() => store.setFontSize(Math.min(28, store.fontSize + 1))} />
        <ToolBtn tip={t("editor_find")} label="🔍" onClick={() => setFinding((v) => !v)} />
      </div>
      {finding && (
        <div className="findbar">
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") find(e.shiftKey ? -1 : 1);
              if (e.key === "Escape") setFinding(false);
            }}
            placeholder={t("editor_find")}
          />
          <button className="btn small" onClick={() => find(-1)}>
            {t("editor_find_prev")}
          </button>
          <button className="btn small" onClick={() => find(1)}>
            {t("editor_find_next")}
          </button>
        </div>
      )}
      <textarea
        ref={areaRef}
        className="editor"
        value={draft}
        style={{ fontSize: store.fontSize }}
        onChange={(e) => commit(e.target.value)}
        onBlur={() => void store.flushText(id)}
        onKeyDown={onKey}
        spellCheck
      />
    </section>
  );
});

function ToolBtn({
  tip,
  label,
  onClick,
  italic,
}: {
  tip: string;
  label: string;
  onClick: () => void;
  italic?: boolean;
}) {
  return (
    <button
      className="tool"
      title={tip}
      onClick={onClick}
      style={italic ? { fontStyle: "italic" } : undefined}
    >
      {label}
    </button>
  );
}

export default memo(Editor);
