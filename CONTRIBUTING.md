# CONTRIBUTING.md — how to contribute to PlotVale

Thanks for wanting to contribute! PlotVale is meant to stay approachable,
even if this is your first open-source project.

## Before you start

1. Read `README.md`, `docs/architecture.md`, and `AGENTS.md`.
2. Open (or pick) an issue: no PR without a linked issue, except typos and docs.
3. Unsure about direction? Discuss first, code second.

## Documentation

The user-facing documentation is in `docs/` and is published on Read the Docs. It is
plain Markdown built with Sphinx; `docs/development.md` explains how to build it
locally. Documentation contributions are welcome without an issue, like typo fixes —
but keep the two rules above: English prose, and no claim about the app that the code
does not actually do.

## Code rules

- Rust edition 2024, `cargo fmt` mandatory, `cargo clippy -- -D warnings` clean.
- `unsafe_code = "forbid"`: no `unsafe` anywhere in the workspace.
- No `unwrap()`/`expect()` on user-reachable paths: errors via `thiserror`, with
  clear messages (translated in the UI) for interface-facing failures.
- The core (`model`, `project`, `search`, `compile`, `export`) **must not depend
  on egui**: it has to stay testable without a GUI and compilable to WASM.
- Every user-visible interface string goes through `t("key")` with an entry in
  both `locales/en.json` **and** `locales/it.json`
  (`cargo run -p xtask -- check-locales`).
- Comments, docs, commit messages, and issue text are in **English**; the
  bilingual (EN/IT) strings live only in `locales/` and the UI.
- New features = new tests. Bug fixes = regression tests.

## Workflow

```sh
git checkout -b issue-123-short-description
# ... changes ...
cargo fmt && cargo clippy --workspace -- -D warnings && cargo test --workspace
git commit -m "Area: what changes and why (issue #123)"
```

Small, focused commits with English messages, never generic ("fix", "update").

## Text and tone

PlotVale talks to writers, including beginners: error messages in plain language,
no jargon, no panic. If an advanced feature confuses a hobbyist, it belongs
behind Expert mode or needs a one-line explanation.
