// Status bar: file + dirty dot, message, language, totals (all local counts).

import { memo } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";

function StatusBar() {
  const { t, lang } = useLocale();
  const { project, path, dirty, status, totalWords, openDocWords, openDoc } = useStore();

  if (!project) {
    return (
      <footer className="statusbar">
        <span>{t("status_no_project")}</span>
      </footer>
    );
  }

  const name =
    path?.split(/[/\\]/).pop() ??
    project.manifest.title;

  return (
    <footer className="statusbar">
      <span className={dirty ? "dirty" : "clean"}>{dirty ? "●" : "○"}</span>
      <span className="filename">{name}</span>
      {status !== "" && status !== t("status_saved") && (
        <>
          <span className="sep">|</span>
          <span className="msg">{status}</span>
        </>
      )}
      <span className="spacer" />
      <span className="small">
        {totalWords} {t("status_words")} {t("status_total")}
      </span>
      {openDoc !== null && (
        <>
          <span className="sep">|</span>
          <span className="small">
            {openDocWords} {t("status_words")}
          </span>
        </>
      )}
      <span className="sep">|</span>
      <span className="small">[{lang === "en" ? "EN" : "IT"}]</span>
    </footer>
  );
}

export default memo(StatusBar);
