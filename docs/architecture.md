# PlotVale architecture

```
┌─────────────────────────────────────────────────────────┐
│ apps/plotvale (eframe)   apps/plotvale-cli (clap)   │
├─────────────────────────────────────────────────────────┤
│ crates/ui: binder, editor, corkboard, inspector, i18n   │
├──────────┬──────────┬───────────┬────────────────────────┤
│ search   │ compile  │  export   │ project                │
│ index,   │ sections,│ md/txt/   │ zip, atomic, backups,  │
│ regex    │ blocks   │ html/docx │ migrations             │
│          │ profiles │ epub      │                        │
├──────────┴──────────┴───────────┴────────────────────────┤
│ crates/model: tree, documents, sheets, counting          │
└─────────────────────────────────────────────────────────┘
```

- **model** knows nothing about files, GUI, or network: data + invariants + tests only.
- **project** is the only place that touches the filesystem (one day: IndexedDB on WASM).
  It also reads and writes the export profiles (`plotvale_compile::ProfilesFile`), so the
  profile settings travel inside the `.plotvale` file: that is the one edge that runs
  against the order drawn in the diagram above.
- **compile** turns an ordered node selection into `Compiled`
  (title, author, typed sections, layout hints).
- **export** renders `Compiled` into every format: same content everywhere.
- **ui** keeps a mirror of the open text (`DocView`) and flushes it into the model on
  every change; autosave fires after 3 s of idle time.
- **Word-count rules** (`WordCounts`): a word = a run with at least one alphanumeric
  character separated by whitespace; `don't`, numbers, and hyphenated forms each count
  as a single word. Same rule everywhere (editor, goals, statistics).
