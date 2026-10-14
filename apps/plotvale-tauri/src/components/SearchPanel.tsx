// Project search: plain/regex, case option, saved collections.

import { memo, useEffect, useState } from "react";
import { api } from "../lib/api";
import { useLocale } from "../lib/i18n";
import { useStore } from "../lib/store";
import type { CollectionDef, HitDto } from "../lib/types";

function SearchPanel() {
  const { t } = useLocale();
  const store = useStore();
  const [query, setQuery] = useState("");
  const [regex, setRegex] = useState(false);
  const [caseSens, setCaseSens] = useState(false);
  const [hits, setHits] = useState<HitDto[]>([]);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState("");
  const [collections, setCollections] = useState<CollectionDef[]>([]);
  const [newName, setNewName] = useState("");

  useEffect(() => {
    if (store.project) api.listCollections().then(setCollections).catch(() => {});
  }, [store.project]);

  async function run(pattern: string, rx: boolean, cs: boolean) {
    setError("");
    try {
      setHits(await api.searchProject(pattern, rx, cs));
      setSearched(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : `${e}`);
      setSearched(true);
    }
  }

  async function saveCurrent() {
    const name = newName.trim();
    if (name === "" || query.trim() === "") return;
    try {
      await api.addCollection({ name, pattern: query, regex, case_sensitive: caseSens });
      setCollections(await api.listCollections());
      setNewName("");
    } catch (e) {
      setError(e instanceof Error ? e.message : `${e}`);
    }
  }

  if (!store.project) return <div className="center-pane" />;

  return (
    <div className="center-pane">
      <form
        className="search-bar"
        onSubmit={(e) => {
          e.preventDefault();
          void run(query, regex, caseSens);
        }}
      >
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("search_hint")}
        />
        <button className="btn" type="submit">
          {t("search_button")}
        </button>
      </form>
      <div className="row">
        <label className="check">
          <input type="checkbox" checked={regex} onChange={(e) => setRegex(e.target.checked)} />
          {t("search_regex")}
        </label>
        <label className="check">
          <input type="checkbox" checked={caseSens} onChange={(e) => setCaseSens(e.target.checked)} />
          {t("search_case")}
        </label>
      </div>
      {error !== "" && <p className="error">{error}</p>}
      {searched && (
        <p className="small">
          {t("search_results")}: {hits.length}
        </p>
      )}
      {searched && hits.length === 0 && <p className="hint">{t("search_none")}</p>}
      <div className="hits">
        {hits.map((h, i) => (
          <button
            key={`${h.doc_id}-${h.line_no}-${h.col}-${i}`}
            className="hit"
            onClick={() => {
              store.setSelection(h.doc_id);
              store.setOpenDoc(h.doc_id);
              store.setCenterTab("editor");
            }}
          >
            <strong>{h.title}</strong>
            <span className="small">
              {" "}
              · {t("tab_editor")} L{h.line_no}:C{h.col}
            </span>
            <div className="small dim">{h.excerpt}</div>
          </button>
        ))}
      </div>
      <div className="row">
        <input
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder={t("search_collection_name")}
        />
        <button className="btn small" onClick={() => void saveCurrent()}>
          {t("search_save_current")}
        </button>
      </div>
      {collections.length > 0 && (
        <div className="collections">
          <h4>{t("search_collections")}</h4>
          {collections.map((c) => (
            <div key={c.name} className="row">
              <button
                className="btn small"
                onClick={() => {
                  setQuery(c.pattern);
                  setRegex(c.regex);
                  setCaseSens(c.case_sensitive);
                  void run(c.pattern, c.regex, c.case_sensitive);
                }}
              >
                {c.name}
              </button>
              <span className="spacer" />
              <button
                className="btn small danger"
                onClick={() =>
                  void api
                    .removeCollection(c.name)
                    .then(() => api.listCollections())
                    .then(setCollections)
                    .catch((e: unknown) => setError(e instanceof Error ? e.message : `${e}`))
                }
              >
                {t("search_delete")}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default memo(SearchPanel);
