# Backups and recovery

PlotVale's first priority is that you never lose your text. Three mechanisms work
together, and they are all automatic.

## 1. Autosave

Your text is written to disk **three seconds after you stop typing**. You can also force
it with *File→ Save* or `Ctrl+S`. The status bar shows a **filled dot** while there are changes
and a **hollow dot** once everything is on disk — see [the editor](editor.md).

## 2. Atomic writes

A save never overwrites your project in place. PlotVale writes the new content to a
temporary file next to the project (`<name>.plotvale.tmp`), verifies it, and only then
renames it over the real file.

This means a crash, a full disk, or a pulled power cable cannot leave you with a
half-written project: either the old file is intact, or the new one is complete.

## 3. Rotating backups

Saving a single-file `.plotvale` project also keeps previous versions in a
**`<name>-backups/`** folder next to the project. The last **five** are kept; older ones
are rotated out automatically.

Folder-mode projects do not get PlotVale backups: a folder of plain Markdown and JSON is
exactly what Git and your own backup tooling want, so use those instead.

## Recovering after a crash

If PlotVale finds a temporary file **newer than the project itself**, it means a save did
not finish — the app crashed or the machine went down mid-write. When you open the
project you are offered the recovered content rather than having anything overwritten
silently.

After recovering, the status bar says:

> Recovered after unexpected shutdown — save to consolidate

Save, and the recovery is written into the project properly.

```{warning}
Never delete a `.plotvale.tmp` file by hand. It is either an in-progress write or the only
copy of text that a crash interrupted — the one case where it is the newest data you have.
```

## Closing with unsaved changes

Closing the window or quitting with unsaved changes stops and asks: **Save** or
**Discard**. PlotVale does not guess, and it does not silently throw work away.

## See also

- [Export](export.md) — keeping a copy in a different format, outside the app.
- [Snapshots](snapshots.md) — your own version history, per document.
- [Format specification](../format-spec.md) — the atomic-write rule, written down.
