// PlotVale shell: menu bar, binder / center tabs / inspector, status bar,
// dialogs, and the unsaved-changes guard (menu actions + window close).

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Binder from "./components/Binder";
import Corkboard from "./components/Corkboard";
import Editor from "./components/Editor";
import Inspector from "./components/Inspector";
import MenuBar from "./components/MenuBar";
import type { DialogName } from "./components/MenuBar";
import SearchPanel from "./components/SearchPanel";
import StatusBar from "./components/StatusBar";
import {
  AboutDialog,
  ExportDialog,
  GoalsDialog,
  ManualDialog,
  StatsDialog,
  UnsavedGuard,
  WizardDialog,
} from "./components/Dialogs";
import { useLocale } from "./lib/i18n";
import { useStore } from "./lib/store";
import { pickProjectFile, pickProjectFolder, pickSaveBundle } from "./lib/files";

export default function App() {
  const { t } = useLocale();
  const store = useStore();
  const [dialog, setDialog] = useState<DialogName | null>(null);
  const [guard, setGuard] = useState<(() => void) | null>(null);
  const [pendingClose, setPendingClose] = useState(false);

  const dirtyRef = useRef(store.dirty);
  dirtyRef.current = store.dirty;
  const hasPathRef = useRef(store.hasPath);
  hasPathRef.current = store.hasPath;

  /** Runs the action immediately, or after the unsaved-changes modal. */
  const guarded = useCallback((action: () => void) => {
    if (dirtyRef.current) {
      setGuard(() => action);
    } else {
      action();
    }
  }, []);

  const saveThroughGuard = useCallback(async () => {
    if (hasPathRef.current) {
      if (await store.doSave()) {
        setGuard((g) => {
          if (g) {
            if (pendingClose) void getCurrentWindow().close();
            else g();
          }
          return null;
        });
        if (pendingClose) setPendingClose(false);
      }
    } else {
      const name = store.project?.manifest.title ?? "Untitled";
      const p = await pickSaveBundle(name);
      if (p && (await store.doSaveAs(p, false))) {
        setGuard((g) => {
          if (g) {
            if (pendingClose) void getCurrentWindow().close();
            else g();
          }
          return null;
        });
        if (pendingClose) setPendingClose(false);
      }
    }
  }, [pendingClose, store]);

  // Intercept window close with unsaved changes (native close button, Alt+F4).
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onCloseRequested((e) => {
        if (dirtyRef.current) {
          e.preventDefault();
          setPendingClose(true);
          setGuard(() => () => {});
        }
      })
      .then((off) => {
        unlisten = off;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, []);

  const centerTab = store.centerTab;

  return (
    <div className="app">
      <MenuBar openDialog={setDialog} guard={guarded} />
      {store.project ? (
        <div className="workspace">
          <Binder />
          <main className="center">
            <div className="tabs" role="tablist">
              {(
                [
                  ["editor", t("tab_editor")],
                  ["corkboard", t("tab_corkboard")],
                  ["search", t("tab_search")],
                ] as const
              ).map(([key, label]) => (
                <button
                  key={key}
                  role="tab"
                  aria-selected={centerTab === key}
                  className={centerTab === key ? "tab active" : "tab"}
                  onClick={() => store.setCenterTab(key)}
                >
                  {label}
                </button>
              ))}
            </div>
            <div className="tab-body">
              {centerTab === "editor" && <Editor />}
              {centerTab === "corkboard" && <Corkboard />}
              {centerTab === "search" && <SearchPanel />}
            </div>
          </main>
          <Inspector />
        </div>
      ) : (
        <Welcome onWizard={() => setDialog("wizard")} />
      )}
      <StatusBar />
      {dialog === "wizard" && <WizardDialog onClose={() => setDialog(null)} />}
      {dialog === "export" && <ExportDialog onClose={() => setDialog(null)} />}
      {dialog === "goals" && <GoalsDialog onClose={() => setDialog(null)} />}
      {dialog === "stats" && <StatsDialog onClose={() => setDialog(null)} />}
      {dialog === "about" && <AboutDialog onClose={() => setDialog(null)} />}
      {dialog === "manual" && <ManualDialog onClose={() => setDialog(null)} />}
      {guard && (
        <UnsavedGuard
          onSave={() => void saveThroughGuard()}
          onDiscard={() => {
            setGuard((g) => {
              if (g) {
                if (pendingClose) void getCurrentWindow().close();
                else g();
              }
              return null;
            });
            if (pendingClose) setPendingClose(false);
          }}
          onCancel={() => {
            setGuard(null);
            setPendingClose(false);
          }}
        />
      )}
    </div>
  );
}

function Welcome({ onWizard }: { onWizard: () => void }) {
  const { t } = useLocale();
  const store = useStore();

  async function openFile() {
    const p = await pickProjectFile();
    if (p) await store.doOpen(p);
  }

  async function openFolder() {
    const p = await pickProjectFolder();
    if (p) await store.doOpen(p);
  }

  return (
    <div className="welcome">
      <h1>{t("welcome_title")}</h1>
      <p className="dim">{t("welcome_sub")}</p>
      <div className="welcome-actions">
        <button className="btn primary" onClick={onWizard}>
          ✦ {t("welcome_new")}
        </button>
        <button className="btn" onClick={() => void openFile()}>
          {t("welcome_open_file")}
        </button>
        <button className="btn" onClick={() => void openFolder()}>
          {t("welcome_open_folder")}
        </button>
      </div>
    </div>
  );
}
