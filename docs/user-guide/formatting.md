# Formatting

The toolbar above the editor applies Markdown formatting to the text you have selected,
like a word processor's toolbar does.

| Button | What it does | Shortcut |
|---|---|---|
| **B** | **Bold** | `Ctrl+B` |
| *I* | *Italic* | `Ctrl+I` |
| **</>** | Inline code | — |
| **H1 H2 H3** | Heading level 1, 2, or 3 | — |
| **❝** | Block quote | — |
| **•** | Bulleted list | — |
| **annotation** | An annotation note, not exported | — |
| **A+** / **A−** | Bigger / smaller text (12–26 pt) | — |

## How the toolbar behaves

- **It works on the selection.** Select text and press **B**; the markers are wrapped
  around what you selected, and the cursor stays where you expect it.
- **It toggles.** Press **B** again on text that is already bold and the formatting is
  removed. The same applies to the other markers.
- **`A+` / `A−` change how you see the text, not what it says.** The zoom applies to the
  editor's rendering of Markdown (headings scale too); the stored text is untouched, so
  exports are unaffected.
- **Every change goes through the model.** Formatting edits are normal edits: they mark
  the project as changed and are saved by the same autosave.

```{tip}
Annotations are the safest way to leave a note to yourself. They show up while you write
and are excluded from every export profile except the "faithful" one.
```

## Keyboard shortcuts

The complete list lives in [keyboard shortcuts](keyboard-shortcuts.md): `Ctrl+B`,
`Ctrl+I`, and `Ctrl+S`. Both formatting shortcuts require the editor to have focus, and
both use `Cmd` instead of `Ctrl` on macOS.

## Under the hood

The editor stores your scene as Markdown text. The toolbar inserts and removes Markdown
markers around the selection; it is not a hidden-format document model. That is a
deliberate trade-off: the file stays readable in any text editor, at the cost of seeing
the markers when the cursor is inside them.
