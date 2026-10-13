# AGENTS.md — regole di lavoro per agenti e contributori

## Direzione del progetto

PlotVale = alternativa libera a Scrivener per romanzieri e scrittori amatoriali.
Priorità: (1) non perdere mai il testo dell'utente, (2) restare semplice per un
dilettante, (3) zero lock-in sul formato. In caso di conflitto, vince l'ordine sopra.

## Architettura (non negoziabile senza discussione)

- Core senza GUI: `model` → `project` → `search`/`compile` → `export`. `ui` dipende
  dal core, mai il contrario. Niente `std::fs` nel core: l'I/O passa per
  `plotvale-project` (astrazione pronta per WASM/IndexedDB).
- Il motore di composizione (`compile`) è l'unica fonte di verità per gli export:
  nessun renderer inventa contenuti.
- Formato `.plotvale` v1: vedi `docs/format-spec.md`. Cambi al formato = bump di
  `schema_version` + migrazione testata su fixture vecchia.

## Qualità

- `cargo fmt --check` pulito; `cargo clippy --workspace -- -D warnings` pulito;
  `unsafe` vietato; niente `unwrap` su input utente.
- Test per ogni crate del core; proptest per invarianti (albero, buffer, round-trip);
  golden file per gli export (confronto byte a byte).
- UI: ogni stringa via `t("chiave")` + voci IT/EN; parità verificata da
  `cargo run -p xtask -- check-locales` e dal test `locales_have_same_keys`.

## Comandi

```sh
cargo run -p plotvale                 # GUI
cargo run -p plotvale-cli -- --help   # CLI
cargo test --workspace                  # test
cargo clippy --workspace -- -D warnings # lint
cargo run -p xtask -- check-locales     # parita' traduzioni
```

## Limiti

- Nessun push, tag, release o deploy senza richiesta esplicita dell'utente.
- Niente nuove dipendenze pesanti senza motivo documentato (build time e WASM contano).
- Niente dizionari/asset con licenza non verificata nel repository (vedi ATTRIBUTION.md).
