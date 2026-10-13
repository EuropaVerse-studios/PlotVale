# Export

*File→ Export…* turns your manuscript into a file you can send to someone. One
composition engine produces the content; the format only decides how it is written out.
What you see in the preview is what every format gets.

## Formats

| Format | Extension | Use it for |
|---|---|---|
| Markdown | `.md` | Keeping your text portable, versioning it, feeding another tool. |
| Plain text | `.txt` | Maximum compatibility, no styling at all. |
| HTML | `.html` | Reading in a browser, printing from a browser. |
| DOCX | `.docx` | Sending the manuscript to someone who works in Word. |
| EPUB | `.epub` | E-readers and ebook stores. |

## Profiles

A profile decides *what* is included and *how* it is shaped, independently of the format.
PlotVale ships four:

| Profile | Intent |
|---|---|
| **manoscritto** *(manuscript)* | The standard submission manuscript. |
| **beta-reader** | The version you hand to a reader for comments. |
| **ebook** | Shaped for EPUB output. |
| **fedelta** *(faithful)* | Keeps everything, including your annotations. |

The profile is chosen in the export window; the CLI takes the same names through
`--profile` (see the [command line reference](../cli.md)).

## Annotations

**Include `[[note: …]]` annotations** decides whether your inline notes
(`[[note: revise this]]`) end up in the exported file. It is off for normal exports and
on in the *fedelta* profile — the name means "fidelity", and it keeps everything.
Selecting a profile aligns the checkbox to that profile's default.

```{tip}
Export twice before sending a manuscript out: once with *beta-reader* for the reader, and
once with *fedelta* for your own archive. The archive copy keeps your notes; the reader's
copy does not.
```

## What never gets exported

Rules that hold for every format and every profile:

- **Research items and notes** are not part of the manuscript.
- **Character and location sheets** are scaffolding for you, not for the reader.
- **Trash** is never touched.
- **Unchecked "Include in export"** items are skipped, even inside the manuscript.

That is the whole point of having a single composition engine: no renderer gets to invent
content, and no format can quietly disagree with another.

## See also

- [Format specification](../format-spec.md) — where your text lives before export.
- [Command line reference](../cli.md) — exporting from a script.
