# ROADMAP.md — tappe di PlotVale

Ogni tappa si chiude solo con: `cargo fmt --check`, `clippy -D warnings`,
`cargo test --workspace`, CI verde e prova manuale sul programma reale.

## Tappa 0 — Fondamenta ✅ (v0.1.0)

Workspace, 6 crate, lints, CI, licenze, README/AGENTS/ROADMAP, `git init`,
finestra eframe con layout binder/editor/inspector, locale IT/EN,
CLI con `new/verify/stats/export`.

## Tappa 1 — Progetto e albero ✅ (v0.1.0)

Modello con ID stabili, crea/apri/salva `.plotvale`, Binder con riordino,
rinomina, cestino, autosalvataggio atomico, backup a rotazione, recupero crash.
Round-trip open→save→open identico, test di crash, proptest sull'albero.

## Tappa 2 — Editor ✅ (v0.1.0)

Editor su rope con stile Markdown inline (Livello A), undo/redo di egui,
conteggio parole live, ricerca nel documento, tipografia italiana,
cursore ricordato per documento.

## Tappa 3 — Scrivenings + Cartellone + Inspector ✅ (v0.1.0)

Editing continuo di più scene, cartellone con schede vive, inspector con
sinossi/stato/etichette/keyword, schede personaggi e luoghi, ricerca globale.

## Tappa 4 — Revisione ✅ (v0.1.0)

Snapshot con diff e ripristino, annotazioni inline `[[nota: …]]`,
obiettivi di progetto/sessione/giorno, statistiche, cronologia automatica.

## Tappa 5 — Composizione ed export ✅ (v0.1.0)

Motore di composizione unico, anteprima, profili e preset,
TXT/Markdown/HTML, DOCX (docx-rs), EPUB (epub-builder).

## Tappa 6 — Rifiniture e distribuzione (prossima)

- [ ] Export PDF impaginato via Typst (fallback: HTML stampabile)
- [ ] Controllo ortografico offline IT/EN (dizionari Hunspell come asset separati)
- [ ] Import da Scrivener (`.scriv`: RTF + `.scrivx`), da DOCX, da cartella Markdown
- [ ] Editor custom Livello B (marker nascosti, virtualizzazione per paragrafo)
- [ ] Build WASM opzionale, installer Windows/macOS/Linux, release CI
- [ ] Benchmark con criterion (apertura/salvataggio progetto 500k parole < 400 ms)
- [ ] Manuale utente completo, specifica pubblica v1, pagina progetto
