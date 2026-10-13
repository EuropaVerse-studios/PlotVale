//! PlotVale on the command line: create, verify, stats, export.
//!
//! Same core as the GUI, without a window: handy for automation, CI, and
//! checking the integrity of your projects.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use plotvale_compile::{CompileProfile, compile, expand_selection};
use plotvale_export::{ExportFormat, export_to_file};
use plotvale_model::{Project, count_words};
use plotvale_project::{SaveOptions, open_any, save_any};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// PlotVale from the terminal.
#[derive(Debug, Parser)]
#[command(name = "plotvale-cli", version = APP_VERSION)]
struct Cli {
    /// Command to run.
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Creates a new project.
    New {
        /// Path of the .plotvale file to create.
        path: PathBuf,
        /// Title of the work.
        #[arg(long, default_value = "Untitled")]
        title: String,
        /// Author.
        #[arg(long, default_value = "")]
        author: String,
    },
    /// Verifies a project's integrity.
    Verify {
        /// Project path (file or folder).
        path: PathBuf,
    },
    /// Shows project statistics.
    Stats {
        /// Project path (file or folder).
        path: PathBuf,
    },
    /// Exports the manuscript.
    Export {
        /// Project path (file or folder).
        path: PathBuf,
        /// Format: md, txt, html, docx, epub.
        #[arg(long, default_value = "md")]
        format: String,
        /// Output file.
        #[arg(long)]
        out: PathBuf,
        /// Profile: manoscritto, beta-reader, ebook, fedelta.
        #[arg(long, default_value = "manoscritto")]
        profile: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("plotvale-cli: error: {e}");
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
                return Err(format!("already exists: {}", path.display()));
            }
            let mut project = Project::new(&title, &author, APP_VERSION);
            let profiles = plotvale_compile::ProfilesFile::default();
            save_any(&mut project, &profiles, &path, &SaveOptions::default())
                .map_err(|e| format!("creation failed: {e}"))?;
            println!("created {}", path.display());
            Ok(())
        }
        Command::Verify { path } => {
            let opened = open_any(&path).map_err(|e| format!("invalid project: {e}"))?;
            opened
                .project
                .tree
                .check_invariants()
                .map_err(|e| format!("inconsistent tree: {e}"))?;
            let docs = opened.project.docs.len();
            println!(
                "OK: {} ({} words, {} documents, {} snapshots)",
                path.display(),
                opened.project.total_words(),
                docs,
                opened.project.snapshots.len()
            );
            Ok(())
        }
        Command::Stats { path } => {
            let opened = open_any(&path).map_err(|e| format!("open failed: {e}"))?;
            let project = &opened.project;
            println!("Title: {}", project.manifest.title);
            println!("Author: {}", project.manifest.author);
            println!("Total words: {}", project.total_words());
            let mut docs = 0;
            let mut chars = 0;
            for id in project.tree.ids() {
                let text = project.doc_text(id);
                if !text.trim().is_empty() {
                    docs += 1;
                }
                chars += count_words(text).characters_no_spaces;
            }
            println!("Documents with text: {docs}");
            println!("Characters (no spaces): {chars}");
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
                println!("  {name}: {} words", project.subtree_words(child));
            }
            Ok(())
        }
        Command::Export {
            path,
            format,
            out,
            profile,
        } => {
            let opened = open_any(&path).map_err(|e| format!("open failed: {e}"))?;
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
                compile(&opened.project, &ids, &prof).map_err(|e| format!("compile: {e}"))?;
            export_to_file(&compiled, format, &out).map_err(|e| format!("export failed: {e}"))?;
            println!("exported {}", out.display());
            Ok(())
        }
    }
}
