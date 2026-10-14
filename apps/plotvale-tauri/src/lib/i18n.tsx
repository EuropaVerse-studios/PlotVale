// Internationalization: every user string via t(key), EN/IT entries in the
// shared ../../locales/*.json (parity enforced by `xtask check-locales`).

import { createContext, useCallback, useContext, useState } from "react";
import type { ReactNode } from "react";
// Shared project locales (single source of truth; xtask checks EN/IT parity).
// Vite `server.fs.allow` (see vite.config.ts) permits reading above the app.
import en from "../../../../locales/en.json";
import it from "../../../../locales/it.json";

export type Lang = "en" | "it";

const BUNDLES: Record<Lang, Record<string, string>> = {
  en: en as Record<string, string>,
  it: it as Record<string, string>,
};

interface LocaleCtx {
  lang: Lang;
  setLang: (lang: Lang) => void;
  /** Looks up `key`; falls back to English, then the key itself. */
  t: (key: string) => string;
}

const Ctx = createContext<LocaleCtx>({
  lang: "en",
  setLang: () => {},
  t: (key: string) => key,
});

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>("en");
  const t = useCallback(
    (key: string): string => BUNDLES[lang][key] ?? BUNDLES.en[key] ?? key,
    [lang],
  );
  return <Ctx.Provider value={{ lang, setLang, t }}>{children}</Ctx.Provider>;
}

export function useLocale(): LocaleCtx {
  return useContext(Ctx);
}
