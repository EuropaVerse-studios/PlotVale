# Keyboard shortcuts

PlotVale keeps the shortcut list short and honest: only what is implemented is listed
here, and everything else is a menu item.

## Global

| Shortcut | Action |
|---|---|
| `Ctrl+S` (`Cmd+S` on macOS) | Save the project. If it has never been saved, opens *Save as…*. |
| `Ctrl+B` (`Cmd+B`) | Bold — in the editor, when it has focus. |
| `Ctrl+I` (`Cmd+I`) | Italic — in the editor, when it has focus. |

`Ctrl+B` and `Ctrl+I` wrap the current selection and **toggle**: pressing them again on
already-formatted text removes the markers. See [formatting](formatting.md).

## In dialogs

| Key | Action |
|---|---|
| `Enter` | Confirm — renaming an item, naming a new one, saving a search. |
| `Escape` | Cancel the dialog. |

## Not bound to commands

A few keystrokes are intentionally left to the text field rather than bound to commands,
so the editor behaves like a text field everywhere:

- undo/redo, cut/copy/paste, and text selection;
- caret movement by words and lines.

```{note}
There is no shortcut for "new scene" or "export": those go through the menus, which offer
the same commands in both *File* and *Project* depending on context.
```

## If a shortcut does not work

Shortcuts that belong to the editor (`Ctrl+B`, `Ctrl+I`) require the editor to have focus —
click inside the text first. `Ctrl+S` works from anywhere in the app.
