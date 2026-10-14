//! Tauri commands: the whole UI contract over the GUI-less core.
//!
//! Every command locks [`AppState`], mutates through `model`/`project` only,
//! and marks `dirty` so autosave and the close guard stay correct. Errors are
//! `String` (never panics on user input).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use plotvale_compile::{
    CompileProfile, ProfilesFile, compile, expand_selection,
};
use plotvale_export::{ExportFormat, export_to_file, render};
use plotvale_model::{
    CharacterSheet, CollectionDef, LocationSheet, NodeId, NodeKind, Project, ProjectMeta, Targets,
    WordCounts, count_words, now_stamp,
};
use plotvale_project::{SaveOptions, find_recovery, open_any, save_any};
use plotvale_search::{Index, SearchOptions};
use tauri::State;

use crate::state::{
    AppState, ChapterWords, HitDto, ProjectDto, ProjectStatus, SaveInfo, StatsDto, parse_id,
    parse_kind, project_dto,
};

/// App version stamped into new manifests.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

type Backend = Mutex<AppState>;

fn lock<'a>(state: &'a State<'_, Backend>) -> Result<MutexGuard<'a, AppState>, String> {
    state.lock().map_err(|_| "backend state is poisoned".to_string())
}

fn open_project_of(state: &mut AppState) -> Result<&mut Project, String> {
    state
        .project
        .as_mut()
        .ok_or_else(|| "no project open".to_string())
}

// ---------------------------------------------------------------------------
// Project lifecycle
// ---------------------------------------------------------------------------

/// Creates a project (same starter content as the CLI/`Project::new`).
#[tauri::command]
pub fn new_project(state: State<'_, Backend>, title: String, author: String) -> Result<ProjectDto, String> {
    let mut backend = lock(&state)?;
    let title = title.trim();
    let title = if title.is_empty() { "Untitled" } else { title };
    let project = Project::new(title, author.trim(), APP_VERSION);
    let total = project.total_words();
    backend.profiles = ProfilesFile::default();
    backend.last_total_words = total;
    backend.project = Some(project);
    backend.path = None;
    backend.folder_mode = false;
    backend.dirty = true;
    project_dto(&backend)
}

/// Opens a `.plotvale` bundle or folder (crash recovery included).
#[tauri::command]
pub fn open_project(state: State<'_, Backend>, path: String) -> Result<ProjectDto, String> {
    let mut backend = lock(&state)?;
    let requested = PathBuf::from(&path);
    // Crash recovery: a newer `.tmp` sibling wins over the original.
    let real = find_recovery(&requested).unwrap_or(requested.clone());
    let opened = open_any(&real).map_err(|e| e.to_string())?;
    backend.last_total_words = opened.project.total_words();
    backend.profiles = opened.profiles;
    backend.project = Some(opened.project);
    backend.folder_mode = requested.is_dir();
    backend.path = Some(requested);
    backend.dirty = false;
    project_dto(&backend)
}

/// Saves to the current location (`None` path = never saved: use save-as).
#[tauri::command]
pub fn save_project(state: State<'_, Backend>) -> Result<SaveInfo, String> {
    let mut backend = lock(&state)?;
    let path = backend
        .path
        .clone()
        .ok_or_else(|| "never saved: use Save as".to_string())?;
    save_locked(&mut backend, &path)
}

/// Saves to a new location (bundle or folder).
#[tauri::command]
pub fn save_project_as(
    state: State<'_, Backend>,
    path: String,
    folder_mode: bool,
) -> Result<SaveInfo, String> {
    let mut backend = lock(&state)?;
    let target = PathBuf::from(&path);
    backend.folder_mode = folder_mode;
    let info = save_locked(&mut backend, &target)?;
    backend.path = Some(target);
    Ok(info)
}

fn save_locked(backend: &mut AppState, path: &Path) -> Result<SaveInfo, String> {
    let (project, profiles) = match (&mut backend.project, &backend.profiles) {
        (Some(project), profiles) => (project, profiles),
        _ => return Err("no project open".to_string()),
    };
    save_any(project, profiles, path, &SaveOptions::default()).map_err(|e| e.to_string())?;
    backend.dirty = false;
    // Daily-goal accounting (same rule as the classic UI: deltas after save).
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if project.targets.words_today_date.as_deref() != Some(&today) {
        project.targets.words_today = 0;
        project.targets.words_today_date = Some(today);
    }
    let total = project.total_words();
    if total > backend.last_total_words {
        project.targets.words_today += (total - backend.last_total_words) as u64;
    }
    backend.last_total_words = total;
    Ok(SaveInfo {
        total_words: total,
        words_today: project.targets.words_today,
    })
}

/// Closes the project (the frontend guards unsaved changes first).
#[tauri::command]
pub fn close_project(state: State<'_, Backend>) -> Result<(), String> {
    let mut backend = lock(&state)?;
    backend.project = None;
    backend.path = None;
    backend.dirty = false;
    Ok(())
}

/// Save-location and dirty state for the status bar / close guard.
#[tauri::command]
pub fn project_status(state: State<'_, Backend>) -> Result<ProjectStatus, String> {
    let backend = lock(&state)?;
    Ok(ProjectStatus {
        has_project: backend.project.is_some(),
        path: backend.path.as_ref().map(|p| p.to_string_lossy().to_string()),
        folder_mode: backend.folder_mode,
        dirty: backend.dirty,
        title: backend
            .project
            .as_ref()
            .map(|p| p.manifest.title.clone())
            .unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// Tree (binder)
// ---------------------------------------------------------------------------

/// Creates a child node (linked sheet rows included, like the classic UI).
#[tauri::command]
pub fn create_node(
    state: State<'_, Backend>,
    parent: String,
    kind: String,
    title: String,
) -> Result<crate::state::NodeDto, String> {
    use crate::state::NodeDto;
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let parent_id = parse_id(&parent)?;
    let kind = parse_kind(&kind)?;
    let title = title.trim();
    let title = if title.is_empty() { "Untitled" } else { title };
    let id = project.tree.create(parent_id, kind, title).map_err(|e| e.to_string())?;
    match kind {
        NodeKind::CharacterSheet => project.sheets.characters.push(CharacterSheet {
            node_id: id,
            name: title.to_string(),
            role: String::new(),
            description: String::new(),
            goal: String::new(),
            conflict: String::new(),
            questionnaire: Default::default(),
        }),
        NodeKind::LocationSheet => project.sheets.locations.push(LocationSheet {
            node_id: id,
            name: title.to_string(),
            description: String::new(),
            sensory: String::new(),
            notes: String::new(),
        }),
        _ => {}
    }
    project.touch();
    let node = project.tree.get(id).map_err(|e| e.to_string())?.clone();
    backend.dirty = true;
    Ok(NodeDto {
        id: node.id.to_string(),
        parent: node.parent.map(|p| p.to_string()),
        children: node.children.iter().map(|c| c.to_string()).collect(),
        kind: node.kind,
        title: node.title.clone(),
        meta: node.meta.clone(),
    })
}

/// Renames a node.
#[tauri::command]
pub fn rename_node(state: State<'_, Backend>, id: String, title: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    project.tree.rename(id, title.trim()).map_err(|e| e.to_string())?;
    // Sheets mirror the node title.
    if let Ok(node) = project.tree.get(id) {
        let title = node.title.clone();
        match node.kind {
            NodeKind::CharacterSheet => {
                if let Some(sheet) = project.sheets.characters.iter_mut().find(|s| s.node_id == id) {
                    sheet.name = title;
                }
            }
            NodeKind::LocationSheet => {
                if let Some(sheet) = project.sheets.locations.iter_mut().find(|s| s.node_id == id) {
                    sheet.name = title;
                }
            }
            _ => {}
        }
    }
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Moves a node under a new parent at `pos` (tree order = export order).
#[tauri::command]
pub fn move_node(
    state: State<'_, Backend>,
    id: String,
    new_parent: String,
    pos: usize,
) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let (id, new_parent) = (parse_id(&id)?, parse_id(&new_parent)?);
    project.tree.move_node(id, new_parent, pos).map_err(|e| e.to_string())?;
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Moves a node one slot among siblings (`up = true` = upwards).
#[tauri::command]
pub fn reorder_node(state: State<'_, Backend>, id: String, up: bool) -> Result<bool, String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    let moved = project.tree.reorder(id, up).map_err(|e| e.to_string())?;
    if moved {
        project.touch();
        backend.dirty = true;
    }
    Ok(moved)
}

/// Indents under the previous sibling / outdents to the parent level.
#[tauri::command]
pub fn indent_node(state: State<'_, Backend>, id: String) -> Result<bool, String> {
    shift_level(&state, &id, true)
}

/// Outdents to the parent level.
#[tauri::command]
pub fn outdent_node(state: State<'_, Backend>, id: String) -> Result<bool, String> {
    shift_level(&state, &id, false)
}

fn shift_level(state: &State<'_, Backend>, raw: &str, indent: bool) -> Result<bool, String> {
    let mut backend = lock(state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(raw)?;
    let moved = if indent {
        project.tree.indent(id)
    } else {
        project.tree.outdent(id)
    }
    .map_err(|e| e.to_string())?;
    if moved {
        project.touch();
        backend.dirty = true;
    }
    Ok(moved)
}

/// Moves a node (and children) to trash.
#[tauri::command]
pub fn trash_node(state: State<'_, Backend>, id: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    project.tree.move_to_trash(id).map_err(|e| e.to_string())?;
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Restores a trashed item to its previous parent.
#[tauri::command]
pub fn restore_node(state: State<'_, Backend>, id: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    project.tree.restore(id).map_err(|e| e.to_string())?;
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Empties the trash (texts, sheets, snapshots of removed ids included).
#[tauri::command]
pub fn empty_trash(state: State<'_, Backend>) -> Result<Vec<String>, String> {
    let mut backend = lock(&state)?;
    let removed = {
        let project = open_project_of(&mut backend)?;
        let removed = project.tree.empty_trash();
        let gone: std::collections::HashSet<NodeId> = removed.iter().copied().collect();
        for id in &removed {
            project.docs.remove(id);
        }
        project.sheets.characters.retain(|s| !gone.contains(&s.node_id));
        project.sheets.locations.retain(|s| !gone.contains(&s.node_id));
        let mut snap_gone = Vec::new();
        project.snapshots.retain(|s| {
            let keep = !gone.contains(&s.doc_id);
            if !keep {
                snap_gone.push(s.id.clone());
            }
            keep
        });
        for id in snap_gone {
            project.snapshot_texts.remove(&id);
        }
        project.touch();
        removed
    };
    backend.dirty = true;
    Ok(removed.iter().map(|id| id.to_string()).collect())
}

/// Permanently deletes a node and its subtree.
#[tauri::command]
pub fn delete_node(state: State<'_, Backend>, id: String) -> Result<Vec<String>, String> {
    let mut backend = lock(&state)?;
    let removed = {
        let project = open_project_of(&mut backend)?;
        let id = parse_id(&id)?;
        let removed = project.tree.delete_permanently(id).map_err(|e| e.to_string())?;
        let gone: std::collections::HashSet<NodeId> = removed.iter().copied().collect();
        for gone_id in &removed {
            project.docs.remove(gone_id);
        }
        project.sheets.characters.retain(|s| !gone.contains(&s.node_id));
        project.sheets.locations.retain(|s| !gone.contains(&s.node_id));
        let mut snap_gone = Vec::new();
        project.snapshots.retain(|s| {
            let keep = !gone.contains(&s.doc_id);
            if !keep {
                snap_gone.push(s.id.clone());
            }
            keep
        });
        for snap_id in snap_gone {
            project.snapshot_texts.remove(&snap_id);
        }
        project.touch();
        removed
    };
    backend.dirty = true;
    Ok(removed.iter().map(|id| id.to_string()).collect())
}

/// Whole tree, JSON-ready (reconciles the frontend mirror after mutations).
#[tauri::command]
pub fn get_tree(state: State<'_, Backend>) -> Result<Vec<crate::state::NodeDto>, String> {
    use crate::state::NodeDto;
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let mut nodes: Vec<NodeDto> = project
        .tree
        .ids()
        .filter_map(|id| project.tree.get(id).ok())
        .map(|node| NodeDto {
            id: node.id.to_string(),
            parent: node.parent.map(|p| p.to_string()),
            children: node.children.iter().map(|c| c.to_string()).collect(),
            kind: node.kind,
            title: node.title.clone(),
            meta: node.meta.clone(),
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(nodes)
}

/// One node, JSON-ready (refreshes the frontend mirror after mutations).
#[tauri::command]
pub fn get_node(state: State<'_, Backend>, id: String) -> Result<crate::state::NodeDto, String> {
    use crate::state::NodeDto;
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let id = parse_id(&id)?;
    let node = project.tree.get(id).map_err(|e| e.to_string())?;
    Ok(NodeDto {
        id: node.id.to_string(),
        parent: node.parent.map(|p| p.to_string()),
        children: node.children.iter().map(|c| c.to_string()).collect(),
        kind: node.kind,
        title: node.title.clone(),
        meta: node.meta.clone(),
    })
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

/// Reads a document's text (empty string if the node has no text).
#[tauri::command]
pub fn get_doc_text(state: State<'_, Backend>, id: String) -> Result<String, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let id = parse_id(&id)?;
    Ok(project.doc_text(id).to_string())
}

/// Writes a document's text (marks dirty; the frontend debounces keystrokes).
#[tauri::command]
pub fn set_doc_text(state: State<'_, Backend>, id: String, text: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    project.set_doc_text(id, &text);
    backend.dirty = true;
    Ok(())
}

// ---------------------------------------------------------------------------
// Metadata, sheets, targets
// ---------------------------------------------------------------------------

/// Replaces a node's metadata (inspector edits).
#[tauri::command]
pub fn update_node_meta(
    state: State<'_, Backend>,
    id: String,
    meta: plotvale_model::NodeMeta,
) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let id = parse_id(&id)?;
    // Trash bookkeeping stays backend-owned: restores must keep working.
    let trashed_from = project
        .tree
        .get(id)
        .map_err(|e| e.to_string())?
        .meta
        .trashed_from;
    let node = project.tree.get_mut(id).map_err(|e| e.to_string())?;
    node.meta = meta;
    node.meta.trashed_from = trashed_from;
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Project-global metadata (keywords, collections, custom fields, notes).
#[tauri::command]
pub fn get_project_meta(state: State<'_, Backend>) -> Result<ProjectMeta, String> {
    let backend = lock(&state)?;
    backend
        .project
        .as_ref()
        .map(|p| p.meta.clone())
        .ok_or_else(|| "no project open".to_string())
}

/// Replaces the project-global metadata.
#[tauri::command]
pub fn set_project_meta(state: State<'_, Backend>, meta: ProjectMeta) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    project.meta = meta;
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Inserts or replaces a character sheet by node id.
#[tauri::command]
pub fn upsert_character(
    state: State<'_, Backend>,
    sheet: plotvale_model::CharacterSheet,
) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    match project.sheets.characters.iter_mut().find(|s| s.node_id == sheet.node_id) {
        Some(existing) => *existing = sheet,
        None => project.sheets.characters.push(sheet),
    }
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Inserts or replaces a location sheet by node id.
#[tauri::command]
pub fn upsert_location(
    state: State<'_, Backend>,
    sheet: plotvale_model::LocationSheet,
) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    match project.sheets.locations.iter_mut().find(|s| s.node_id == sheet.node_id) {
        Some(existing) => *existing = sheet,
        None => project.sheets.locations.push(sheet),
    }
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Writing goals.
#[tauri::command]
pub fn get_targets(state: State<'_, Backend>) -> Result<Targets, String> {
    let backend = lock(&state)?;
    backend
        .project
        .as_ref()
        .map(|p| p.targets.clone())
        .ok_or_else(|| "no project open".to_string())
}

/// Replaces the writing goals.
#[tauri::command]
pub fn set_targets(state: State<'_, Backend>, targets: Targets) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    project.targets = targets;
    project.touch();
    backend.dirty = true;
    Ok(())
}

// ---------------------------------------------------------------------------
// Snapshots
// ---------------------------------------------------------------------------

/// Snapshot list of one document (newest last, like the classic UI).
#[tauri::command]
pub fn list_snapshots(
    state: State<'_, Backend>,
    doc_id: String,
) -> Result<Vec<plotvale_model::SnapshotMeta>, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let doc_id = parse_id(&doc_id)?;
    Ok(project
        .snapshots
        .iter()
        .filter(|s| s.doc_id == doc_id)
        .cloned()
        .collect())
}

/// Saves the current text as a snapshot (empty label = timestamp).
#[tauri::command]
pub fn create_snapshot(
    state: State<'_, Backend>,
    doc_id: String,
    label: String,
) -> Result<plotvale_model::SnapshotMeta, String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let doc_id = parse_id(&doc_id)?;
    let label = label.trim();
    let label = if label.is_empty() {
        now_stamp()
    } else {
        label.to_string()
    };
    let id = ulid::Ulid::new().to_string();
    let meta = plotvale_model::SnapshotMeta {
        id: id.clone(),
        doc_id,
        created_at: now_stamp(),
        label,
    };
    let text = project.doc_text(doc_id).to_string();
    project.snapshots.push(meta.clone());
    project.snapshot_texts.insert(id, text);
    project.touch();
    backend.dirty = true;
    Ok(meta)
}

/// Reads a snapshot's text (for diff and preview).
#[tauri::command]
pub fn get_snapshot_text(state: State<'_, Backend>, snap_id: String) -> Result<String, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    project
        .snapshot_texts
        .get(&snap_id)
        .cloned()
        .ok_or_else(|| format!("snapshot not found: {snap_id}"))
}

/// Restores a snapshot's text into the document.
#[tauri::command]
pub fn restore_snapshot(
    state: State<'_, Backend>,
    doc_id: String,
    snap_id: String,
) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let doc_id = parse_id(&doc_id)?;
    let text = project
        .snapshot_texts
        .get(&snap_id)
        .cloned()
        .ok_or_else(|| format!("snapshot not found: {snap_id}"))?;
    project.set_doc_text(doc_id, &text);
    backend.dirty = true;
    Ok(())
}

/// Deletes a snapshot and its text.
#[tauri::command]
pub fn delete_snapshot(state: State<'_, Backend>, snap_id: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    project.snapshots.retain(|s| s.id != snap_id);
    project.snapshot_texts.remove(&snap_id);
    project.touch();
    backend.dirty = true;
    Ok(())
}

// ---------------------------------------------------------------------------
// Search (index rebuilt per call: always current, never stale)
// ---------------------------------------------------------------------------

/// Project-wide search (plain or regex, trash excluded, max 5000 hits).
#[tauri::command]
pub fn search_project(
    state: State<'_, Backend>,
    pattern: String,
    regex: bool,
    case_sensitive: bool,
) -> Result<Vec<HitDto>, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let index = Index::build(project);
    let hits = index
        .search(
            &pattern,
            SearchOptions {
                case_sensitive,
                regex,
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(hits
        .into_iter()
        .map(|hit| {
            let title = project
                .tree
                .get(hit.doc_id)
                .map(|n| n.title.clone())
                .unwrap_or_default();
            HitDto {
                doc_id: hit.doc_id.to_string(),
                title,
                line_no: hit.line_no,
                col: hit.col,
                excerpt: hit.excerpt,
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Statistics (on demand only: safe on 150k-word scenes)
// ---------------------------------------------------------------------------

/// Full statistics for the dialog (words, chars, paragraphs, per chapter).
#[tauri::command]
pub fn project_stats(state: State<'_, Backend>) -> Result<StatsDto, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let mut total = WordCounts::default();
    let mut docs_with_text = 0usize;
    for id in project.tree.ids() {
        if project.tree.is_in_trash(id) {
            continue;
        }
        let counts = count_words(project.doc_text(id));
        if counts.characters_no_spaces > 0 {
            docs_with_text += 1;
        }
        total = total + counts;
    }
    let mut per_chapter = Vec::new();
    for child in project.tree.children_of(project.tree.draft_root).unwrap_or_default() {
        let title = project
            .tree
            .get(child)
            .map(|n| n.title.clone())
            .unwrap_or_default();
        per_chapter.push(ChapterWords {
            id: child.to_string(),
            title,
            words: project.subtree_words(child),
        });
    }
    Ok(StatsDto {
        words: total.words,
        characters_no_spaces: total.characters_no_spaces,
        paragraphs: total.paragraphs,
        docs_with_text,
        per_chapter,
        total_counts: total,
    })
}

/// Words of one document.
#[tauri::command]
pub fn doc_words(state: State<'_, Backend>, id: String) -> Result<WordCounts, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let id = parse_id(&id)?;
    Ok(count_words(project.doc_text(id)))
}

/// Total words outside trash.
#[tauri::command]
pub fn total_words(state: State<'_, Backend>) -> Result<usize, String> {
    let backend = lock(&state)?;
    backend
        .project
        .as_ref()
        .map(|p| p.total_words())
        .ok_or_else(|| "no project open".to_string())
}

/// Words of a subtree (chapter, folder…), trash excluded.
#[tauri::command]
pub fn subtree_words(state: State<'_, Backend>, id: String) -> Result<usize, String> {
    let backend = lock(&state)?;
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let id = parse_id(&id)?;
    Ok(project.subtree_words(id))
}

// ---------------------------------------------------------------------------
// Compile and export (single composition engine, like every other UI)
// ----------------------------------------------------------------}

/// Available export profiles plus the active name.
#[tauri::command]
pub fn list_profiles(state: State<'_, Backend>) -> Result<ProfilesFile, String> {
    let backend = lock(&state)?;
    Ok(backend.profiles.clone())
}

/// Switches the active export profile.
#[tauri::command]
pub fn set_active_profile(state: State<'_, Backend>, name: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    if backend.project.is_none() {
        return Err("no project open".to_string());
    }
    if !backend.profiles.profiles.iter().any(|p| p.name == name) {
        return Err(format!("unknown profile: {name}"));
    }
    backend.profiles.active = name;
    backend.dirty = true;
    Ok(())
}

/// Inserts or replaces an export profile.
#[tauri::command]
pub fn update_profile(state: State<'_, Backend>, profile: CompileProfile) -> Result<(), String> {
    let mut backend = lock(&state)?;
    if backend.project.is_none() {
        return Err("no project open".to_string());
    }
    match backend
        .profiles
        .profiles
        .iter_mut()
        .find(|p| p.name == profile.name)
    {
        Some(existing) => *existing = profile,
        None => backend.profiles.profiles.push(profile),
    }
    backend.dirty = true;
    Ok(())
}

fn compile_locked(
    backend: &AppState,
    profile_name: Option<String>,
    keep_notes: Option<bool>,
    selection: Vec<String>,
) -> Result<(plotvale_compile::Compiled, CompileProfile), String> {
    let project = backend.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let mut profile = match profile_name {
        Some(name) => backend
            .profiles
            .profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| format!("unknown profile: {name}"))?,
        None => backend.profiles.active_profile(),
    };
    if let Some(keep) = keep_notes {
        profile.keep_notes = keep;
    }
    let mut ids = Vec::new();
    for raw in selection {
        ids.push(parse_id(&raw)?);
    }
    let ids = expand_selection(project, &ids);
    let compiled = compile(project, &ids, &profile).map_err(|e| e.to_string())?;
    Ok((compiled, profile))
}

/// Compiles the manuscript and renders HTML for the preview pane.
#[tauri::command]
pub fn compile_preview(
    state: State<'_, Backend>,
    profile_name: Option<String>,
    keep_notes: Option<bool>,
    selection: Vec<String>,
) -> Result<String, String> {
    let backend = lock(&state)?;
    let (compiled, _) = compile_locked(&backend, profile_name, keep_notes, selection)?;
    let bytes = render(&compiled, ExportFormat::Html).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Compiles and writes the manuscript file (`md`, `txt`, `html`, `docx`, `epub`).
#[tauri::command]
pub fn export_manuscript(
    state: State<'_, Backend>,
    format: String,
    path: String,
    profile_name: Option<String>,
    keep_notes: Option<bool>,
    selection: Vec<String>,
) -> Result<(), String> {
    let backend = lock(&state)?;
    let format: ExportFormat = format.parse().map_err(|e: String| e)?;
    let (compiled, _) = compile_locked(&backend, profile_name, keep_notes, selection)?;
    export_to_file(&compiled, format, &PathBuf::from(&path)).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Saved searches (collections live in `metadati.json`)
// ---------------------------------------------------------------------------

/// Saved searches of the project.
#[tauri::command]
pub fn list_collections(state: State<'_, Backend>) -> Result<Vec<CollectionDef>, String> {
    let backend = lock(&state)?;
    backend
        .project
        .as_ref()
        .map(|p| p.meta.collections.clone())
        .ok_or_else(|| "no project open".to_string())
}

/// Adds a saved search.
#[tauri::command]
pub fn add_collection(state: State<'_, Backend>, collection: CollectionDef) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    if project.meta.collections.iter().any(|c| c.name == collection.name) {
        return Err(format!("collection already exists: {}", collection.name));
    }
    project.meta.collections.push(collection);
    project.touch();
    backend.dirty = true;
    Ok(())
}

/// Removes a saved search by name.
#[tauri::command]
pub fn remove_collection(state: State<'_, Backend>, name: String) -> Result<(), String> {
    let mut backend = lock(&state)?;
    let project = open_project_of(&mut backend)?;
    let before = project.meta.collections.len();
    project.meta.collections.retain(|c| c.name != name);
    if project.meta.collections.len() == before {
        return Err(format!("collection not found: {name}"));
    }
    project.touch();
    backend.dirty = true;
    Ok(())
}
