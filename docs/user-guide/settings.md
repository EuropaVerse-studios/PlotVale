# Settings and language

PlotVale has very few settings on purpose. There is no preferences dialog full of
toggles; the few choices live where they are used.

## Interface language

*View → Language* switches the interface between **English** and **Italian**. The change
is immediate and the current language is shown in the status bar as `[EN]` or `[IT]`.

```{note}
Every interface string exists in both languages. A test and a CI check
(`cargo run -p xtask -- check-locales`) fail the build if a translation is missing, so the
app cannot ship half-translated.
```

The interface language is separate from the **project language**, which you set in the
new-project wizard (*Details* step) and which is recorded in the project's manifest.

## Expert mode

*View → Expert mode* reveals the advanced options:

- **Custom fields** in the [inspector](inspector.md).
- The sharper tools that would only be noise for a beginner.

It changes no text and no file format. Turn it off again and the extra fields are hidden,
not deleted.

## Editor text size

The text size of the editor is changed with **A+** and **A−** in the toolbar above the
text, between 12 and 26 pt. It is a view setting: exported files are unaffected. See
[formatting](formatting.md).

## Where a project is saved

A project is either a single `.plotvale` file or a folder. The choice is made when you
create the project (*Location* step of the wizard) and can be changed afterwards with
*File → Save as…*:

- **Single file** — one portable file, easy to email. Gets rotating backups.
- **Folder** — plain files and folders; ideal for Git, Dropbox, and any sync tool. No
  rotating backups, because your tooling already provides them.

Both contain exactly the same thing: see the [format specification](../format-spec.md).

## Themes

The interface is dark, using the PlotVale palette taken from the logo, and there is no
light theme to choose from: one theme, done properly, instead of two half-done ones.

## See also

- [Keyboard shortcuts](keyboard-shortcuts.md) — the complete list.
- [Backups and recovery](backups-and-recovery.md) — what the save options imply.
