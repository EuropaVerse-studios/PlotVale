// PlotVale shell: menu bar, binder / center tabs / inspector, status bar,
// dialogs (code-split), onboarding tour, busy overlay, unsaved-changes guard.

import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Binder from "./components/Binder";
import Corkboard from "./components/Corkboard";
import Editor from "./components/Editor";
import Inspector from "./components/Inspector";
import MenuBar from "./components/MenuBar";
import type { DialogName } from "./components/MenuBar";
import Onboarding from "./components/Onboarding";
import SearchPanel from "./components/SearchPanel";
import StatusBar from "./components/StatusBar";
import { useLocale } from "./lib/i18n";
import { useStore } from "./lib/store";
import { APP_VERSION, greetingKey, hasSeenOnboarding, markOnboarded } from "./lib/appinfo";
import bannerUrl from "./assets/banner.jpg";
import logoUrl from "./assets/logo.png";
import { pickProjectFile, pickProjectFolder, pickSaveBundle } from "./lib/files";

// Heavy dialogs live in their own chunk: the first paint stays light and
// opening a dialog shows a spinner instead of blocking startup.
const WizardDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.WizardDialog })),
);
const ExportDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.ExportDialog })),
);
const GoalsDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.GoalsDialog })),
);
const StatsDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.StatsDialog })),
);
const AboutDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.AboutDialog })),
);
const ManualDialog = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.ManualDialog })),
);
const UnsavedGuard = lazy(() =>
  import("./components/Dialogs").then((m) => ({ default: m.UnsavedGuard })),
);

export default function App() {
  const { t } = useLocale();
  const store = useStore();
  const [dialog, setDialog] = useState<DialogName | null>(null);
  const [guard, setGuard] = useState<(() => void) | null>(null);
  const [pendingClose, setPendingClose] = useState(false);
  // First launch: tour over the welcome screen (persisted in localStorage).
  const [tour, setTour] = useState<boolean>(() => !hasSeenOnboarding());

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
            // destroy() bypasses onCloseRequested: close() would re-emit it
            // while still dirty and trap the window open.
            if (pendingClose) void getCurrentWindow().destroy();
            else g();
          }
          return null;
        });
        if (pendingClose) setPendingClose(false);
      }
    } else {
      const name = store.project?.manifest.title ?? t("untitled");
      const p = await pickSaveBundle(name, t("save_title"), t("filter_project"));
      if (p && (await store.doSaveAs(p, false))) {
        setGuard((g) => {
          if (g) {
            // destroy() bypasses onCloseRequested: close() would re-emit it
            // while still dirty and trap the window open.
            if (pendingClose) void getCurrentWindow().destroy();
            else g();
          }
          return null;
        });
        if (pendingClose) setPendingClose(false);
      }
    }
  }, [pendingClose, store, t]);

  /** Quit from the menu follows the same path as the window X button. */
  const requestQuit = useCallback(() => {
    if (dirtyRef.current) {
      setPendingClose(true);
      setGuard(() => () => {});
    } else {
      void getCurrentWindow().close();
    }
  }, []);

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

  const closeTour = useCallback(() => {
    markOnboarded();
    setTour(false);
    setDialog((d) => (d === "tour" ? null : d));
  }, []);

  const openWizardFromTour = useCallback(() => {
    markOnboarded();
    setTour(false);
    setDialog("wizard");
  }, []);

  // The tour shows on first launch (welcome screen) or on demand from Help.
  const showTour = dialog === "tour" || (tour && !store.project);

  const centerTab = store.centerTab;

  return (
    <div className="app">
      <MenuBar openDialog={setDialog} guard={guarded} onQuit={requestQuit} />
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
      <Suspense fallback={<DialogFallback />}>
        {dialog === "wizard" && <WizardDialog onClose={() => setDialog(null)} />}
        {dialog === "export" && <ExportDialog onClose={() => setDialog(null)} />}
        {dialog === "goals" && <GoalsDialog onClose={() => setDialog(null)} />}
        {dialog === "stats" && <StatsDialog onClose={() => setDialog(null)} />}
        {dialog === "about" && <AboutDialog onClose={() => setDialog(null)} />}
        {dialog === "manual" && <ManualDialog onClose={() => setDialog(null)} />}
      </Suspense>
      {showTour && <Onboarding onWizard={openWizardFromTour} onClose={closeTour} />}
      {store.busy && (
        <div className="busy-overlay" role="status" aria-live="polite">
          <div className="spinner large" />
          <p>{t(store.busy)}</p>
        </div>
      )}
      <Suspense fallback={guard ? <DialogFallback /> : null}>
      {guard && (
        <UnsavedGuard
          onSave={() => void saveThroughGuard()}
          onDiscard={() => {
            setGuard((g) => {
              if (g) {
                if (pendingClose) void getCurrentWindow().destroy();
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
      </Suspense>
    </div>
  );
}

function DialogFallback() {
  const { t } = useLocale();
  return (
    <div className="modal-overlay">
      <div className="dlg-fallback" role="status">
        <div className="spinner" />
        <span className="small dim">{t("common_loading")}</span>
      </div>
    </div>
  );
}

function Welcome({ onWizard }: { onWizard: () => void }) {
  const { t } = useLocale();
  const store = useStore();

  async function openFile() {
    const p = await pickProjectFile(t("open_title_file"), t("filter_project"));
    if (p) await store.doOpen(p);
  }

  async function openFolder() {
    const p = await pickProjectFolder(t("open_title_folder"));
    if (p) await store.doOpen(p);
  }

  const cards = [
    {
      icon: "✦",
      title: t("welcome_new"),
      desc: t("welcome_card_new_d"),
      action: onWizard,
      primary: true,
    },
    {
      icon: "📂",
      title: t("welcome_open_file"),
      desc: t("welcome_card_open_d"),
      action: () => void openFile(),
      primary: false,
    },
    {
      icon: "📁",
      title: t("welcome_open_folder"),
      desc: t("welcome_card_folder_d"),
      action: () => void openFolder(),
      primary: false,
    },
  ];

  return (
    <div
      className="welcome has-banner"
      style={{
        backgroundImage: `linear-gradient(rgba(23, 19, 16, 0.9), rgba(23, 19, 16, 0.95)), url(${bannerUrl})`,
      }}
    >
      <div className="welcome-hero">
        <img className="brand-logo" src={logoUrl} alt="PlotVale" />
        <h1 className="brand-title">PlotVale</h1>
        <span className="version-pill">v{APP_VERSION}</span>
      </div>
      <p className="welcome-greet">
        {t(greetingKey())} — {t("welcome_hello")}
      </p>
      <p className="dim welcome-tag">{t("welcome_tag")}</p>
      <div className="welcome-cards">
        {cards.map((c, i) => (
          <button
            key={c.title}
            className={c.primary ? "welcome-card primary-card" : "welcome-card"}
            style={{ "--d": `${i * 90}ms` } as CSSProperties}
            onClick={c.action}
          >
            <span className="wc-icon" aria-hidden>
              {c.icon}
            </span>
            <span className="wc-title">{c.title}</span>
            <span className="wc-desc">{c.desc}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
