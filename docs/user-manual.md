# PlotVale user guide

> The app interface is bilingual (English/Italian): switch languages from the
> settings menu. This guide is in English; the in-app strings live in `locales/`.

This is the map of the interface. Each section below has its own page — start here for
the concepts, then jump to what you need.

## The five-minute tour

1. **Create a project**: *File → New project*. The wizard starts you with a
   "Manuscript" folder, an example scene, "Research", and "Trash".
2. **Write**: click a scene in the left-hand binder and write in the center. You can
   use Markdown formatting: `**bold**`, `*italic*`, `# Heading` — or the toolbar above
   the editor.
3. **Organize**: right-click an item to create scenes, folders, and notes, move them
   around, reorder them, or send them to the trash.
4. **Save**: everything autosaves after a few seconds; *File → Save* saves immediately.

## Key concepts

- **Scene** — a chunk of text. A chapter is a folder that holds scenes.
- **Binder** — the left-hand tree: your whole project, chapters, scenes, notes,
  research, character and location sheets.
- **Corkboard** — the card view of the scenes in a folder, with their synopses.
- **Inspector** — the right-hand panel: title, kind, status, label, synopsis, keywords,
  custom fields, snapshots.
- **Scrivenings** — continuous editing of several scenes at once, as one flow of text.
- **Snapshot** — a saved version of a scene's text, taken before a revision; compare and
  restore it at any time.
- **Annotation** — write `[[note: revise this]]` in the text. It is visible while you
  write and never exported.
- **Export** — *File → Export…*, pick a profile and a format: DOCX, EPUB, Markdown,
  HTML, or plain text.

## The pages

**The workspace**

- [The binder](user-guide/binder.md) — the tree, creating and organizing items.
- [The editor](user-guide/editor.md) — writing, Markdown, autosave, word count.
- [Formatting](user-guide/formatting.md) — the toolbar, headings, shortcuts.
- [Continuous editing (scrivenings)](user-guide/scrivenings.md) — several scenes as one.
- [The corkboard](user-guide/corkboard.md) — cards and synopses at a glance.
- [The inspector](user-guide/inspector.md) — metadata, sheets, custom fields.

**While you write**

- [Search](user-guide/search.md) — project-wide search, regex, saved searches.
- [Snapshots](user-guide/snapshots.md) — version history you can restore.
- [Goals and statistics](user-guide/goals-and-statistics.md) — daily and project targets.
- [Backups and recovery](user-guide/backups-and-recovery.md) — how PlotVale protects your
  text.

**Getting the text out**

- [Export](user-guide/export.md) — profiles and formats.
- [Settings and language](user-guide/settings.md) — Expert mode, interface language.
- [Keyboard shortcuts](user-guide/keyboard-shortcuts.md) — the full list.

## Your files are yours

The project is a single `.plotvale` file, but inside there are only text files. With
*Save as* you can also save it as a **folder** (handy for Git and backups). No accounts,
no mandatory cloud, no subscriptions. The container is documented in the
[format specification](format-spec.md).
