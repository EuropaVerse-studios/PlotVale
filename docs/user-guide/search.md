# Search

The **Search** tab at the top of the center panel searches the whole project at once —
every scene, note, and research item — and brings back the matching lines.

## Searching

1. Type your query in the search field.
2. Click **Search**.
3. Click a result to open the item it belongs to.

Then click away from the tab to go back to the [editor](editor.md).

## Options

- **Regular expression** — treat the query as a regex instead of plain text. Useful for
  "find every `[[note: …]]`" or for hunting a pattern of dialogue punctuation.
- **Case sensitive** — match capitals exactly, so `Rose` (the character) stops matching
  `rose` (the flower).

The status line reports how many results were found, or says plainly that there were
none.

## Saved searches

The search tab also keeps **saved searches**.

- **Save this search** stores the current query together with its options, under a name
  you choose.
- The saved list sits in the tab: one click re-runs a saved search.
- **Delete** removes a saved search. It does not touch your text.

A practical use: save a query for `TODO`, and you have a project dashboard of open work
that takes one click to reach.

```{note}
Search runs in memory over the project you have open. It does not index anything outside
your project, and it never sends your text anywhere: PlotVale has no network access.
```

## See also

- [Formatting](formatting.md) — leaving annotations for the search to find.
- [Export](export.md) — whether annotations end up in the exported file.
