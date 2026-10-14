// Modal dialogs: writing goals, statistics, export (+HTML preview),
// new-project wizard, about, quick guide, unsaved-changes guard.

import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { pickExportFile, pickSaveBundle, pickSaveFolder } from "../lib/files";
import type { StatsDto, Targets } from "../lib/types";
import pkg from "../../package.json";

export function Modal({
  title,
  onClose,
  children,
  wide,
}: {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
  wide?: boolean;
}) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className={wide ? "modal wide" : "modal"} onClick={(e) => e.stopPropagation()}>
        <div className="row">
          <h3>{title}</h3>
          <span className="spacer" />
          <button className="btn small" onClick={onClose}>
            ✕
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

export function GoalsDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  const store = useStore();
  const targets = store.project?.targets;
  if (!targets) return null;

  async function set(next: Targets) {
    await store.setTargets(next);
  }

  const total = store.totalWords;
  const projectGoal = targets.project_goal_words ?? 0;
  const dailyGoal = targets.daily_goal_words ?? 0;

  return (
    <Modal title={t("goals_title")} onClose={onClose}>
      <label className="check">
        <input
          type="checkbox"
          checked={targets.project_goal_words !== null}
          onChange={(e) => void set({ ...targets, project_goal_words: e.target.checked ? 50000 : null })}
        />
        {t("goals_project_enable")}
      </label>
      {targets.project_goal_words !== null && (
        <>
          <div className="row">
            <span>{t("goals_project_words")}</span>
            <input
              type="number"
              min={100}
              step={100}
              value={projectGoal}
              onChange={(e) => void set({ ...targets, project_goal_words: Number(e.target.value) || 0 })}
            />
          </div>
          <progress value={Math.min(1, total / Math.max(1, projectGoal))} max={1}>
            {t("goals_progress")}
          </progress>
        </>
      )}
      <label className="check">
        <input
          type="checkbox"
          checked={targets.daily_goal_words !== null}
          onChange={(e) => void set({ ...targets, daily_goal_words: e.target.checked ? 500 : null })}
        />
        {t("goals_daily_enable")}
      </label>
      {targets.daily_goal_words !== null && (
        <>
          <div className="row">
            <span>{t("goals_daily_words")}</span>
            <input
              type="number"
              min={10}
              step={10}
              value={dailyGoal}
              onChange={(e) => void set({ ...targets, daily_goal_words: Number(e.target.value) || 0 })}
            />
          </div>
          <p className="small">
            {t("goals_today")}: {targets.words_today}
          </p>
          <progress
            value={Math.min(1, targets.words_today / Math.max(1, dailyGoal))}
            max={1}
          >
            {t("goals_progress")}
          </progress>
        </>
      )}
      {targets.project_goal_words === null && targets.daily_goal_words === null && (
        <p className="small dim">{t("goals_no_goal")}</p>
      )}
    </Modal>
  );
}

export function StatsDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  const [stats, setStats] = useState<StatsDto | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    api.projectStats().then(setStats).catch((e: unknown) => setError(`${e}`));
  }, []);

  return (
    <Modal title={t("stats_title")} onClose={onClose}>
      {error !== "" && <p className="error">{error}</p>}
      {!stats ? (
        <p className="small">…</p>
      ) : (
        <>
          <p>
            {t("stats_total_words")}: <strong>{stats.words}</strong>
          </p>
          <p>
            {t("stats_docs")}: {stats.docs_with_text}
          </p>
          <p>
            {t("stats_chars")}: {stats.characters_no_spaces}
          </p>
          <p>
            {t("stats_paras")}: {stats.paragraphs}
          </p>
          <h4>{t("stats_by_chapter")}</h4>
          {stats.per_chapter.map((c) => (
            <p key={c.id} className="small">
              {c.title}: {c.words}
            </p>
          ))}
        </>
      )}
    </Modal>
  );
}

const FORMATS: [string, string, string][] = [
  ["md", "Markdown (.md)", "md"],
  ["txt", "Testo (.txt)", "txt"],
  ["html", "Pagina web (.html)", "html"],
  ["docx", "Word (.docx)", "docx"],
  ["epub", "Ebook (.epub)", "epub"],
];

export function ExportDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  const store = useStore();
  const profiles = store.project?.profiles;
  const [profileName, setProfileName] = useState(profiles?.active ?? "manoscritto");
  const [keepNotes, setKeepNotes] = useState(
    profiles?.profiles.find((p) => p.name === profiles.active)?.keep_notes ?? false,
  );
  const [dedication, setDedication] = useState(
    profiles?.profiles.find((p) => p.name === profiles.active)?.dedication ?? "",
  );
  const [preview, setPreview] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (!profiles) return null;
  const profs = profiles;
  const selection = store.selection ? [store.selection] : [];

  async function persistProfile() {
    const base = profs.profiles.find((p) => p.name === profileName);
    if (!base) return;
    await api.updateProfile({ ...base, keep_notes: keepNotes, dedication });
    await api.setActiveProfile(profileName);
  }

  async function chooseProfile(name: string) {
    setProfileName(name);
    const base = profs.profiles.find((p) => p.name === name);
    if (base) {
      setKeepNotes(base.keep_notes);
      setDedication(base.dedication);
      await api.setActiveProfile(name).catch(() => {});
    }
  }

  async function doPreview() {
    setBusy(true);
    try {
      await persistProfile();
      setPreview(await api.compilePreview(profileName, keepNotes, selection));
    } catch (e) {
      store.setStatus(`${t("err_export")}: ${e instanceof Error ? e.message : e}`);
    } finally {
      setBusy(false);
    }
  }

  async function doExport(format: string, displayName: string, ext: string) {
    const title = store.project?.manifest.title ?? "manuscript";
    const dest = await pickExportFile(title, displayName, ext);
    if (!dest) return;
    setBusy(true);
    try {
      await persistProfile();
      await api.exportManuscript(format, dest, profileName, keepNotes, selection);
      store.setStatus(t("export_done"));
      onClose();
    } catch (e) {
      store.setStatus(`${t("err_export")}: ${e instanceof Error ? e.message : e}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal title={t("export_title")} onClose={onClose} wide>
      <div className="row">
        <span>{t("export_preset")}</span>
        <select value={profileName} onChange={(e) => void chooseProfile(e.target.value)}>
          {profs.profiles.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name}
            </option>
          ))}
        </select>
        <label className="check">
          <input
            type="checkbox"
            checked={keepNotes}
            onChange={(e) => setKeepNotes(e.target.checked)}
          />
          {t("export_keep_notes")}
        </label>
      </div>
      <label className="field">
        <span>{t("export_dedication")}</span>
        <input value={dedication} onChange={(e) => setDedication(e.target.value)} />
      </label>
      <div className="row">
        <button className="btn" disabled={busy} onClick={() => void doPreview()}>
          {t("export_preview")}
        </button>
        {FORMATS.map(([format, displayName, ext]) => (
          <button
            key={format}
            className="btn"
            disabled={busy}
            onClick={() => void doExport(format, displayName, ext)}
          >
            {displayName}
          </button>
        ))}
      </div>
      {preview !== null && (
        <iframe title="preview" className="preview-frame" srcDoc={preview} sandbox="" />
      )}
    </Modal>
  );
}

export function AboutDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  return (
    <Modal title={t("about_title")} onClose={onClose}>
      <p style={{ whiteSpace: "pre-line" }}>{t("about_text")}</p>
      <p className="small dim">PlotVale {(pkg as { version: string }).version}</p>
    </Modal>
  );
}

export function ManualDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  return (
    <Modal title={t("menu_manual")} onClose={onClose}>
      <p style={{ whiteSpace: "pre-line" }}>{t("manual_body")}</p>
    </Modal>
  );
}

export function UnsavedGuard({
  onSave,
  onDiscard,
  onCancel,
}: {
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}) {
  const { t } = useLocale();
  return (
    <div className="modal-overlay">
      <div className="modal">
        <h3>{t("dlg_unsaved_title")}</h3>
        <p>{t("dlg_unsaved_msg")}</p>
        <div className="row end">
          <button className="btn primary" onClick={onSave}>
            {t("dlg_save")}
          </button>
          <button className="btn danger" onClick={onDiscard}>
            {t("dlg_discard")}
          </button>
          <button className="btn" onClick={onCancel}>
            {t("dlg_cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}

type Template = "novel" | "story" | "blank";

export function WizardDialog({ onClose }: { onClose: () => void }) {
  const { t } = useLocale();
  const store = useStore();
  const [step, setStep] = useState(0);
  const [template, setTemplate] = useState<Template>("novel");
  const [title, setTitle] = useState("");
  const [author, setAuthor] = useState("");
  const [chapters, setChapters] = useState(3);
  const [scenes, setScenes] = useState(3);
  const [example, setExample] = useState(true);
  const [daily, setDaily] = useState(0);
  const [expert, setExpert] = useState(false);
  const [folderMode, setFolderMode] = useState(false);
  const [location, setLocation] = useState("");
  const [busy, setBusy] = useState(false);

  const steps = [
    t("wiz_step_template"),
    t("wiz_step_details"),
    t("wiz_step_structure"),
    t("wiz_step_features"),
    t("wiz_step_location"),
  ];

  async function browse() {
    if (folderMode) {
      const p = await pickSaveFolder();
      if (p) setLocation(p);
    } else {
      const p = await pickSaveBundle(title.trim() === "" ? "Untitled" : title.trim());
      if (p) setLocation(p);
    }
  }

  async function create() {
    setBusy(true);
    try {
      const name = title.trim() === "" ? t("untitled") : title.trim();
      await store.doNew(name, author.trim());
      // Structural reads go straight to the backend: the render closure
      // would otherwise see the pre-wizard project.
      const tree = await api.getTree();
      const root = tree.find((n) => n.kind === "draft_root")?.id ?? "";
      const firstChapter = tree.find((n) => n.parent === root);
      if (firstChapter) {
        try {
          await store.deleteNode(firstChapter.id);
        } catch {
          /* keep starter on failure */
        }
      }
      const nChapters = template === "blank" ? 0 : template === "story" ? 1 : Math.max(1, chapters);
      const nScenes = template === "blank" ? 0 : Math.max(1, scenes);
      for (let c = 0; c < nChapters; c++) {
        const ch = await store.createNode(root, "folder", `${t("chapter_label")} ${c + 1}`);
        for (let s = 0; s < nScenes; s++) {
          const sc = await store.createNode(ch.id, "scene", `${t("scene_label")} ${s + 1}`);
          if (example && c === 0 && s === 0) {
            store.setText(sc.id, t("wiz_example_d"));
          }
        }
      }
      if (template === "blank") {
        await store.createNode(root, "scene", t("scene_label"));
      }
      if (daily > 0) {
        const tg = await api.getTargets();
        await store.setTargets({ ...tg, daily_goal_words: daily });
      }
      store.setExpert(expert);
      if (location.trim() !== "") {
        await store.doSaveAs(location.trim(), folderMode);
      }
      onClose();
    } catch (e) {
      store.setStatus(`Error: ${e instanceof Error ? e.message : e}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal wide" onClick={(e) => e.stopPropagation()}>
        <h3>{t("wiz_title")}</h3>
        <ol className="steps">
          {steps.map((s, i) => (
            <li key={s} className={i === step ? "current" : i < step ? "done" : ""}>
              {s}
            </li>
          ))}
        </ol>
        {step === 0 && (
          <>
            <p className="small dim">{t("wiz_template_hint")}</p>
            {(["novel", "story", "blank"] as Template[]).map((tpl) => (
              <label key={tpl} className="check big">
                <input
                  type="radio"
                  name="tpl"
                  checked={template === tpl}
                  onChange={() => setTemplate(tpl)}
                />
                <strong>{t(`wiz_tpl_${tpl}`)}</strong> — {t(`wiz_tpl_${tpl}_d`)}
              </label>
            ))}
          </>
        )}
        {step === 1 && (
          <>
            <p className="small dim">{t("wiz_details_hint")}</p>
            <label className="field">
              <span>{t("wiz_title_label")}</span>
              <input value={title} onChange={(e) => setTitle(e.target.value)} />
            </label>
            <label className="field">
              <span>{t("wiz_author_label")}</span>
              <input value={author} onChange={(e) => setAuthor(e.target.value)} />
            </label>
          </>
        )}
        {step === 2 && (
          <>
            <p className="small dim">{t("wiz_structure_hint")}</p>
            <label className="field">
              <span>{t("wiz_chapters")}</span>
              <input
                type="number"
                min={1}
                max={50}
                value={chapters}
                onChange={(e) => setChapters(Number(e.target.value) || 1)}
              />
            </label>
            <label className="field">
              <span>{t("wiz_scenes")}</span>
              <input
                type="number"
                min={1}
                max={50}
                value={scenes}
                onChange={(e) => setScenes(Number(e.target.value) || 1)}
              />
            </label>
            <label className="check">
              <input type="checkbox" checked={example} onChange={(e) => setExample(e.target.checked)} />
              {t("wiz_example")} — {t("wiz_example_d")}
            </label>
          </>
        )}
        {step === 3 && (
          <>
            <p className="small dim">{t("wiz_features_hint")}</p>
            <label className="field">
              <span>{t("wiz_daily")}</span>
              <input
                type="number"
                min={0}
                step={50}
                value={daily}
                onChange={(e) => setDaily(Number(e.target.value) || 0)}
              />
            </label>
            <label className="check">
              <input type="checkbox" checked={expert} onChange={(e) => setExpert(e.target.checked)} />
              {t("wiz_expert")} — {t("wiz_expert_d")}
            </label>
          </>
        )}
        {step === 4 && (
          <>
            <p className="small dim">{t("wiz_location_hint")}</p>
            <label className="check">
              <input
                type="radio"
                name="locmode"
                checked={!folderMode}
                onChange={() => setFolderMode(false)}
              />
              {t("wiz_bundle")}
            </label>
            <label className="check">
              <input
                type="radio"
                name="locmode"
                checked={folderMode}
                onChange={() => setFolderMode(true)}
              />
              {t("wiz_folder")}
            </label>
            <p className="small dim">{t("wiz_location_d")}</p>
            <div className="row">
              <input
                value={location}
                onChange={(e) => setLocation(e.target.value)}
                placeholder={t("wiz_location_optional")}
              />
              <button className="btn small" onClick={() => void browse()}>
                {t("wiz_browse")}
              </button>
            </div>
          </>
        )}
        <div className="row end">
          <button className="btn" onClick={onClose}>
            {t("wiz_cancel")}
          </button>
          {step > 0 && (
            <button className="btn" onClick={() => setStep(step - 1)}>
              {t("wiz_back")}
            </button>
          )}
          {step < 4 ? (
            <button className="btn primary" onClick={() => setStep(step + 1)}>
              {t("wiz_next")}
            </button>
          ) : (
            <button className="btn primary" disabled={busy} onClick={() => void create()}>
              {t("wiz_create")}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
