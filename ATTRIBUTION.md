# ATTRIBUTION.md — thanks and third-party licenses

## PlotVale and ArtCraft

PlotVale is an **independent project**: it is not affiliated with, endorsed by,
or part of the ArtCraft team. It is inspired by the ArtCraft philosophy — free
tools for creators, open formats, no lock-in — and adopts the same licensing
habit (dual MIT/Apache-2.0) to stay reuse-friendly. The "ArtCraft" name and
logos are trademarks of their respective owners; nothing in this repository
claims otherwise.

## PlotVale code

All code in this repository is the work of PlotVale contributors and is
distributed under the dual **MIT OR Apache-2.0** license (see `LICENSE-MIT`,
`LICENSE-APACHE`). The logo and banner in `assets/` are original PlotVale
project assets, covered by the same dual license.

The `LICENSE-MIT` and `LICENSE-APACHE` texts are the official texts of the
respective licenses.

## Dependencies (from crates.io; see `Cargo.lock` for exact versions)

| Crate | License | Use in PlotVale |
|---|---|---|
| `eframe`, `egui` | MIT OR Apache-2.0 | window and graphical interface |
| `egui-phosphor` | MIT OR Apache-2.0 | Phosphor icon font (menus, binder, toolbar) |
| `egui_dock` | MIT | dockable panels |
| `rfd` | MIT | native open/save dialogs |
| `ropey` | MIT | editor text buffer |
| `serde`, `serde_json` | MIT OR Apache-2.0 | `.plotvale` format serialization |
| `zip` | MIT | `.plotvale` container |
| `ulid` | MIT OR Apache-2.0 | stable node identifiers |
| `chrono` | MIT OR Apache-2.0 | backup and snapshot timestamps |
| `regex` | MIT OR Apache-2.0 | regular-expression search |
| `clap` | MIT OR Apache-2.0 | command-line interface |
| `docx-rs` | MIT | DOCX export |
| `epub-builder` | **MPL-2.0** | EPUB export |
| `thiserror` | MIT OR Apache-2.0 | error types |
| `proptest`, `tempfile` | MIT OR Apache-2.0 (dev only) | tests |

Note on `epub-builder` (MPL-2.0): it is used as a library dependency; PlotVale's
own code stays MIT/Apache-2.0. The MPL crate's source is publicly available
on crates.io as the license requires.

## Documentation build (Python; not shipped with the app)

These are build-time only: they render the documentation site and are never linked into
the binaries. Versions are pinned in `docs/requirements.txt`.

| Package | License | Use in PlotVale |
|---|---|---|
| `Sphinx` | BSD-2-Clause | documentation generator |
| `myst-parser` | MIT | Markdown support for Sphinx |
| `sphinx-rtd-theme` | MIT | documentation theme |
| `sphinx-copybutton` | MIT | copy button on code blocks |

## Language resources (planned, not yet included)

Future Hunspell spell-check dictionaries (e.g. Italian) carry their own licenses
(LGPL/MPL) and will ship as **separate, optional assets**, never linked into the
binary. See `ROADMAP.md`.
