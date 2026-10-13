# Specifica del formato `.plotvale` — v1 (`schema_version = 1`)

Un progetto PlotVale è un archivio **ZIP** (compressione deflate, Zip64 ammesso)
oppure una **cartella** con la stessa struttura interna ("modalità cartella").
Tutti i testi sono **UTF-8**. Questa specifica è pubblica: chiunque può leggere e
scrivere progetti PlotVale senza il nostro programma.

## Struttura

```
plotvale.json            manifest (obbligatorio)
struttura.json             albero dei nodi (obbligatorio)
metadati.json              keyword globali, campi personalizzati, note (obbligatorio)
obiettivi.json             obiettivi di scrittura (obbligatorio)
testi/<ULID>.md            testo di un documento (uno per nodo testuale)
schede/personaggi.json     array di schede personaggio
schede/luoghi.json         array di schede luogo
snapshot/<ULID-doc>/<id>-<etichetta>.md   versioni salvate
composizione/profili.json  profili di esportazione
ricerca/<nome-file>        allegati di ricerca (opzionali)
```

## Regole

- Gli ID sono **ULID** (stringhe Crockford base32, es. `01J9Z…`). Il nome del file in
  `testi/` è l'ID del nodo: rinominare una scena non sposta il file.
- L'albero in `struttura.json` contiene i nodi con `parent`/`children`; le radici sono
  sempre tre: manoscritto, ricerca, cestino.
- I nodi con `kind` testuale (`scene`, `note`, `research_item`, `character`,
  `location`) possono avere un file in `testi/`; se manca, il testo è vuoto.
- I testi sono **Markdown** con due sole estensioni documentate:
  - `[[nota: testo]]` — annotazione inline (non fa parte del manoscritto);
  - `[^1]` e `[^1]: testo` — note a piè di pagina.
- I nomi dei file snapshot usano solo `[A-Za-z0-9_-]`; data e ora in RFC 3339 nei JSON.
- Scrittura **atomica**: si scrive sempre un file temporaneo fratello
  (`<nome>.plotvale.tmp`) e lo si rinomina sopra l'originale solo a scrittura
  completata e verificata. Un `.tmp` più recente del progetto indica un crash:
  va proposto il recupero, mai sovrascritto in silenzio.
- Compatibilità: lettori che trovano `schema_version` maggiore di quello supportato
  devono rifiutare l'apertura con errore esplicito, mai tentare di "indovinare".
