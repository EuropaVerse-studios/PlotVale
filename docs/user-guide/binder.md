# The binder

The binder is the tree on the left: the whole project as a structure. Click an item to
open it in the editor; right-click it for the actions that apply to it.

## The three roots

Every project has exactly three top-level items, in this order:

| Root | What goes there |
|---|---|
| **Manuscript** | Everything that will be exported: chapters (folders) and scenes. |
| **Research** | Notes, ideas, and references that must never end up in the book. |
| **Trash** | Deleted items, until you empty it. |

Content is kept out of the manuscript by construction: research items, notes, and sheets
are never part of a compiled export, no matter where they live.

## Item kinds

Each item in the tree has a kind, shown by its icon:

| Kind | Icon | Text? |
|---|---|---|
| Manuscript (root) | open book | no |
| Folder (a chapter) | folder | no |
| Scene | page | **yes** |
| Note | pencil note | **yes** |
| Research folder | compass | no |
| Research item | bookmark | **yes** |
| Character sheet | person | no |
| Location sheet | map pin | no |
| Trash | bin | no |

Folders group items; scenes and notes hold text. Sheets hold structured fields — see
[the inspector](inspector.md).

## Creating and organizing items

Right-click any item for the context menu:

- **New scene here**, **New folder here**, **New note here** — inside the item you
  clicked. There are also **New research item**, **New character sheet**, and
  **New location sheet**.
- **Rename** — the title you see in the binder (it is stored as data, never as a file
  name, so renaming is always safe and instant).
- **Move up** / **Move down** — reorder among siblings.
- **Indent (under previous)** / **Outdent (one level up)** — change the nesting level.
- **Move to trash** — send the item and everything inside it to the trash.

```{note}
The binder does not drag and drop. Reordering is done through *Move up*, *Move down*,
*Indent*, and *Outdent*: it is slower than dragging, and it never happens by accident.
```

The *Project* menu offers the same creation commands for the item you have selected, so
you rarely need the right-click menu.

## The trash

Items in the trash are not gone yet:

- **Restore** puts an item back where it was.
- **Delete permanently** removes one item for good, after a confirmation that asks
  whether you meant it.
- **Empty trash** removes everything inside, after its own confirmation.

Nothing is deleted permanently without an explicit confirmation dialog.

## See also

- [The editor](editor.md) — what an open scene looks like.
- [The corkboard](corkboard.md) — a folder's scenes as cards.
- [The inspector](inspector.md) — metadata for the selected item.
