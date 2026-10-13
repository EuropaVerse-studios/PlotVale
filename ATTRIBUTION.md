# ATTRIBUTION.md — ringraziamenti e licenze di terze parti

PlotVale è un progetto indipendente della community, **non affiliato né approvato
dall'ArtCraft team** (il nome e i loghi "ArtCraft" sono marchi dei rispettivi proprietari).
PlotVale segue le convenzioni della suite (Rust puro, licenza doppia MIT/Apache-2.0,
naming `*Craft`) per restare compatibile con l'ecosistema, senza rivendicarne l'ufficialità.

## Codice di PlotVale

Tutto il codice in questo repository è opera dei contributori di PlotVale ed è
distribuito con doppia licenza **MIT OR Apache-2.0** (vedi `LICENSE-MIT`, `LICENSE-APACHE`).

I testi `LICENSE-MIT` e `LICENSE-APACHE` sono i testi ufficiali delle rispettive licenze.

## Dipendenze (da crates.io, vedi `Cargo.lock` per le versioni esatte)

| Crate | Licenza | Uso in PlotVale |
|---|---|---|
| `eframe`, `egui` | MIT OR Apache-2.0 | finestra e interfaccia grafica |
| `egui_dock` | MIT | pannelli agganciabili |
| `rfd` | MIT | dialoghi nativi di apertura/salvataggio |
| `ropey` | MIT | buffer di testo dell'editor |
| `serde`, `serde_json` | MIT OR Apache-2.0 | serializzazione del formato `.plotvale` |
| `zip` | MIT | contenitore `.plotvale` |
| `ulid` | MIT OR Apache-2.0 | identificatori stabili dei nodi |
| `chrono` | MIT OR Apache-2.0 | timestamp di backup e snapshot |
| `regex` | MIT OR Apache-2.0 | ricerca con espressioni regolari |
| `clap` | MIT OR Apache-2.0 | interfaccia a riga di comando |
| `docx-rs` | MIT | esportazione DOCX |
| `epub-builder` | **MPL-2.0** | esportazione EPUB |
| `thiserror` | MIT OR Apache-2.0 | tipi di errore |
| `proptest`, `tempfile` | MIT OR Apache-2.0 (solo dev) | test |

Nota su `epub-builder` (MPL-2.0): è usato come dipendenza libreria; il codice di
PlotVale resta MIT/Apache-2.0. Il sorgente del crate MPL è disponibile pubblicamente
su crates.io come previsto dalla licenza.

## Risorse linguistiche (pianificate, non ancora incluse)

I futuri dizionari ortografici Hunspell (es. italiano) hanno licenze proprie
(LGPL/MPL) e saranno distribuiti come **asset separati e opzionali**, mai linkati nel
binario. Vedi `ROADMAP.md`.
