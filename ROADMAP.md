# ROADMAP.md — PlotVale milestones

A milestone closes only with: `cargo fmt --check`, `clippy -D warnings`,
`cargo test --workspace`, green CI, and a manual run of the real program.

## Milestone 0 — Foundations ✅ (v0.1.0)

Workspace, core crates, lints, CI, licenses, README/AGENTS/ROADMAP, `git init`,
desktop window with binder/editor/inspector layout, EN/IT locales,
CLI with `new/verify/stats/export`.

## Milestone 1 — Project and tree ✅ (v0.1.0)

Model with stable IDs, create/open/save `.plotvale`, binder with reorder,
rename, trash, atomic autosave, rotating backups, crash recovery.
Identical open→save→open round-trip, crash test, tree proptests.

## Milestone 2 — Editor ✅ (v0.1.0)

Fast editor with a Markdown formatting toolbar, live word count on every
keystroke, in-document find, and per-document text mirrors.

## Milestone 3 — Scrivenings + Corkboard + Inspector ✅ (v0.1.0)

Continuous editing of multiple scenes, corkboard with live cards, inspector with
synopsis/status/labels/keywords, character and location sheets, global search.

## Milestone 4 — Revision ✅ (v0.1.0)

Snapshots with diff and restore, inline `[[note: …]]` annotations,
project/session/daily goals, statistics, automatic history.

## Milestone 5 — Compile and export ✅ (v0.1.0)

Single composition engine, preview, profiles and presets,
TXT/Markdown/HTML, DOCX (docx-rs), EPUB (epub-builder).

## Milestone 6 — React + Tauri interface ✅ (v0.2.0)

Desktop app rewritten as React 19 + TypeScript (Vite) in a Tauri v2 window:
the Rust shell owns the project and exposes typed commands, the frontend keeps
no file I/O. Editing, binder, corkboard, inspector, snapshots, goals,
statistics, search, wizard, export (with HTML preview) and EN/IT locales all
moved over; the egui crates were removed.

## Milestone 7 — Polish and distribution (next)

- [ ] Typeset PDF export via Typst (fallback: printable HTML)
- [ ] Offline EN/IT spell checking (Hunspell dictionaries as separate assets)
- [ ] Import from Scrivener (`.scriv`: RTF + `.scrivx`), from DOCX, from a Markdown folder
- [ ] Custom Level-B editor (hidden markers, per-paragraph virtualization)
- [ ] Real-time collaboration-free autosave indicator and crash-safe UI state
- [ ] Optional WASM build, Windows/macOS/Linux installers, release CI
- [ ] Criterion benchmarks (open/save of a 500k-word project < 400 ms)
- [x] Full user manual and public v1 format specification (Sphinx site, sources in `docs/`)
- [ ] Project page
