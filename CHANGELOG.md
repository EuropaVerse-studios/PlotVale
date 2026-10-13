# CHANGELOG.md

Formato ispirato a [Keep a Changelog](https://keepachangelog.com/it/1.0.0/).

## [0.2.0] — 2026-10-13

### Cambiato
- Rinominato tutto il progetto da StoryCraft a **PlotVale** (crate `plotvale-*`, binari `plotvale` / `plotvale-cli`, formato **`.plotvale`**).
- Il formato rompe la compatibilità con la 0.1.0 mai rilasciata: voce manifest `plotvale.json`, estensione `.plotvale`. I vecchi file vengono rifiutati con errore chiaro.
- Nuovo tema scuro brand (corallo su nero caldo, testi crema) dai colori del logo; interfaccia modernizzata con card, spacing e empty-state.
- Logo negli asset (`assets/logo.png`, `assets/banner.png`): icona della finestra e schermata di benvenuto.

## [0.1.0] — 2026-10-13

### Aggiunto
- Workspace Cargo con 6 crate (`model`, `project`, `search`, `compile`, `export`, `ui`),
  2 app (`plotvale`, `plotvale-cli`) e `xtask`.
- Formato `.plotvale` v1 (specifica in `docs/format-spec.md`): zip di Markdown + JSON
  con ID stabili, salvataggio atomico, backup a rotazione, recupero dopo crash,
  modalità cartella per Git/sync.
- Interfaccia grafica (eframe/egui): binder, editor con stile Markdown inline,
  inspector, cartellone, ricerca globale, snapshot, obiettivi, statistiche, export.
- Interfaccia bilingue italiano/inglese con test di parità delle chiavi.
- CLI: `new`, `verify`, `stats`, `export` (md/txt/html/docx/epub).
- Export: TXT, Markdown, HTML, DOCX, EPUB da un unico motore di composizione.
- Suite di test: unit, proptest (albero, buffer), golden file sugli export, crash test.
- CI su Windows/macOS/Linux (fmt, clippy, test).
