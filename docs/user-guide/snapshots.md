# Snapshots

A snapshot is a saved copy of a document's text, taken before you change it. It is the
safety net for a revision: you can compare and restore it later, even after days of
editing.

Snapshots live in the [inspector](inspector.md), which lists the ones belonging to the
selected item.

## Taking a snapshot

Click **Take snapshot** in the inspector before a big change — restructuring a chapter,
rewriting a scene, cutting a character. You give it a label; the date and time are
recorded with it.

Labels are stored in a filesystem-safe form and the dates are RFC 3339, as required by
the [format specification](../format-spec.md).

## Comparing

**Compare** opens a diff of the snapshot against the current text: added lines and
removed lines. This is the fastest way to answer "what exactly did I change in this
scene?" without trusting your memory.

## Restoring and deleting

- **Restore** puts the snapshot's text back into the document. Take a snapshot of the
  *current* text first if you might want it back — restoring replaces the text.
- **Delete** removes one snapshot permanently, after confirmation.

```{tip}
Snapshots are per document, not per project, and they are stored inside the project file.
A project stays self-contained: give someone the `.plotvale` file and they get your
snapshots too.
```

## See also

- [Backups and recovery](backups-and-recovery.md) — the automatic safety net, one level
  below snapshots.
- [Export](export.md) — getting a version out of the app entirely.
