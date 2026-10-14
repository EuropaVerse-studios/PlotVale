# Getting started

This page takes you from an empty machine to text on the page. It takes about five
minutes, most of which is compilation.

## What you need

- **Rust, stable, 1.90 or newer.** Install it with [rustup](https://rustup.rs/):

  ```sh
  rustup toolchain install stable
  ```

  The repository pins the channel in `rust-toolchain.toml`, so rustup picks the right
  toolchain automatically once you are inside the project.

- **Node.js 20 or newer** for the desktop interface (React + Tauri). The CLI needs
  only Rust.

PlotVale has no accounts and no network access at runtime: your project stays on
your disk.

```{note}
On Linux, Tauri needs the usual WebKitGTK and windowing libraries. If the window
fails to appear, install your distribution's `webkit2gtk` (4.1) and X11/Wayland
development packages, then re-run the app.
```

## Build and run

The desktop app lives in `apps/plotvale-tauri`:

```sh
cd apps/plotvale-tauri
npm install            # once
npm run tauri dev      # opens the window with hot reload
npm run tauri build    # release binary and installers
```

Prefer the terminal? The CLI needs no Node:

```sh
cargo run -p plotvale-cli -- --help
```

The first build compiles the whole workspace and takes a few minutes. Later builds are
incremental and fast.

## Try the command line

The CLI uses exactly the same core as the graphical app, without a window — handy for
automation and for checking that a project is intact.

```sh
cargo run -p plotvale-cli -- --help
```

```text
Usage: plotvale-cli <COMMAND>

Commands:
  new     Creates a new project
  verify  Verifies a project's integrity
  stats   Shows project statistics
  export  Exports the manuscript
```

A complete round trip, without ever opening a window:

```sh
cargo run -p plotvale-cli -- new "My novel.plotvale" --title "My novel" --author "Ada"
cargo run -p plotvale-cli -- verify "My novel.plotvale"
cargo run -p plotvale-cli -- stats "My novel.plotvale"
cargo run -p plotvale-cli -- export "My novel.plotvale" --format epub --out novel.epub
```

See the [command line reference](cli.md) for every flag and profile.

## Your first project in the app

1. **Launch PlotVale** and click **Create a new project** on the welcome screen (or use
   *File→ New project*).
2. **Walk the wizard.** Five short steps: *Template* (Novel, Short story, or Blank),
   *Details* (title, author, language), *Structure* (chapters and scenes per chapter),
   *Features* (daily goal, total goal, Expert mode), *Location* (a single `.plotvale`
   file, or a folder for Git and syncing). You can leave the location empty and decide
   at the first save.
3. **Write.** Click a scene in the left-hand binder and start typing in the middle. The
   status bar tells you when the text is saved — it happens on its own after a few
   seconds of inactivity.
4. **Format.** Use the toolbar above the editor for **bold**, *italic*, inline code,
   headings, quotes, and lists, or the `Ctrl+B` / `Ctrl+I` shortcuts. The toolbar works
   on the selected text and toggles off if you press it twice.
5. **Export.** *File→ Export…*, pick a preset and a format: DOCX for your editor, EPUB
   for e-readers, Markdown or HTML for anything else.

```{tip}
Not sure where a feature lives? *Help→ Quick guide* opens a short tour, and the
[user guide](user-manual.md) documents every panel.
```

## Where your text lives

A project is a **single `.plotvale` file** — a zip archive that contains only plain
text: Markdown documents and JSON metadata. With *File→ Save as…* you can save it as a
**folder** instead, which works better with Git and file sync. Either way there is
nothing proprietary inside: the [format specification](format-spec.md) is public.

## Developer checks

Before sending a patch, run the same commands CI runs:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p xtask -- check-locales
```

The [development guide](development.md) explains what each one protects.

## Next steps

- [User guide](user-manual.md) — every panel, explained.
- [Command line reference](cli.md) — the CLI in full.
- [Architecture](architecture.md) — how the crates fit together.
- [Contributing](contributing.md) — the rules for a patch.
