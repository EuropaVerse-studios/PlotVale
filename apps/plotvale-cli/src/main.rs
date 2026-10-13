//! PlotVale a riga di comando: crea, verifica, statistiche, esporta.
//!
//! Stesso core della GUI, senza finestra: utile per automazioni, CI e per
//! controllare l'integrità dei propri progetti.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use plotvale_compile::{CompileProfile, compile, expand_selection};
use plotvale_export::{ExportFormat, export_to_file};
use plotvale_model::{Project, count_words};
use plotvale_project::{SaveOptions, open_any, save_any};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// PlotVale da terminale.
#[derive(Debug, Parser)]
#[command(name = "plotvale-cli", version = APP_VERSION)]
struct Cli {
    /// Comando da eseguire.
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Crea un nuovo progetto.
    New {
        /// Percorso del file .plotvale da creare.
        path: PathBuf,
        /// Titolo dell'opera.
        #[arg(long, default_value = "Senza titolo")]
        title: String,
        /// Autore.
        #[arg(long, default_value = "")]
        author: String,
    },
    /// Verifica l'integrità di un progetto.
    Verify {
        /// Percorso del progetto (file o cartella).
        path: PathBuf,
    },
    /// Mostra le statistiche del progetto.
    Stats {
        /// Percorso del progetto (file o cartella).
        path: PathBuf,
    },
    /// Esporta il manoscritto.
    Export {
        /// Percorso del progetto (file o cartella).
        path: PathBuf,
        /// Formato: md, txt, html, docx, epub.
        #[arg(long, default_value = "md")]
        format: String,
        /// File di output.
        #[arg(long)]
        out: PathBuf,
        /// Profilo: manoscritto, beta-reader, ebook, fedelta.
        #[arg(long, default_value = "manoscritto")]
        profile: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("plotvale-cli: errore: {e}");
            1
        }
    };
    std::process::exit(code);
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::New {
            path,
            title,
            author,
        } => {
            if path.exists() {
                return Err(format!("esiste già: {}", path.display()));
            }
            let mut project = Project::new(&title, &author, APP_VERSION);
            let profiles = plotvale_compile::ProfilesFile::default();
            save_any(&mut project, &profiles, &path, &SaveOptions::default())
                .map_err(|e| format!("creazione non riuscita: {e}"))?;
            println!("creato {}", path.display());
            Ok(())
        }
        Command::Verify { path } => {
            let opened = open_any(&path).map_err(|e| format!("progetto non valido: {e}"))?;
            opened
                .project
                .tree
                .check_invariants()
                .map_err(|e| format!("struttura incoerente: {e}"))?;
            let docs = opened.project.docs.len();
            println!(
                "OK: {} ({} parole, {} documenti, {} snapshot)",
                path.display(),
                opened.project.total_words(),
                docs,
                opened.project.snapshots.len()
            );
            Ok(())
        }
        Command::Stats { path } => {
            let opened = open_any(&path).map_err(|e| format!("apertura non riuscita: {e}"))?;
            let project = &opened.project;
            println!("Titolo: {}", project.manifest.title);
            println!("Autore: {}", project.manifest.author);
            println!("Parole totali: {}", project.total_words());
            let mut docs = 0;
            let mut chars = 0;
            for id in project.tree.ids() {
                let text = project.doc_text(id);
                if !text.trim().is_empty() {
                    docs += 1;
                }
                chars += count_words(text).characters_no_spaces;
            }
            println!("Documenti con testo: {docs}");
            println!("Caratteri (spazi esclusi): {chars}");
            for child in project
                .tree
                .children_of(project.tree.draft_root)
                .unwrap_or_default()
            {
                let name = project
                    .tree
                    .get(child)
                    .map(|n| n.title.clone())
                    .unwrap_or_default();
                println!("  {name}: {} parole", project.subtree_words(child));
            }
            Ok(())
        }
        Command::Export {
            path,
            format,
            out,
            profile,
        } => {
            let opened = open_any(&path).map_err(|e| format!("apertura non riuscita: {e}"))?;
            let format: ExportFormat = format.parse().map_err(|e: String| e)?;
            let mut prof = match profile.as_str() {
                "beta-reader" => CompileProfile::beta_reader(),
                "ebook" => CompileProfile::ebook(),
                "fedelta" => CompileProfile::faithful(),
                _ => CompileProfile::manuscript(),
            };
            if profile == "fedelta" {
                prof.keep_notes = true;
            }
            let ids = expand_selection(&opened.project, &[]);
            let compiled =
                compile(&opened.project, &ids, &prof).map_err(|e| format!("composizione: {e}"))?;
            export_to_file(&compiled, format, &out)
                .map_err(|e| format!("esportazione non riuscita: {e}"))?;
            println!("esportato {}", out.display());
            Ok(())
        }
    }
}
