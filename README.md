# PlotVale — l'anti-Scrivener libero, per scrittori veri

**PlotVale** è un programma gratuito e open source per organizzare e scrivere romanzi,
racconti e testi lunghi: l'alternativa libera a Scrivener, scritta in **Rust puro**,
pensata anche per **scrittori amatoriali** e per chi scrive in **italiano**.

> Progetto indipendente della community, **non affiliato all'ArtCraft team**.
> Ne segue le convenzioni (Rust puro, licenza `MIT OR Apache-2.0`, naming `*Craft`)
> per restare compatibile con l'ecosistema. Vedi `ATTRIBUTION.md`.

## Perché un altro programma di scrittura?

- **Gratis e open source**, per sempre. Nessun abbonamento, nessuna edizione "pro" a pagamento.
- **Zero lock-in**: il tuo romanzo è uno zip di file di testo (Markdown + JSON) con
  [specifica pubblica](docs/format-spec.md). Lo apri anche fra 20 anni con il Blocco note.
- **Snapshot che si possono ripristinare** (sì, davvero), backup automatici, recupero dopo crash.
- **Editing continuo** di più scene selezionate, cartellone con schede vive, obiettivi di
  scrittura, esportazione in TXT / Markdown / HTML / DOCX / EPUB.
- **Interfaccia in italiano e inglese**, tipografia italiana (virgolette «», trattino —),
  conteggio parole secondo le regole della lingua.

## Stato del progetto

Versione **0.1.0** — fondamenta funzionanti (vedi `ROADMAP.md` per le tappe):

- [x] Modello dati, formato `.plotvale`, salvataggio atomico, backup, recupero crash
- [x] Binder gerarchico, editor con stile Markdown inline, inspector, cartellone
- [x] Ricerca globale, snapshot, obiettivi, statistiche
- [x] Export TXT/Markdown/HTML/DOCX/EPUB, CLI `plotvale-cli`
- [ ] PDF impaginato (Typst), controllo ortografico, import da Scrivener

## Compilare ed eseguire

Serve Rust stabile ≥ 1.90 ([installazione](https://rustup.rs/)).

```sh
# interfaccia grafica
cargo run -p plotvale

# riga di comando: crea, verifica, statistiche, esporta
cargo run -p plotvale-cli -- new "Il mio romanzo.plotvale" --title "Il mio romanzo"
cargo run -p plotvale-cli -- stats "Il mio romanzo.plotvale"
cargo run -p plotvale-cli -- verify "Il mio romanzo.plotvale"
cargo run -p plotvale-cli -- export "Il mio romanzo.plotvale" --format epub --out romanzo.epub

# controlli qualità
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p xtask -- check-locales
```

Windows, macOS e Linux sono supportati. La CI compila tutti e tre.

## Struttura del repository

```
crates/model/     modello dati: albero, documenti, schede, obiettivi, conteggio parole
crates/project/   lettura/scrittura .plotvale, salvataggio atomico, backup, migrazioni
crates/search/    indice full-text, regex, ricerche salvate
crates/compile/   motore di composizione: selezione → sezioni → blocchi
crates/export/    renderer TXT/Markdown/HTML/DOCX/EPUB
crates/ui/        interfaccia egui: binder, editor, cartellone, inspector
apps/plotvale/      programma con finestra
apps/plotvale-cli/  programma a riga di comando
xtask/            automazioni (es. verifica parità delle traduzioni)
locales/          stringhe dell'interfaccia (it.json, en.json)
docs/             specifica del formato, architettura, manuale utente
```

## Licenza

Doppia licenza a tua scelta: **MIT** oppure **Apache-2.0**
(`LICENSE-MIT`, `LICENSE-APACHE`). Vedi `NOTICE` e `ATTRIBUTION.md`.
