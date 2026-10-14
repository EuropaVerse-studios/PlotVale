# PlotVale architecture

```
┌─────────────────────────────────────────────────────────┐
│ apps/plotvale-tauri: React 19 UI in a Tauri v2 window    │
│ apps/plotvale-cli (clap)                                 │
├─────────────────────────────────────────────────────────┤
│ src-tauri: state + #[tauri::command] API over the core   │
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
- **The desktop shell** (`apps/plotvale-tauri/src-tauri`) owns the open `Project`, the
  profiles, and the save path behind a `Mutex`, and exposes them as typed commands.
  The React frontend never touches the filesystem.
- **The React interface** keeps its own mirror of the open texts for instant typing and
  flushes each document to Rust debounced (600 ms); autosave writes to disk after 3 s of
  idle time. Word counts run locally in TypeScript with the same rules as
  `plotvale-model::count_words`, so no keystroke waits on IPC.
- **Word-count rules** (`WordCounts`): a word = a run with at least one alphanumeric
  character separated by whitespace; `don't`, numbers, and hyphenated forms each count
  as a single word. Same rule everywhere (editor, goals, statistics).
