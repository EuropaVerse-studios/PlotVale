# `.plotvale` format specification — v1 (`schema_version = 1`)

A PlotVale project is a **ZIP** archive (deflate compression, Zip64 allowed)
or a **folder** with the same internal layout ("folder mode").
All texts are **UTF-8**. This specification is public: anyone can read and
write PlotVale projects without our program.

## Layout

```
plotvale.json            manifest (required)
struttura.json           node tree (required)
metadati.json            global keywords, custom fields, notes (required)
obiettivi.json           writing goals (required)
testi/<ULID>.md          one document's text (one per textual node)
schede/personaggi.json   character-sheet array
schede/luoghi.json       location-sheet array
snapshot/<ULID-doc>/<id>-<label>.md   saved versions
composizione/profili.json  export profiles
ricerca/<file-name>      research attachments (optional)
```

> Internal entry names (`struttura.json`, `testi/`, …) are frozen for v1
> compatibility and stay as they are; only this documentation is in English.

## Rules

- IDs are **ULIDs** (Crockford base32 strings, e.g. `01J9Z…`). The file name in
  `testi/` is the node's ID: renaming a scene never moves the file.
- The tree in `struttura.json` holds nodes with `parent`/`children`; there are
  always three roots: manuscript, research, trash.
- Nodes with a textual `kind` (`scene`, `note`, `research_item`, `character`,
  `location`) may have a file in `testi/`; if missing, the text is empty.
- Texts are **Markdown** with only two documented extensions:
  - `[[note: text]]` — inline annotation (not part of the manuscript);
    `[[nota: text]]` is accepted as an alias;
  - `[^1]` and `[^1]: text` — footnotes.
- Snapshot file names use only `[A-Za-z0-9_-]`; dates and times in JSON are RFC 3339.
- **Atomic** writes: always write a sibling temporary file
  (`<name>.plotvale.tmp`) and rename it over the original only once the write
  is complete and verified. A `.tmp` newer than the project means a crash:
  offer recovery, never silently overwrite.
- Compatibility: readers finding a `schema_version` higher than the supported one
  must refuse to open with an explicit error, never try to "guess".
