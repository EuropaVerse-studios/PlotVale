// First-launch onboarding: a short 3-step tour over the welcome screen.
// Shown once (localStorage flag in lib/appinfo); reopenable via Help > tour.

import { useState } from "react";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import { APP_NAME, APP_VERSION, greetingKey } from "../lib/appinfo";
import logoUrl from "../assets/logo.png";
import { pickProjectFile, pickProjectFolder } from "../lib/files";

const FEATURES = [
  { icon: "📚", title: "onboard_f1t", desc: "onboard_f1d" },
  { icon: "✍️", title: "onboard_f2t", desc: "onboard_f2d" },
  { icon: "🗂️", title: "onboard_f3t", desc: "onboard_f3d" },
  { icon: "📦", title: "onboard_f4t", desc: "onboard_f4d" },
] as const;

interface Props {
  onWizard: () => void;
  onClose: () => void;
}

export default function Onboarding({ onWizard, onClose }: Props) {
  const { t } = useLocale();
  const store = useStore();
  const [step, setStep] = useState(0);

  async function openFile() {
    const p = await pickProjectFile(t("open_title_file"), t("filter_project"));
    if (p) {
      await store.doOpen(p);
      onClose();
    }
  }

  async function openFolder() {
    const p = await pickProjectFolder(t("open_title_folder"));
    if (p) {
      await store.doOpen(p);
      onClose();
    }
  }

  return (
    <div className="modal-overlay onboard-overlay" onClick={onClose}>
      <div className="modal onboard" onClick={(e) => e.stopPropagation()}>
        <div className="onboard-dots" aria-hidden>
          {[0, 1, 2].map((i) => (
            <span
              key={i}
              className={i === step ? "dot current" : i < step ? "dot done" : "dot"}
            />
          ))}
        </div>

        <div key={step} className="onboard-step">
          {step === 0 && (
            <>
              <img className="hero-logo" src={logoUrl} alt={APP_NAME} />
              <h2>
                {t("welcome_title")}
              </h2>
              <p className="onboard-greet">
                {t(greetingKey())} — {t("welcome_hello")}
              </p>
              <p className="dim">{t("onboard_sub")}</p>
              <p className="small dim">
                {APP_NAME} v{APP_VERSION}
              </p>
            </>
          )}

          {step === 1 && (
            <>
              <h2>{t("menu_project")}</h2>
              <div className="feat-grid">
                {FEATURES.map((f) => (
                  <div key={f.title} className="feat-card">
                    <span className="feat-icon">{f.icon}</span>
                    <strong>{t(f.title)}</strong>
                    <span className="small dim">{t(f.desc)}</span>
                  </div>
                ))}
              </div>
            </>
          )}

          {step === 2 && (
            <>
              <h2>{t("onboard_start")}</h2>
              <div className="onboard-actions">
                <button className="btn primary" onClick={onWizard}>
                  ✦ {t("welcome_new")}
                </button>
                <button className="btn" onClick={() => void openFile()}>
                  📂 {t("welcome_open_file")}
                </button>
                <button className="btn" onClick={() => void openFolder()}>
                  📁 {t("welcome_open_folder")}
                </button>
              </div>
              <p className="small dim">{t("welcome_tag")}</p>
            </>
          )}
        </div>

        <div className="row end onboard-nav">
          {step > 0 ? (
            <button className="btn" onClick={() => setStep(step - 1)}>
              ← {t("wiz_back")}
            </button>
          ) : (
            <button className="btn" onClick={onClose}>
              {t("onboard_skip")}
            </button>
          )}
          <span className="spacer" />
          {step < 2 ? (
            <button className="btn primary" onClick={() => setStep(step + 1)}>
              {t("wiz_next")} →
            </button>
          ) : (
            <button className="btn primary" onClick={onClose}>
              {t("onboard_start")} ✦
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
