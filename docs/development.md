# Development

Everything you need to build, test, and check PlotVale, plus the rules that keep the
codebase honest.

## Setup

Install stable Rust with [rustup](https://rustup.rs/). The repository pins the channel in
`rust-toolchain.toml`, so rustup selects the right toolchain automatically. There is
nothing else to install: no Node, no Python, and no system packages for the app itself.

```sh
git clone https://github.com/EuropaVerse-studios/PlotVale
cd plotvale
cargo run -p plotvale
```

## The four gates

Every change must pass these, and CI enforces the same four on Windows, macOS, and Linux:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p xtask -- check-locales
```

What each one protects:

| Gate | It fails when |
|---|---|
| `fmt --check` | Formatting drifts from `rustfmt`. |
| `clippy -D warnings` | Any lint fires — warnings are errors, here and in CI. |
| `test --workspace` | A unit test, proptest, or export golden file fails. |
| `check-locales` | A string exists in `locales/en.json` but not in `locales/it.json`, or vice versa. |

There are no suppressions in this project: `unsafe` is forbidden workspace-wide, and lints
are fixed rather than silenced.

## Workspace layout

```
crates/model/     data model: tree, documents, sheets, goals, word counting
crates/project/   .plotvale reading/writing, atomic saves, backups, migrations
crates/search/    full-text index, regex, saved queries
crates/compile/   composition engine: selection -> sections -> blocks
crates/export/    TXT/Markdown/HTML/DOCX/EPUB renderers
crates/ui/        egui interface: binder, editor, corkboard, inspector
apps/plotvale/    the desktop app
apps/plotvale-cli/ the command line app
xtask/            automation (translation parity check)
locales/          interface strings (en.json, it.json)
docs/             this documentation
```

## Rules that are not negotiable

These come from `AGENTS.md` and exist to keep the project maintainable:

- **The core must not depend on egui.** `model`, `project`, `search`, `compile`, and
  `export` have to stay testable without a GUI and compilable to WASM. Only `crates/ui`
  knows about egui, and only `apps/plotvale` depends on `crates/ui`.
- **No `std::fs` in the core.** All filesystem access goes through `plotvale-project`,
  which is the single place that opens files.
- **One composition engine.** `compile` turns a node selection into sections and blocks;
  no renderer invents content, so every format agrees.
- **Errors via `thiserror`**, with messages a writer can understand. No `unwrap()` or
  `expect()` on a path a user can reach.
- **Format changes bump `schema_version`** and ship with a migration tested against an old
  fixture, never a guess.
- **Every string is translated.** UI text goes through `t("key")` with entries in both
  locale files.

## Testing

- Unit tests in each core crate.
- **Property tests** (`proptest`) for invariants: the tree, the text buffer, and the open
  → save → open round trip must hold for arbitrary inputs.
- **Golden files** for exports: byte-for-byte comparison, so a renderer change that alters
  output is caught rather than shipped.
- A crash test for the recovery path.

Look at `crates/project/tests/roundtrip.rs` and `crates/model/tests/tree_props.rs` for the
shape of the tests to imitate.

## Building this documentation

The site is built with Sphinx and MyST and published on Read the Docs from
`.readthedocs.yaml`. To build it locally:

```sh
python -m venv .venv
.venv/Scripts/pip install -r docs/requirements.txt   # Windows
.venv/bin/pip install -r docs/requirements.txt       # Linux / macOS
```

Then build. `-W` turns warnings into errors, exactly like `-D warnings` does for Rust, and
`-d` keeps the Sphinx cache **outside** the output directory:

```sh
# HTML
.venv/Scripts/python -m sphinx -b html -W -d docs/_build/doctrees docs docs/_build/html

# EPUB and PDF (the offline formats Read the Docs builds)
.venv/Scripts/python -m sphinx -b epub -W -d docs/_build/doctrees docs docs/_build/epub
.venv/Scripts/python -m sphinx -b latex -W -d docs/_build/doctrees docs docs/_build/latex
cd docs/_build/latex && latexmk -pdf plotvale.tex
```

On Linux and macOS use `.venv/bin/python` instead. Then open
`docs/_build/html/index.html` in a browser.

```{note}
Do not leave out `-d`. Without it Sphinx puts its cache in the output directory, and the
EPUB builder then warns about every file it finds there — 25 warnings, zero real problems.
Read the Docs always passes `-d`, so this only bites local builds.
```

```{note}
Read the Docs builds with `fail_on_warning: true`, so a broken internal link, a missing
image, or an unknown `{doc}` reference **fails the docs build**. The published site cannot
silently rot.
```

The pages under *The project* (roadmap, changelog, attribution) are **included from the
repository root**, so there is one copy of each. Edit `ROADMAP.md`, `CHANGELOG.md`, or
`ATTRIBUTION.md` at the root and the documentation picks the change up on the next build.
Remember that the root files are also typeset in the PDF: characters that `pdflatex`
cannot handle need an entry in `latex_elements` in `docs/conf.py`, or the PDF build breaks
while HTML and EPUB stay green.

## See also

- [Architecture](architecture.md) — how the pieces fit, in one diagram.
- [Contributing](contributing.md) — the process for sending a change.
