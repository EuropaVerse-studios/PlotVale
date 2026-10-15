// Corkboard: live cards of the container's scenes/folders/notes.

import { memo } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { countWords } from "../lib/count";

const DOT: Record<string, string> = { todo: "○", draft: "●", revised: "◆", final: "★" };

function Corkboard() {
  const { t } = useLocale();
  const store = useStore();
  const { nodes, selection, project } = store;

  if (!project) return <div className="center-pane" />;

  let parentId = project.draft_root;
  if (selection) {
    const sel = nodes.get(selection);
    if (sel && (sel.kind === "draft_root" || sel.kind === "folder" || sel.kind === "research_folder")) {
      parentId = selection;
    } else if (sel?.parent) {
      parentId = sel.parent;
    }
  }
  const parent = nodes.get(parentId);
  const cards = (parent?.children ?? [])
    .map((id) => nodes.get(id))
    .filter(
      (n): n is NonNullable<typeof n> =>
        !!n && (n.kind === "scene" || n.kind === "folder" || n.kind === "note"),
    );

  return (
    <div className="center-pane">
      <header className="doc-head">
        <h3>
          {t("tab_corkboard")}: {parent?.title ?? ""}
        </h3>
      </header>
      {cards.length === 0 ? (
        <p className="hint">{t("cork_empty")}</p>
      ) : (
        <div className="cards">
          {cards.map((card) => (
            <article key={card.id} className="card">
              <h4>
                <span className={`status-dot st-${card.meta.status}`}>{DOT[card.meta.status]}</span>{" "}
                {card.title}
              </h4>
              {card.meta.synopsis.trim() === "" ? (
                <p className="small dim">{t("cork_no_synopsis")}</p>
              ) : (
                <p>{card.meta.synopsis}</p>
              )}
              <div className="row">
                <span className="small">
                  {countWords(store.texts[card.id] ?? "").words} {t("cork_words")}
                </span>
                <span className="spacer" />
                <button
                  className="btn small"
                  onClick={() => {
                    store.setSelection(card.id);
                    store.setOpenDoc(card.id);
                    store.setCenterTab("editor");
                  }}
                >
                  {t("cork_open")}
                </button>
                <button
                  className="btn small"
                  title={t("ctx_move_up")}
                  onClick={() => void store.reorderNode(card.id, true)}
                >
                  ▲
                </button>
                <button
                  className="btn small"
                  title={t("ctx_move_down")}
                  onClick={() => void store.reorderNode(card.id, false)}
                >
                  ▼
                </button>
              </div>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}

export default memo(Corkboard);
