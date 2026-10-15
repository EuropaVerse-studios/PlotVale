// Inspector: metadata of the selection (status, label, keywords, synopsis,
// export flag, color, custom fields in expert mode), character/location
// sheets, and snapshots.

import { memo, useEffect, useState } from "react";
import { api } from "../lib/api";
import { diffLines } from "../lib/diff";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { countWords } from "../lib/count";
import type {
  CharacterSheet,
  DocStatus,
  LocationSheet,
  NodeMeta,
  SnapshotMeta,
} from "../lib/types";
import { canHaveText } from "../lib/types";

const STATUSES: DocStatus[] = ["todo", "draft", "revised", "final"];

/** Stable storage keys of the guided questionnaire (never translated). */
const SHEET_QUESTIONS: [string, string][] = [
  ["appearance", "sheet_q1"],
  ["past", "sheet_q2"],
  ["fear", "sheet_q3"],
  ["want", "sheet_q4"],
];
/** Pre-v0.3 Italian storage keys, migrated on write. */
const LEGACY_KEYS: [string, string][] = [
  ["Aspetto e modo di fare", "appearance"],
  ["Passato in poche righe", "past"],
  ["La sua paura più grande", "fear"],
  ["Cosa vuole davvero", "want"],
];

function Inspector() {
  const { t } = useLocale();
  const store = useStore();
  const { nodes, selection } = store;

  if (!store.project) return null;
  const node = selection ? nodes.get(selection) : undefined;
  if (!node) return <aside className="inspector" />;

  return (
    <aside className="inspector">
      <h2>{t("inspector")}</h2>
      <div className="insp-scroll">
        <MetaEditor key={node.id} id={node.id} />
        {node.kind === "character_sheet" && <CharacterEditor id={node.id} />}
        {node.kind === "location_sheet" && <LocationEditor id={node.id} />}
        {canHaveText(node.kind) && <SnapshotBox id={node.id} />}
      </div>
    </aside>
  );
}

const MetaEditor = memo(function MetaEditor({ id }: { id: string }) {
  const { t } = useLocale();
  const store = useStore();
  const node = store.nodes.get(id);
  const meta = node?.meta;
  const [customName, setCustomName] = useState("");
  const [customValue, setCustomValue] = useState("");
  if (!node || !meta) return null;

  function patch(next: NodeMeta) {
    void store.updateNodeMeta(id, next);
  }

  const words = canHaveText(node.kind) ? countWords(store.texts[id] ?? "").words : 0;

  return (
    <section className="insp-group">
      <label className="field">
        <span>{t("insp_title")}</span>
        <input
          defaultValue={node.title}
          key={`title-${id}`}
          onBlur={(e) => {
            if (e.target.value.trim() !== "" && e.target.value !== node.title) {
              void store.renameNode(id, e.target.value.trim());
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
          }}
        />
      </label>
      <div className="field">
        <span>{t("insp_kind")}</span>
        <span className="small">{t(kindKey(node.kind))}</span>
      </div>
      <label className="field">
        <span>{t("insp_status")}</span>
        <select
          value={meta.status}
          onChange={(e) => patch({ ...meta, status: e.target.value as DocStatus })}
        >
          {STATUSES.map((s) => (
            <option key={s} value={s}>
              {t(`status_${s}`)}
            </option>
          ))}
        </select>
      </label>
      <label className="field">
        <span>{t("insp_label")}</span>
        <input
          defaultValue={meta.label}
          key={`label-${id}`}
          onBlur={(e) => patch({ ...meta, label: e.target.value })}
        />
      </label>
      <label className="field">
        <span>{t("insp_keywords")}</span>
        <input
          defaultValue={meta.keywords.join(", ")}
          key={`kw-${id}`}
          onBlur={(e) =>
            patch({
              ...meta,
              keywords: e.target.value.split(",").map((k) => k.trim()).filter((k) => k !== ""),
            })
          }
        />
      </label>
      <label className="field">
        <span>{t("insp_synopsis")}</span>
        <textarea
          rows={3}
          defaultValue={meta.synopsis}
          key={`syn-${id}`}
          onBlur={(e) => patch({ ...meta, synopsis: e.target.value })}
        />
      </label>
      <label className="check">
        <input
          type="checkbox"
          checked={meta.include_in_export}
          onChange={(e) => patch({ ...meta, include_in_export: e.target.checked })}
        />
        {t("insp_include")}
      </label>
      <label className="field">
        <span>🎨</span>
        <input
          type="color"
          value={meta.color ?? "#f07050"}
          onChange={(e) => patch({ ...meta, color: e.target.value })}
        />
      </label>
      {canHaveText(node.kind) && (
        <div className="field">
          <span>{t("insp_words")}</span>
          <span className="small">{words}</span>
        </div>
      )}
      {store.expert && (
        <details className="custom">
          <summary>{t("insp_custom")}</summary>
          {Object.entries(meta.custom).map(([k, v]) => (
            <div key={k} className="custom-row">
              <span className="small">{k}</span>
              <input
                defaultValue={v}
                onBlur={(e) => patch({ ...meta, custom: { ...meta.custom, [k]: e.target.value } })}
              />
              <button
                className="btn small"
                onClick={() => {
                  const next = { ...meta.custom };
                  delete next[k];
                  patch({ ...meta, custom: next });
                }}
              >
                ✕
              </button>
            </div>
          ))}
          <div className="custom-row">
            <input
              value={customName}
              onChange={(e) => setCustomName(e.target.value)}
              placeholder={t("insp_custom_name")}
            />
            <input
              value={customValue}
              onChange={(e) => setCustomValue(e.target.value)}
              placeholder={t("insp_custom_value")}
            />
            <button
              className="btn small"
              onClick={() => {
                if (customName.trim() === "") return;
                patch({ ...meta, custom: { ...meta.custom, [customName.trim()]: customValue } });
                setCustomName("");
                setCustomValue("");
              }}
            >
            +
            </button>
          </div>
        </details>
      )}
    </section>
  );
});

function kindKey(kind: string): string {
  switch (kind) {
    case "folder":
      return "kind_folder";
    case "scene":
      return "kind_scene";
    case "note":
      return "kind_note";
    case "research_folder":
      return "kind_research_folder";
    case "research_item":
      return "kind_research_item";
    case "character_sheet":
      return "kind_character";
    case "location_sheet":
      return "kind_location";
    case "draft_root":
      return "kind_draft_root";
    case "trash":
      return "kind_trash";
    default:
      return "kind_unknown";
  }
}

function CharacterEditor({ id }: { id: string }) {
  const { t } = useLocale();
  const store = useStore();
  const sheet = store.project?.characters.find((s) => s.node_id === id);
  if (!sheet) return null;

  function set(patch: Partial<CharacterSheet>) {
    const next = { ...sheet, ...patch } as CharacterSheet;
    void store.upsertCharacter(migrateQuestionnaire(next));
  }

  return (
    <section className="insp-group">
      <label className="field">
        <span>{t("sheet_role")}</span>
        <input defaultValue={sheet.role} key={`r-${id}`} onBlur={(e) => set({ role: e.target.value })} />
      </label>
      <label className="field">
        <span>{t("sheet_description")}</span>
        <textarea rows={3} defaultValue={sheet.description} key={`d-${id}`} onBlur={(e) => set({ description: e.target.value })} />
      </label>
      <label className="field">
        <span>{t("sheet_goal")}</span>
        <textarea rows={2} defaultValue={sheet.goal} key={`g-${id}`} onBlur={(e) => set({ goal: e.target.value })} />
      </label>
      <label className="field">
        <span>{t("sheet_conflict")}</span>
        <textarea rows={2} defaultValue={sheet.conflict} key={`c-${id}`} onBlur={(e) => set({ conflict: e.target.value })} />
      </label>
      <h4>{t("sheet_q")}</h4>
      {SHEET_QUESTIONS.map(([key, labelKey]) => (
        <label className="field" key={key}>
          <span>{t(labelKey)}</span>
          <textarea
            rows={2}
            defaultValue={sheet.questionnaire[key] ?? ""}
            key={`${key}-${id}`}
            onBlur={(e) =>
              set({ questionnaire: { ...sheet.questionnaire, [key]: e.target.value } })
            }
          />
        </label>
      ))}
    </section>
  );
}

/** Moves answers stored under legacy Italian keys to the stable keys. */
function migrateQuestionnaire(sheet: CharacterSheet): CharacterSheet {
  const q = { ...sheet.questionnaire };
  for (const [legacy, stable] of LEGACY_KEYS) {
    if (q[legacy] !== undefined && q[stable] === undefined) {
      q[stable] = q[legacy];
    }
    delete q[legacy];
  }
  return { ...sheet, questionnaire: q };
}

function LocationEditor({ id }: { id: string }) {
  const { t } = useLocale();
  const store = useStore();
  const sheet = store.project?.locations.find((s) => s.node_id === id);
  if (!sheet) return null;

  function set(patch: Partial<LocationSheet>) {
    const next = { ...sheet, ...patch } as LocationSheet;
    void store.upsertLocation(next);
  }

  return (
    <section className="insp-group">
      <label className="field">
        <span>{t("sheet_description")}</span>
        <textarea rows={3} defaultValue={sheet.description} key={`d-${id}`} onBlur={(e) => set({ description: e.target.value })} />
      </label>
      <label className="field">
        <span>{t("sheet_sensory")}</span>
        <textarea rows={3} defaultValue={sheet.sensory} key={`s-${id}`} onBlur={(e) => set({ sensory: e.target.value })} />
      </label>
      <label className="field">
        <span>{t("sheet_notes")}</span>
        <textarea rows={2} defaultValue={sheet.notes} key={`n-${id}`} onBlur={(e) => set({ notes: e.target.value })} />
      </label>
    </section>
  );
}

function SnapshotBox({ id }: { id: string }) {
  const { t } = useLocale();
  const store = useStore();
  const [label, setLabel] = useState("");
  const [snaps, setSnaps] = useState<SnapshotMeta[]>([]);
  const [compareId, setCompareId] = useState<string | null>(null);

  useEffect(() => {
    api.listSnapshots(id).then(setSnaps).catch(() => setSnaps([]));
  }, [id]);

  async function take() {
    try {
      await api.createSnapshot(id, label);
      setLabel("");
      setSnaps(await api.listSnapshots(id));
    } catch (e) {
      store.setStatus(`${t("err_generic")}: ${e instanceof Error ? e.message : e}`);
    }
  }

  return (
    <section className="insp-group">
      <h4>{t("insp_snapshots")}</h4>
      <div className="row">
        <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder={t("snapshot_title")} />
        <button className="btn small" onClick={() => void take()}>
          {t("insp_snapshot_new")}
        </button>
      </div>
      {snaps.map((s) => (
        <div key={s.id} className="row snap">
          <span className="small">
            {s.created_at} {s.label}
          </span>
          <span className="spacer" />
          <button className="btn small" onClick={() => setCompareId(s.id)}>
            {t("snapshot_show")}
          </button>
          <button
            className="btn small"
            onClick={() =>
              void api
                .restoreSnapshot(id, s.id)
                .then(() => store.adoptBackendText(id))
                .catch((e: unknown) =>
                  store.setStatus(`${t("err_generic")}: ${e instanceof Error ? e.message : e}`),
                )
            }
          >
            {t("insp_snapshot_restore")}
          </button>
          <button
            className="btn small danger"
            onClick={() =>
              void api
                .deleteSnapshot(s.id)
                .then(() => api.listSnapshots(id))
                .then(setSnaps)
                .catch((e: unknown) =>
                  store.setStatus(`${t("err_generic")}: ${e instanceof Error ? e.message : e}`),
                )
            }
          >
            {t("insp_snapshot_delete")}
          </button>
        </div>
      ))}
      {compareId && <SnapshotCompare docId={id} snapId={compareId} onClose={() => setCompareId(null)} />}
    </section>
  );
}

export function SnapshotCompare({
  docId,
  snapId,
  onClose,
}: {
  docId: string;
  snapId: string;
  onClose: () => void;
}) {
  const { t } = useLocale();
  const store = useStore();
  const [oldText, setOldText] = useState<string | null>(null);

  useEffect(() => {
    api.getSnapshotText(snapId).then(setOldText).catch(() => setOldText(""));
  }, [snapId]);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal wide" onClick={(e) => e.stopPropagation()}>
        <h3>{t("snapshot_title")}</h3>
        <DiffView oldText={oldText ?? ""} newText={store.texts[docId] ?? ""} />
        <div className="row end">
          <button
            className="btn"
            onClick={() =>
              void api
                .restoreSnapshot(docId, snapId)
                .then(() => store.adoptBackendText(docId))
                .then(onClose)
                .catch((e: unknown) =>
                  store.setStatus(`${t("err_generic")}: ${e instanceof Error ? e.message : e}`),
                )
            }
          >
            {t("insp_snapshot_restore")}
          </button>
          <button className="btn" onClick={onClose}>
            {t("common_close")}
          </button>
        </div>
      </div>
    </div>
  );
}

function DiffView({ oldText, newText }: { oldText: string; newText: string }) {
  const { lines, truncated } = diffLines(oldText, newText);
  return (
    <div className="diff">
      {lines.map((l, i) => (
        <div key={i} className={`diff-${l.type}`}>
          {l.type === "del" ? `− ${l.text}` : l.type === "add" ? `+ ${l.text}` : l.text}
        </div>
      ))}
      {truncated && <div className="small">…</div>}
    </div>
  );
}

export default memo(Inspector);
