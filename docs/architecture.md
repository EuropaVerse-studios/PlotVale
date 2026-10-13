# Architettura di PlotVale

```
┌─────────────────────────────────────────────────────────┐
│ apps/plotvale (eframe)   apps/plotvale-cli (clap)   │
├─────────────────────────────────────────────────────────┤
│ crates/ui : binder, editor, cartellone, inspector, i18n │
├──────────┬──────────┬───────────┬────────────────────────┤
│ search   │ compile  │  export   │ project                │
│ indice,  │ sezioni, │ md/txt/   │ zip, atomico, backup,  │
│ regex    │ blocchi  │ html/docx │ migrazioni             │
│          │ profili  │ epub      │                        │
├──────────┴──────────┴───────────┴────────────────────────┤
│ crates/model : albero, documenti, schede, conteggio      │
└─────────────────────────────────────────────────────────┘
```

- **model** non sa nulla di file, GUI o rete: solo dati + invarianti + test.
- **project** è l'unico punto che tocca il filesystem (un giorno: IndexedDB su WASM).
- **compile** trasforma una selezione ordinata di nodi in `Compiled`
  (titolo, autore, sezioni tipate, suggerimenti di impaginazione).
- **export** renderizza `Compiled` in ogni formato: stesso contenuto ovunque.
- **ui** tiene uno specchio del testo aperto (`DocView`) e riversa nel modello a ogni
  modifica; l'autosalvataggio scatta dopo 3 s di inattività.
- **Regole di conteggio parole** (`WordCounts`): una parola = sequenza con almeno un
  carattere alfanumerico separata da spazi; `l'ho`, `dell'arte`, i numeri e i trattini
  contano come una parola sola. Stessa regola ovunque (editor, obiettivi, statistiche).
