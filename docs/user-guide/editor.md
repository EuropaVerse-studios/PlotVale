# The editor

The center panel is the editor. Select a scene, a note, or a research item in the binder
and write there. The heading above the text shows the item's title and its word count.

## Writing

Text is plain **Markdown** with inline styling as you type — headings, bold, italic,
quotes, lists, and inline code appear styled while you are still writing them. See
[formatting](formatting.md) for the toolbar and shortcuts.

Two extra notations are specific to PlotVale:

- **Annotations** — `[[note: revise this paragraph]]`. The annotation is visible while
  you write and is left out of exports, unless you explicitly ask for it (see
  [export](export.md)). `[[nota: …]]` works too.
- **Footnotes** — `[^1]` in the text, and `[^1]: the note text` on its own line.

Because the text is stored as Markdown, you never lose the ability to read it with
another program — see the [format specification](../format-spec.md).

## Saving

You do not have to save. PlotVale writes your work **three seconds after you stop
typing**, and immediately on *File → Save* or `Ctrl+S`. The status bar at the bottom
tells you what is going on:

- **● and the project name** — there are unsaved changes.
- **○ and the project name** — everything is written to disk.
- A message appears when something fails, in plain language, never as a stack trace.

Saves are atomic: PlotVale writes a temporary file next to the project and only renames
it over the real one once the write is complete. See
[backups and recovery](backups-and-recovery.md).

```{warning}
Closing the window with unsaved changes does not silently discard them: PlotVale asks
what you want to do (save or discard) and waits for an answer.
```

## The status bar

The bottom strip shows, from left to right:

- the dirty indicator and the open project's file name (or its title, if the project has
  not been saved to a location yet);
- the latest status message — save result, errors, language changes;
- on the right: the current interface language (`[EN]` / `[IT]`), the **total words** of
  the project, and the **word count of the open document**.

Word counting follows one rule everywhere in the app — the editor, the goals, and the
statistics all call the same function. A word is a run of characters containing at least
one alphanumeric character, so `don't`, numbers, and hyphenated forms each count as one
word, and a lone dash counts as none.

## Expert mode

*View → Expert mode* reveals the advanced options, such as custom fields in the
inspector. It changes nothing about your text; it just stops hiding the sharper tools.

## See also

- [Formatting](formatting.md) — the toolbar above the editor.
- [Continuous editing](scrivenings.md) — several scenes as a single flow.
- [Snapshots](snapshots.md) — keep a version before a big revision.
