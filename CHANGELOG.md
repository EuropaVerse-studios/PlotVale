# CHANGELOG.md

Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Changed
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
- Graphical interface (eframe/egui): binder, editor with inline Markdown styling,
  inspector, corkboard, global search, snapshots, goals, statistics, export.
- Bilingual English/Italian interface with key-parity tests.
- CLI: `new`, `verify`, `stats`, `export` (md/txt/html/docx/epub).
- Export: TXT, Markdown, HTML, DOCX, EPUB from a single composition engine.
- Test suite: unit, proptests (tree, buffer), export golden files, crash test.
- CI on Windows/macOS/Linux (fmt, clippy, test).
