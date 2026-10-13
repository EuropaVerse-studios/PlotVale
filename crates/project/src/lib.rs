//! `plotvale-project` — l'unico punto che tocca il filesystem.
//!
//! Legge e scrive progetti `.plotvale` (ZIP di Markdown + JSON, vedi
//! `docs/format-spec.md`) e cartelle progetto con la stessa struttura.
//! Il salvataggio è **atomico** (file temporaneo + rename), con **backup a
//! rotazione** e **recupero dopo crash**. Il resto del programma non apre mai file.

use chrono::Utc;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use plotvale_compile::ProfilesFile;
use plotvale_model::{NodeId, Project, SCHEMA_VERSION};

/// Versione del formato che sappiamo leggere e scrivere.
pub use plotvale_model::SCHEMA_VERSION as FORMAT_VERSION;

/// Nomi delle voci dentro il bundle (vedi `docs/format-spec.md`).
pub mod entries {
    /// Manifest del progetto.
    pub const MANIFEST: &str = "plotvale.json";
    /// Albero dei nodi.
    pub const TREE: &str = "struttura.json";
    /// Metadati globali.
    pub const META: &str = "metadati.json";
    /// Obiettivi di scrittura.
    pub const TARGETS: &str = "obiettivi.json";
    /// Schede personaggio.
    pub const CHARACTERS: &str = "schede/personaggi.json";
    /// Schede luogo.
    pub const LOCATIONS: &str = "schede/luoghi.json";
    /// Profili di composizione.
    pub const PROFILES: &str = "composizione/profili.json";
    /// Prefisso dei testi (`testi/<ULID>.md`).
    pub const TEXTS_PREFIX: &str = "testi/";
    /// Prefisso degli snapshot (`snapshot/<ULID-doc>/<id>-<etichetta>.md`).
    pub const SNAPSHOTS_PREFIX: &str = "snapshot/";
}

/// Errori di apertura e salvataggio. Mai un panic su file utente.
#[derive(Debug, Error)]
pub enum ProjectError {
    /// Errore di I/O (permessi, disco pieno, …).
    #[error("errore di lettura/scrittura: {0}")]
    Io(#[from] std::io::Error),
    /// Archivio ZIP illeggibile o troncato.
    #[error("file danneggiato o non valido: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// JSON interno non valido.
    #[error("dati interni non validi in {entry}: {message}")]
    Json {
        /// Voce dell'archivio coinvolta.
        entry: String,
        /// Dettaglio.
        message: String,
    },
    /// Voce obbligatoria mancante.
    #[error("voce mancante nel progetto: {0}")]
    MissingEntry(String),
    /// Versione del formato più recente di quella supportata.
    #[error(
        "progetto creato con una versione più recente (schema {found}, supportato {supported}): aggiorna PlotVale"
    )]
    UnsupportedVersion {
        /// Versione trovata nel file.
        found: u32,
        /// Versione supportata.
        supported: u32,
    },
    /// Struttura dati incoerente.
    #[error("progetto incoerente: {0}")]
    Inconsistent(String),
}

/// Opzioni di salvataggio.
#[derive(Clone, Debug)]
pub struct SaveOptions {
    /// Quanti backup tenere a rotazione (0 = nessun backup).
    pub backup_count: usize,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self { backup_count: 5 }
    }
}

/// Un progetto aperto, con i suoi profili di composizione.
#[derive(Clone, Debug)]
pub struct OpenedProject {
    /// Dati del progetto.
    pub project: Project,
    /// Profili di esportazione.
    pub profiles: ProfilesFile,
}

// ---------------------------------------------------------------------------
// Apertura
// ---------------------------------------------------------------------------

/// Apre un progetto da file `.plotvale` o da cartella (rilevato dal percorso).
pub fn open_any(path: &Path) -> Result<OpenedProject, ProjectError> {
    if path.is_dir() {
        open_folder(path)
    } else {
        open_zip(path)
    }
}

/// Apre un bundle `.plotvale`.
pub fn open_zip(path: &Path) -> Result<OpenedProject, ProjectError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(ProjectError::Zip)?;
    read_from_zip(&mut archive)
}

/// Apre una cartella progetto (modalità cartella).
pub fn open_folder(dir: &Path) -> Result<OpenedProject, ProjectError> {
    let read_json = |name: &str| -> Result<Vec<u8>, ProjectError> {
        fs::read(dir.join(name)).map_err(|_| ProjectError::MissingEntry(name.to_string()))
    };
    let manifest: plotvale_model::Manifest =
        parse_json(entries::MANIFEST, &read_json(entries::MANIFEST)?)?;
    check_version(&manifest)?;
    let tree = parse_json(entries::TREE, &read_json(entries::TREE)?)?;
    let meta = parse_json(entries::META, &read_json(entries::META)?)?;
    let targets = parse_json(entries::TARGETS, &read_json(entries::TARGETS)?)?;
    let characters = parse_json(entries::CHARACTERS, &read_json(entries::CHARACTERS)?)?;
    let locations = parse_json(entries::LOCATIONS, &read_json(entries::LOCATIONS)?)?;
    let profiles = match fs::read(dir.join(entries::PROFILES)) {
        Ok(bytes) => parse_json(entries::PROFILES, &bytes)?,
        Err(_) => ProfilesFile::default(),
    };
    let mut project = Project {
        manifest,
        tree,
        docs: Default::default(),
        meta,
        sheets: plotvale_model::Sheets {
            characters,
            locations,
        },
        targets,
        snapshots: Default::default(),
        snapshot_texts: Default::default(),
    };
    // Testi: testi/<ULID>.md
    let texts_dir = dir.join("testi");
    if texts_dir.is_dir() {
        for entry in fs::read_dir(&texts_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(id_str) = name.strip_suffix(".md")
                && let Ok(id) = id_str.parse::<NodeId>()
            {
                let text = fs::read_to_string(entry.path())?;
                project.docs.insert(
                    id,
                    plotvale_model::Doc {
                        text,
                        updated_at: project.manifest.updated_at.clone(),
                    },
                );
            }
        }
    }
    // Snapshot: snapshot/<doc>/<file>.md
    let snaps_dir = dir.join("snapshot");
    project.snapshots = read_snapshot_metas_folder(&snaps_dir)?;
    project.snapshot_texts = read_snapshot_texts_folder(&snaps_dir)?;
    check_consistency(&project)?;
    Ok(OpenedProject { project, profiles })
}

fn read_from_zip(archive: &mut ZipArchive<File>) -> Result<OpenedProject, ProjectError> {
    let read_entry =
        |archive: &mut ZipArchive<File>, name: &str| -> Result<Vec<u8>, ProjectError> {
            let mut file = archive
                .by_name(name)
                .map_err(|_| ProjectError::MissingEntry(name.to_string()))?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok(bytes)
        };
    let manifest: plotvale_model::Manifest =
        parse_json(entries::MANIFEST, &read_entry(archive, entries::MANIFEST)?)?;
    check_version(&manifest)?;
    let tree = parse_json(entries::TREE, &read_entry(archive, entries::TREE)?)?;
    let meta = parse_json(entries::META, &read_entry(archive, entries::META)?)?;
    let targets = parse_json(entries::TARGETS, &read_entry(archive, entries::TARGETS)?)?;
    let characters = parse_json(
        entries::CHARACTERS,
        &read_entry(archive, entries::CHARACTERS)?,
    )?;
    let locations = parse_json(
        entries::LOCATIONS,
        &read_entry(archive, entries::LOCATIONS)?,
    )?;
    let profiles = match archive.by_name(entries::PROFILES) {
        Ok(mut file) => {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            parse_json(entries::PROFILES, &bytes)?
        }
        Err(_) => ProfilesFile::default(),
    };
    let mut project = Project {
        manifest,
        tree,
        docs: Default::default(),
        meta,
        sheets: plotvale_model::Sheets {
            characters,
            locations,
        },
        targets,
        snapshots: Default::default(),
        snapshot_texts: Default::default(),
    };
    // Raccoglie i nomi prima (il borrow dell'archivio non permette altro).
    let names: Vec<String> = archive.file_names().map(str::to_string).collect();
    for name in &names {
        if let Some(rest) = name.strip_prefix(entries::TEXTS_PREFIX) {
            if let Some(id_str) = rest.strip_suffix(".md")
                && let Ok(id) = id_str.parse::<NodeId>()
            {
                let bytes = read_entry(archive, name)?;
                let text = String::from_utf8(bytes).map_err(|e| ProjectError::Json {
                    entry: name.clone(),
                    message: format!("testo non UTF-8: {e}"),
                })?;
                project.docs.insert(
                    id,
                    plotvale_model::Doc {
                        text,
                        updated_at: project.manifest.updated_at.clone(),
                    },
                );
            }
        } else if let Some(rest) = name.strip_prefix(entries::SNAPSHOTS_PREFIX) {
            let bytes = read_entry(archive, name)?;
            let text = String::from_utf8(bytes).map_err(|e| ProjectError::Json {
                entry: name.clone(),
                message: format!("snapshot non UTF-8: {e}"),
            })?;
            if let Some(snap) = snapshot_from_path(rest) {
                let id = snap.meta.id.clone();
                project.snapshots.push(snap.meta);
                project.snapshot_texts.insert(id, text);
            }
        }
    }
    check_consistency(&project)?;
    Ok(OpenedProject { project, profiles })
}

fn parse_json<T: serde::de::DeserializeOwned>(
    entry: &str,
    bytes: &[u8],
) -> Result<T, ProjectError> {
    serde_json::from_slice(bytes).map_err(|e| ProjectError::Json {
        entry: entry.to_string(),
        message: e.to_string(),
    })
}

fn check_version(manifest: &plotvale_model::Manifest) -> Result<(), ProjectError> {
    if manifest.schema_version > SCHEMA_VERSION {
        return Err(ProjectError::UnsupportedVersion {
            found: manifest.schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn check_consistency(project: &Project) -> Result<(), ProjectError> {
    project
        .tree
        .check_invariants()
        .map_err(|e| ProjectError::Inconsistent(e.to_string()))?;
    for snap in &project.snapshots {
        if !project.snapshot_texts.contains_key(&snap.id) {
            return Err(ProjectError::Inconsistent(format!(
                "snapshot {} senza testo",
                snap.id
            )));
        }
    }
    Ok(())
}

struct ParsedSnapshot {
    meta: plotvale_model::SnapshotMeta,
}

/// Ricostruisce i metadati di uno snapshot dal percorso `snapshot/<doc>/<file>.md`.
fn snapshot_from_path(rest: &str) -> Option<ParsedSnapshot> {
    let (doc_str, file) = rest.split_once('/')?;
    let doc_id = doc_str.parse::<NodeId>().ok()?;
    let _ = file.strip_suffix(".md")?;
    // Il formato è <id>-<etichetta>; l'id non contiene '-' (ULID Crockford).
    let (id, label) = file.split_once('-').unwrap_or((file, ""));
    Some(ParsedSnapshot {
        meta: plotvale_model::SnapshotMeta {
            id: id.to_string(),
            doc_id,
            created_at: String::new(),
            label: label.replace('_', " "),
        },
    })
}

fn read_snapshot_metas_folder(
    dir: &Path,
) -> Result<Vec<plotvale_model::SnapshotMeta>, ProjectError> {
    let mut metas = Vec::new();
    if !dir.is_dir() {
        return Ok(metas);
    }
    for doc_entry in fs::read_dir(dir)? {
        let doc_entry = doc_entry?;
        let doc_name = doc_entry.file_name().to_string_lossy().to_string();
        if doc_entry.path().is_dir() && doc_name.parse::<NodeId>().is_ok() {
            for snap_entry in fs::read_dir(doc_entry.path())? {
                let snap_entry = snap_entry?;
                let name = snap_entry.file_name().to_string_lossy().to_string();
                let rel = format!("{doc_name}/{name}");
                if let Some(parsed) = snapshot_from_path(&rel) {
                    metas.push(parsed.meta);
                }
            }
        }
    }
    metas.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(metas)
}

fn read_snapshot_texts_folder(
    dir: &Path,
) -> Result<std::collections::HashMap<String, String>, ProjectError> {
    let mut texts = std::collections::HashMap::new();
    if !dir.is_dir() {
        return Ok(texts);
    }
    for doc_entry in fs::read_dir(dir)? {
        let doc_entry = doc_entry?;
        let doc_name = doc_entry.file_name().to_string_lossy().to_string();
        if doc_entry.path().is_dir() {
            for snap_entry in fs::read_dir(doc_entry.path())? {
                let snap_entry = snap_entry?;
                let name = snap_entry.file_name().to_string_lossy().to_string();
                let rel = format!("{doc_name}/{name}");
                if let Some(parsed) = snapshot_from_path(&rel) {
                    let text = fs::read_to_string(snap_entry.path())?;
                    texts.insert(parsed.meta.id, text);
                }
            }
        }
    }
    Ok(texts)
}

// ---------------------------------------------------------------------------
// Salvataggio
// ---------------------------------------------------------------------------

/// Salva su file `.plotvale` o cartella (rilevato dal percorso).
pub fn save_any(
    project: &mut Project,
    profiles: &ProfilesFile,
    path: &Path,
    opts: &SaveOptions,
) -> Result<(), ProjectError> {
    if path.is_dir() {
        save_folder(project, profiles, path)
    } else {
        save_zip(project, profiles, path, opts)
    }
}

/// Salva un bundle `.plotvale` in modo atomico, con backup a rotazione.
pub fn save_zip(
    project: &mut Project,
    profiles: &ProfilesFile,
    path: &Path,
    opts: &SaveOptions,
) -> Result<(), ProjectError> {
    project.touch();
    let tmp = tmp_sibling(path);
    write_zip(project, profiles, &tmp)?;
    // Scarica su disco prima del rename: senza fsync un crash può perdere tutto.
    // (Su Windows il flush richiede un handle in scrittura.)
    File::options().write(true).open(&tmp)?.sync_all()?;
    if path.exists() {
        backup_existing(path, opts.backup_count)?;
    }
    fs::rename(&tmp, path)?;
    sync_parent_dir(path)?;
    Ok(())
}

/// Salva in una cartella (modalità cartella: comoda per Git e sync).
pub fn save_folder(
    project: &mut Project,
    profiles: &ProfilesFile,
    dir: &Path,
) -> Result<(), ProjectError> {
    project.touch();
    fs::create_dir_all(dir)?;
    let write = |name: &str, bytes: &[u8]| -> Result<(), ProjectError> {
        let full = dir.join(name);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full, bytes)?;
        Ok(())
    };
    write(
        entries::MANIFEST,
        &to_pretty(&project.manifest, entries::MANIFEST)?,
    )?;
    write(entries::TREE, &to_pretty(&project.tree, entries::TREE)?)?;
    write(entries::META, &to_pretty(&project.meta, entries::META)?)?;
    write(
        entries::TARGETS,
        &to_pretty(&project.targets, entries::TARGETS)?,
    )?;
    write(
        entries::CHARACTERS,
        &to_pretty(&project.sheets.characters, entries::CHARACTERS)?,
    )?;
    write(
        entries::LOCATIONS,
        &to_pretty(&project.sheets.locations, entries::LOCATIONS)?,
    )?;
    write(entries::PROFILES, &to_pretty(profiles, entries::PROFILES)?)?;
    for (id, doc) in &project.docs {
        write(
            &format!("{}{}.md", entries::TEXTS_PREFIX, id),
            doc.text.as_bytes(),
        )?;
    }
    for snap in &project.snapshots {
        if let Some(text) = project.snapshot_texts.get(&snap.id) {
            write(&snapshot_entry_name(snap), text.as_bytes())?;
        }
    }
    Ok(())
}

fn to_pretty<T: serde::Serialize>(value: &T, entry: &str) -> Result<Vec<u8>, ProjectError> {
    serde_json::to_vec_pretty(value).map_err(|e| ProjectError::Json {
        entry: entry.to_string(),
        message: e.to_string(),
    })
}

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644)
}

fn write_zip(project: &Project, profiles: &ProfilesFile, tmp: &Path) -> Result<(), ProjectError> {
    if let Some(parent) = tmp.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(tmp)?;
    let mut zip = ZipWriter::new(file);
    let opts = zip_options();
    let mut add = |name: &str, bytes: &[u8]| -> Result<(), ProjectError> {
        zip.start_file(name, opts)?;
        zip.write_all(bytes)?;
        Ok(())
    };
    add(
        entries::MANIFEST,
        &to_pretty(&project.manifest, entries::MANIFEST)?,
    )?;
    add(entries::TREE, &to_pretty(&project.tree, entries::TREE)?)?;
    add(entries::META, &to_pretty(&project.meta, entries::META)?)?;
    add(
        entries::TARGETS,
        &to_pretty(&project.targets, entries::TARGETS)?,
    )?;
    add(
        entries::CHARACTERS,
        &to_pretty(&project.sheets.characters, entries::CHARACTERS)?,
    )?;
    add(
        entries::LOCATIONS,
        &to_pretty(&project.sheets.locations, entries::LOCATIONS)?,
    )?;
    add(entries::PROFILES, &to_pretty(profiles, entries::PROFILES)?)?;
    let mut ids: Vec<NodeId> = project.docs.keys().copied().collect();
    ids.sort();
    for id in ids {
        if let Some(doc) = project.docs.get(&id) {
            add(
                &format!("{}{}.md", entries::TEXTS_PREFIX, id),
                doc.text.as_bytes(),
            )?;
        }
    }
    let mut snaps = project.snapshots.clone();
    snaps.sort_by(|a, b| a.id.cmp(&b.id));
    for snap in &snaps {
        if let Some(text) = project.snapshot_texts.get(&snap.id) {
            add(&snapshot_entry_name(snap), text.as_bytes())?;
        }
    }
    zip.finish()?;
    Ok(())
}

/// Nome della voce snapshot: `snapshot/<doc>/<id>-<etichetta>.md` (solo ASCII sicuri).
fn snapshot_entry_name(snap: &plotvale_model::SnapshotMeta) -> String {
    let safe: String = snap
        .label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe = safe.trim_matches('_');
    let file = if safe.is_empty() {
        snap.id.clone()
    } else {
        format!("{}-{safe}", snap.id)
    };
    format!("{}{}/{file}.md", entries::SNAPSHOTS_PREFIX, snap.doc_id)
}

/// File temporaneo fratello del progetto (`<nome>.plotvale.tmp`).
pub fn tmp_sibling(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

/// Cerca un recupero dopo crash: un `.tmp` più recente del progetto (o senza progetto).
/// Restituisce il percorso del file da aprire al posto dell'originale.
pub fn find_recovery(path: &Path) -> Option<PathBuf> {
    let tmp = tmp_sibling(path);
    let tmp_meta = fs::metadata(&tmp).ok()?;
    let tmp_time = tmp_meta.modified().ok()?;
    match fs::metadata(path).ok().and_then(|m| m.modified().ok()) {
        Some(main_time) if tmp_time <= main_time => None,
        _ => Some(tmp),
    }
}

/// Directory dei backup (`<nome>-backups/` accanto al progetto).
pub fn backup_dir(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "progetto".to_string());
    parent.join(format!("{stem}-backups"))
}

fn backup_existing(path: &Path, keep: usize) -> Result<(), ProjectError> {
    if keep == 0 {
        return Ok(());
    }
    let dir = backup_dir(path);
    fs::create_dir_all(&dir)?;
    // Nanosecondi dall'epoca: unico anche per salvataggi ravvicinati e ordinabile.
    let stamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "progetto.plotvale".to_string());
    let dest = dir.join(format!("{stamp}-{file_name}"));
    fs::copy(path, &dest)?;
    // Potatura: tieni solo i `keep` più recenti.
    let mut backups: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    backups.sort();
    while backups.len() > keep {
        let oldest = backups.remove(0);
        let _ = fs::remove_file(oldest);
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<(), ProjectError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Ok(dir) = File::open(parent)
    {
        let _ = dir.sync_all();
    }
    Ok(())
}
