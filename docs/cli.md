# Command line reference

`plotvale-cli` is the same core as the graphical app, without a window. It is useful for
automating a build, checking that a project is intact, or producing an export from a
script.

```sh
cargo run -p plotvale-cli -- <COMMAND>
```

Run it with `--help` for the built-in summary, and `--version` for the version.

```text
Usage: plotvale-cli <COMMAND>

Commands:
  new     Creates a new project
  verify  Verifies a project's integrity
  stats   Shows project statistics
  export  Exports the manuscript
```

Every command that takes a project accepts **either** a `.plotvale` file **or** a folder
saved in folder mode. The exit code is `0` on success and `1` on failure, with the error
message written to standard error — so a script can rely on it.

## `new`

Creates a new project at the given path.

```sh
cargo run -p plotvale-cli -- new "My novel.plotvale" --title "My novel" --author "Ada"
```

| Argument | Default | Meaning |
|---|---|---|
| `<PATH>` | — | Where to create the project. If it already exists, the command fails. |
| `--title <TITLE>` | `Untitled` | The work's title. |
| `--author <AUTHOR>` | *(empty)* | The author's name. |

## `verify`

Checks that a project is readable and that its internal structure is consistent.

```sh
cargo run -p plotvale-cli -- verify "My novel.plotvale"
```

```text
OK: My novel.plotvale (41230 words, 37 documents, 6 snapshots)
```

It fails, with a clear message, if the file is not a valid PlotVale project, if the tree's
invariants are broken, or if the document was written by a newer `schema_version` than
this build understands.

Use it in a backup script or a CI job: it is the cheap way to know that the project you
are about to archive is not damaged.

## `stats`

Prints the numbers, in the same shape the app's statistics window shows them.

```sh
cargo run -p plotvale-cli -- stats "My novel.plotvale"
```

```text
Title: My novel
Author: Ada
Total words: 41230
Documents with text: 37
Characters (no spaces): 214903
  Chapter 1: 8210 words
  Chapter 2: 9640 words
```

Note that `Documents with text` counts only items that actually contain text, while the
per-chapter lines cover the manuscript.

## `export`

Compiles the manuscript and writes it to a file.

```sh
cargo run -p plotvale-cli -- export "My novel.plotvale" \
    --format epub --out novel.epub --profile ebook
```

| Option | Default | Values |
|---|---|---|
| `<PATH>` | — | Project file or folder. |
| `--format <FORMAT>` | `md` | `md`, `txt`, `html`, `docx`, `epub` |
| `--out <FILE>` | *(required)* | The output file. |
| `--profile <PROFILE>` | `manoscritto` | `manoscritto`, `beta-reader`, `ebook`, `fedelta` |

The profiles are the same ones the graphical app offers — see
[export](user-guide/export.md). The `fedelta` profile includes your `[[note: …]]`
annotations; the others leave them out.

```{warning}
An unrecognized `--profile` value silently falls back to the **manuscript** profile
instead of failing. If you script the CLI, pass a known profile name explicitly.
```

## See also

- [Export](user-guide/export.md) — what the profiles and formats contain.
- [Format specification](format-spec.md) — what `verify` is checking against.
