//! Test del formato: round-trip identico, ZIP troncato, backup, recupero crash.
//!
//! La promessa di PlotVale è non perdere mai il testo: questi test la verificano.

use std::fs;
use std::path::PathBuf;
use plotvale_compile::ProfilesFile;
use plotvale_model::{NodeKind, Project};
use plotvale_project::{
    SaveOptions, backup_dir, find_recovery, open_any, open_zip, save_any, save_folder, save_zip,
    tmp_sibling,
};

fn sample() -> (Project, ProfilesFile, String) {
    let mut project = Project::new("Romanzo di prova", "Autrice", "0.1.0-test");
    let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
    let s2 = project
        .tree
        .create(cap, NodeKind::Scene, "Scena 2")
        .unwrap();
    project.set_doc_text(s2, "Seconda scena con **grassetto** e [[nota: segreta]].");
    // Uno snapshot sulla prima scena.
    let s1 = project.tree.children_of(cap).unwrap()[0];
    let text = project.doc_text(s1).to_string();
    let snap_id = plotvale_model::NodeId::new().to_string();
    project.snapshots.push(plotvale_model::SnapshotMeta {
        id: snap_id.to_string(),
        doc_id: s1,
        created_at: "2026-10-13T00:00:00Z".to_string(),
        label: "prima stesura".to_string(),
    });
    project.snapshot_texts.insert(snap_id.to_string(), text);
    // Una scheda personaggio collegata all'albero.
    let sheet = project
        .tree
        .create(project.tree.draft_root, NodeKind::CharacterSheet, "Elena")
        .unwrap();
    project
        .sheets
        .characters
        .push(plotvale_model::CharacterSheet {
            node_id: sheet,
            name: "Elena".to_string(),
            role: "Protagonista".to_string(),
            description: "Ostinata.".to_string(),
            goal: "Trovare casa.".to_string(),
            conflict: "Il passato.".to_string(),
            questionnaire: Default::default(),
        });
    (project, ProfilesFile::default(), snap_id)
}

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plotvale-test-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir.join("romanzo.plotvale")
}

#[test]
fn zip_roundtrip_is_identical() {
    let (mut project, profiles, snap_id) = sample();
    let path = temp_path("roundtrip");
    save_zip(&mut project, &profiles, &path, &SaveOptions::default()).unwrap();
    let opened = open_zip(&path).unwrap();
    assert_eq!(opened.project.manifest.title, "Romanzo di prova");
    assert_eq!(opened.project.total_words(), project.total_words());
    assert_eq!(opened.project.snapshots.len(), 1);
    assert_eq!(opened.project.sheets.characters.len(), 1);
    assert_eq!(opened.profiles, profiles);
    // Il testo delle snapshot sopravvive al viaggio.
    let s1 = opened
        .project
        .tree
        .children_of(opened.project.tree.draft_root)
        .unwrap()[0];
    let s1 = opened.project.tree.children_of(s1).unwrap()[0];
    assert_eq!(
        opened.project.snapshot_texts[snap_id.as_str()],
        opened.project.doc_text(s1)
    );
    // Secondo giro: stabile.
    let path2 = path.with_extension("plotvale2");
    let mut again = opened.project.clone();
    save_zip(
        &mut again,
        &opened.profiles,
        &path2,
        &SaveOptions::default(),
    )
    .unwrap();
    let reopened = open_zip(&path2).unwrap();
    assert_eq!(reopened.project.total_words(), project.total_words());
}

#[test]
fn truncated_zip_is_an_error_not_a_panic() {
    let (mut project, profiles, _snap_id) = sample();
    let path = temp_path("truncated");
    save_zip(&mut project, &profiles, &path, &SaveOptions::default()).unwrap();
    // Tronca il file a metà: deve dare errore, mai panic.
    let bytes = fs::read(&path).unwrap();
    fs::write(&path, &bytes[..bytes.len() / 2]).unwrap();
    assert!(open_zip(&path).is_err());
}

#[test]
fn missing_entry_is_an_error() {
    let path = temp_path("missing");
    // ZIP valido ma senza le voci obbligatorie.
    let file = fs::File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("testi/x.md", zip::write::SimpleFileOptions::default())
        .unwrap();
    use std::io::Write as _;
    zip.write_all(b"ciao").unwrap();
    zip.finish().unwrap();
    assert!(open_zip(&path).is_err());
}

#[test]
fn future_schema_is_rejected() {
    let (mut project, profiles, _snap_id) = sample();
    project.manifest.schema_version = 999;
    let path = temp_path("future");
    save_zip(&mut project, &profiles, &path, &SaveOptions::default()).unwrap();
    let err = open_zip(&path).unwrap_err().to_string();
    assert!(err.contains("recente"), "errore inatteso: {err}");
}

#[test]
fn backups_rotate_and_recovery_detects_tmp() {
    let (mut project, profiles, _snap_id) = sample();
    let path = temp_path("backups");
    let opts = SaveOptions { backup_count: 3 };
    for _ in 0..5 {
        save_zip(&mut project, &profiles, &path, &opts).unwrap();
    }
    let backups: Vec<_> = fs::read_dir(backup_dir(&path))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(backups.len(), 3, "rotazione dei backup");
    // Simula un crash: .tmp più recente del progetto.
    let tmp = tmp_sibling(&path);
    fs::copy(&path, &tmp).unwrap();
    // Tocca il tmp per renderlo più recente.
    let future = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    let _ = filetime_set(&tmp, future);
    assert_eq!(find_recovery(&path), Some(tmp));
}

fn filetime_set(path: &std::path::Path, t: std::time::SystemTime) -> std::io::Result<()> {
    fs::File::options().write(true).open(path)?.set_modified(t)
}

#[test]
fn folder_mode_roundtrip() {
    let (mut project, profiles, _snap_id) = sample();
    let dir = std::env::temp_dir().join(format!("plotvale-folder-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    save_folder(&mut project, &profiles, &dir).unwrap();
    let opened = open_any(&dir).unwrap();
    assert_eq!(opened.project.total_words(), project.total_words());
    assert_eq!(opened.project.snapshots.len(), 1);
    // E si può risalvare sopra senza errori.
    let mut again = opened.project.clone();
    save_any(&mut again, &opened.profiles, &dir, &SaveOptions::default()).unwrap();
}
