# CONTRIBUTING.md — come contribuire a PlotVale

Grazie per voler contribuire! PlotVale è pensato per restare semplice da
approcciare, anche per chi è al primo progetto open source.

## Prima di iniziare

1. Leggi `README.md`, `docs/architecture.md` e `AGENTS.md`.
2. Apri (o scegli) una issue: niente PR senza issue collegata, salvo typo e docs.
3. Per dubbi sulla direzione, la discussione viene prima del codice.

## Regole del codice

- Rust edition 2024, `cargo fmt` obbligatorio, `cargo clippy -- -D warnings` pulito.
- `unsafe_code = "forbid"`: niente `unsafe` in tutto il workspace.
- Niente `unwrap()`/`expect()` su percorsi raggiungibili dall'utente: errori con
  `thiserror`, messaggi chiari (e tradotti) nell'interfaccia.
- Il core (`model`, `project`, `search`, `compile`, `export`) **non dipende da egui**:
  deve restare testabile senza GUI e compilabile per WASM.
- Ogni stringa visibile nell'interfaccia passa per `t("chiave")` con voce in
  `locales/it.json` **e** `locales/en.json` (`cargo run -p xtask -- check-locales`).
- Nuove funzioni = nuovi test. Bug fix = test di regressione.

## Flusso di lavoro

```sh
git checkout -b issue-123-breve-descrizione
# ... modifiche ...
cargo fmt && cargo clippy --workspace -- -D warnings && cargo test --workspace
git commit -m "Area: cosa cambia e perche' (issue #123)"
```

Commit piccoli e tematici, messaggi in italiano o inglese, mai generici ("fix", "update").

## Testo e tono

PlotVale parla a scrittori, anche principianti: messaggi d'errore in linguaggio
chiaro, niente gergo, niente panico. Se una funzione avanzata confonde un dilettante,
va nascosta in modalità Esperto o spiegata in una riga.
