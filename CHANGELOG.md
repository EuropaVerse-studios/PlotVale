# CHANGELOG.md

Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Changed
- **The whole interface is now React 19 + TypeScript inside a Tauri v2 window.**
  The Rust shell (`apps/plotvale-tauri/src-tauri`) owns the project and exposes it
  as typed commands; the frontend holds no file I/O. The egui/eframe/dock crates,
  `rfd`, and the whole `crates/ui` crate are gone: the core crates are untouched
  and shared with `plotvale-cli`.
- Word counts run locally in TypeScript with the rules of
  `plotvale-model::count_words`, so typing a 150k-word scene never waits on IPC,
  and counts no longer re-scan the project on every frame.
- New project shortcuts: the desktop app opens a `.plotvale` file or folder given
  as a command-line argument, ready to write (the editor tab comes to the front
  with the first manuscript scene selected).
- Added `editor_find`, `snapshot_title`, `export_preview`, `export_dedication`,
  and `manual_body` to both locale files.
- Removed three dependencies that no code used (`ropey`, `serde_json` in the model,
  `tempfile`), and aligned the workspace version with the released 0.2.0.
- Every repository link now points to the real repository; the architecture notes
  describe the one dependency edge (`project` → `compile`) that the diagram does not
  draw, and the roadmap records the user manual and the format specification as done.
- The README no longer promises Italian typography or language-aware word counting:
  neither exists in the code.
- The README banner is a JPEG now: 339 KB instead of 2.6 MB, the same picture.

## [0.2.0] — 2026-10-13

### Changed
- Renamed the whole project from StoryCraft to **PlotVale** (crates `plotvale-*`,
  binaries `plotvale` / `plotvale-cli`, **`.plotvale`** format).
- The format breaks compatibility with the unreleased 0.1.0: manifest entry
  `plotvale.json`, `.plotvale` extension. Old files are rejected with a clear error.
- New brand dark theme (coral on warm black, cream text) taken from the logo colors;
  modernized interface with cards, spacing, and empty states.
- Logo in assets (`assets/logo.png`, `assets/banner.png`): window icon and
  welcome screen.

## [0.1.0] — 2026-10-13

### Added
- Cargo workspace with 6 crates (`model`, `project`, `search`, `compile`, `export`,
  `ui`), 2 apps (`plotvale`, `plotvale-cli`), and `xtask`.
- `.plotvale` format v1 (spec in `docs/format-spec.md`): zip of Markdown + JSON
  with stable IDs, atomic saves, rotating backups, crash recovery,
  folder mode for Git/sync.
- Graphical interface: binder, editor with Markdown toolbar, inspector,
  corkboard, global search, snapshots, goals, statistics, export.
- Bilingual English/Italian interface with key-parity tests.
- CLI: `new`, `verify`, `stats`, `export` (md/txt/html/docx/epub).
- Export: TXT, Markdown, HTML, DOCX, EPUB from a single composition engine.
- Test suite: unit, proptests (tree, buffer), export golden files, crash test.
- CI on Windows/macOS/Linux (fmt, clippy, test).
