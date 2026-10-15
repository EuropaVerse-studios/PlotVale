// Custom menu bar (brand-styled): File / Project / View / Help.
// Destructive navigation goes through `guard` (unsaved-changes modal).

import { useState } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { pickProjectFile, pickProjectFolder, pickSaveBundle } from "../lib/files";

export type DialogName =
  | "wizard"
  | "export"
  | "goals"
  | "stats"
  | "about"
  | "manual"
  | "tour";

interface Props {
  openDialog: (name: DialogName) => void;
  guard: (action: () => void) => void;
  /** Quit through the same confirmed path as the window X button. */
  onQuit: () => void;
}

export default function MenuBar({ openDialog, guard, onQuit }: Props) {
  const { t, lang, setLang } = useLocale();
  const store = useStore();
  const { project, expert, scrivenings } = store;
  const [open, setOpen] = useState<string | null>(null);

  const hasProject = project !== null;

  async function saveAsFlow(): Promise<void> {
    const name = store.project?.manifest.title ?? t("untitled");
    const p = await pickSaveBundle(name, t("save_title"), t("filter_project"));
    if (p) await store.doSaveAs(p, false);
  }

  async function saveFlow(): Promise<void> {
    if (!store.hasPath) {
      await saveAsFlow();
      return;
    }
    await store.doSave();
  }

  function menu(label: string, key: string, body: () => React.ReactNode) {
    return (
      <div
        className="menu"
        onMouseEnter={() => {
          if (open !== null) setOpen(key);
        }}
      >
        <button
          className={open === key ? "menu-label active" : "menu-label"}
          onClick={() => setOpen(open === key ? null : key)}
        >
          {label}
        </button>
        {open === key && (
          <>
            <div className="menu-scrim" onClick={() => setOpen(null)} />
            <div className="menu-drop" onClick={() => setOpen(null)}>
              {body()}
            </div>
          </>
        )}
      </div>
    );
  }

  function item(
    label: string,
    action: () => void,
    opts?: { disabled?: boolean; check?: boolean },
  ) {
    return (
      <button
        className="menu-item"
        disabled={opts?.disabled}
        onClick={() => {
          setOpen(null);
          action();
        }}
      >
        {opts?.check !== undefined && <span className="check">{opts.check ? "✓ " : "  "}</span>}
        {label}
      </button>
    );
  }

  return (
    <nav className="menubar" onMouseLeave={() => setOpen(null)}>
      {menu(t("menu_file"), "file", () => (
        <>
          {item(`✦ ${t("menu_new")}`, () => guard(() => openDialog("wizard")))}
          {item(`📂 ${t("menu_open_file")}`, () =>
            guard(async () => {
              const p = await pickProjectFile(t("open_title_file"), t("filter_project"));
              if (p) await store.doOpen(p);
            }),
          )}
          {item(`📁 ${t("menu_open_folder")}`, () =>
            guard(async () => {
              const p = await pickProjectFolder(t("open_title_folder"));
              if (p) await store.doOpen(p);
            }),
          )}
          <div className="menu-sep" />
          {item(`💾 ${t("menu_save")}`, () => void saveFlow(), { disabled: !hasProject })}
          {item(t("menu_save_as"), () => void saveAsFlow(), { disabled: !hasProject })}
          {item(t("menu_export"), () => openDialog("export"), { disabled: !hasProject })}
          {item(t("menu_close_project"), () => guard(() => void store.doClose()), {
            disabled: !hasProject,
          })}
          <div className="menu-sep" />
          {item(`✕ ${t("menu_quit")}`, () => onQuit())}
        </>
      ))}
      {menu(t("menu_project"), "project", () => (
        <>
          {item(`📄 ${t("menu_new_scene")}`, () => void store.createNode(parentForNew(store), "scene", t("scene_label")), {
            disabled: !hasProject,
          })}
          {item(`📁 ${t("menu_new_folder")}`, () => void store.createNode(parentForNew(store), "folder", t("chapter_label")), {
            disabled: !hasProject,
          })}
          {item(`📝 ${t("menu_new_note")}`, () => void store.createNode(parentForNew(store), "note", t("kind_note")), {
            disabled: !hasProject,
          })}
          <div className="menu-sep" />
          {item(`🎯 ${t("menu_goals")}`, () => openDialog("goals"), { disabled: !hasProject })}
          {item(`📊 ${t("menu_stats")}`, () => openDialog("stats"), { disabled: !hasProject })}
        </>
      ))}
      {menu(t("menu_view"), "view", () => (
        <>
          {item(t("menu_expert"), () => store.setExpert(!expert), { check: expert })}
          {item(t("view_scrivenings"), () => store.setScrivenings(!scrivenings), {
            check: scrivenings,
          })}
          <div className="menu-sep" />
          {item(t("lang_it"), () => setLang("it"), { check: lang === "it" })}
          {item(t("lang_en"), () => setLang("en"), { check: lang === "en" })}
        </>
      ))}
      {menu(t("menu_help"), "help", () => (
        <>
          {item(`📖 ${t("menu_manual")}`, () => openDialog("manual"))}
          {item(`🎉 ${t("menu_tour")}`, () => openDialog("tour"))}
          {item(`ℹ️ ${t("menu_about")}`, () => openDialog("about"))}
        </>
      ))}
    </nav>
  );
}

/** Fitting parent for menu-created nodes (selection if a container, else root). */
function parentForNew(store: ReturnType<typeof useStore>): string {
  const { nodes, selection, draftRoot } = store;
  if (selection) {
    const node = nodes.get(selection);
    if (
      node &&
      (node.kind === "draft_root" ||
        node.kind === "folder" ||
        node.kind === "research_folder" ||
        node.kind === "research_item")
    ) {
      return selection;
    }
  }
  return draftRoot;
}
