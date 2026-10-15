# AGENTS.md — working rules for agents and contributors

## Project direction

PlotVale = a libre Scrivener alternative for novelists and hobbyist writers.
Priorities: (1) never lose the user's text, (2) stay simple for a beginner,
(3) zero format lock-in. In case of conflict, the order above wins.

## Architecture (non-negotiable without discussion)

- GUI-less core: `model` → `project` → `search`/`compile` → `export`. The desktop
  app depends on the core, never the other way around: the Tauri shell
  (`src-tauri`) holds state and exposes commands, the React frontend (`src/`)
  holds no file I/O. No `std::fs` in the core: I/O goes through
  `plotvale-project` (abstraction ready for WASM/IndexedDB).
- Single project: one Cargo workspace (`crates/*`, `src-tauri`, `cli`, `xtask`,
  one `Cargo.lock`) plus one `package.json` at the root for the frontend.
- Interface: React 19 + TypeScript + Vite inside a Tauri v2 window; every string
  via `t("key")` and the shared `locales/*.json`. Editor text stays local and
  syncs debounced, so large scenes never block on IPC.
- The composition engine (`compile`) is the single source of truth for exports:
  no renderer invents content.
- `.plotvale` format v1: see `docs/format-spec.md`. Format changes = bump
  `schema_version` + migration tested against an old fixture.

## Quality

- `cargo fmt --check` clean; `cargo clippy --workspace -- -D warnings` clean;
  `unsafe` forbidden; no `unwrap` on user input.
- Tests for every core crate; proptests for invariants (tree, buffer, round-trip);
  golden files for exports (byte-for-byte comparison).
- UI: every string via `t("key")` + EN/IT entries; parity enforced by
  `cargo run -p xtask -- check-locales` and the `locales_have_same_keys` test.

## Commands

```sh
npm install                             # frontend deps (project root)
npm run tauri dev                       # desktop app (React + Tauri)
npm run build                           # typecheck + frontend bundle
cd src-tauri && cargo clippy --lib      # backend lint
cargo run -p plotvale-cli -- --help     # CLI
cargo test --workspace                  # tests
cargo clippy --workspace -- -D warnings # core lint
cargo run -p xtask -- check-locales     # translation parity
```

## Limits

- No push, tag, release, or deploy without the user's explicit request.
- No heavy new dependencies without documented reason (build time and WASM matter).
- No dictionaries/assets with unverified licenses in the repository (see ATTRIBUTION.md).
