//! Shared backend state and JSON DTOs.
//!
//! The Tauri backend owns the [`Project`] (GUI-less core crates); the React
//! frontend holds a mirror for rendering and sends deltas back as commands.
//! Node ids travel as ULID strings (`NodeId: Display + FromStr`).

use std::collections::HashMap;
use std::path::PathBuf;

use plotvale_compile::ProfilesFile;
use plotvale_model::{
    CharacterSheet, LocationSheet, Manifest, Node, NodeId, NodeKind, NodeMeta, Project, ProjectMeta,
    SnapshotMeta, Targets, WordCounts,
};
use serde::Serialize;

/// Mutable backend state behind a `Mutex`.
#[derive(Default)]
pub struct AppState {
    /// Open project (`None` = welcome screen).
    pub project: Option<Project>,
    /// Export profiles (`composizione/profili.json`).
    pub profiles: ProfilesFile,
    /// Save location (`None` = never saved).
    pub path: Option<PathBuf>,
    /// Folder mode vs `.plotvale` bundle.
    pub folder_mode: bool,
    /// Unsaved changes (set by any mutation, cleared by save).
    pub dirty: bool,
    /// Total words at the last save (for daily-goal accounting).
    pub last_total_words: usize,
}

/// One tree item, JSON-ready (ids as strings).
#[derive(Clone, Debug, Serialize)]
pub struct NodeDto {
    pub id: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub kind: NodeKind,
    pub title: String,
    pub meta: NodeMeta,
}

impl NodeDto {
    fn of(node: &Node) -> Self {
        Self {
            id: node.id.to_string(),
            parent: node.parent.map(|p| p.to_string()),
            children: node.children.iter().map(|c| c.to_string()).collect(),
            kind: node.kind,
            title: node.title.clone(),
            meta: node.meta.clone(),
        }
    }
}

/// Whole project snapshot sent to the frontend on open/new.
#[derive(Clone, Debug, Serialize)]
pub struct ProjectDto {
    pub manifest: Manifest,
    pub nodes: Vec<NodeDto>,
    pub draft_root: String,
    pub research_root: String,
    pub trash: String,
    /// Texts by node id (string keys).
    pub docs: HashMap<String, String>,
    pub meta: ProjectMeta,
    pub characters: Vec<CharacterSheet>,
    pub locations: Vec<LocationSheet>,
    pub targets: Targets,
    pub snapshots: Vec<SnapshotMeta>,
    pub profiles: ProfilesFile,
}

/// Project already open at startup (command-line argument, file association).
#[derive(Clone, Debug, Serialize)]
pub struct InitialOpen {
    pub project: ProjectDto,
    pub path: String,
    pub folder_mode: bool,
}

/// Save-location and dirty state for the status bar / close guard.
#[derive(Clone, Debug, Serialize)]
pub struct ProjectStatus {
    pub has_project: bool,
    pub path: Option<String>,
    pub folder_mode: bool,
    pub dirty: bool,
    pub title: String,
}

/// Save outcome (feeds the daily-goal display).
#[derive(Clone, Debug, Serialize)]
pub struct SaveInfo {
    pub total_words: usize,
    pub words_today: u64,
}

/// One search hit, JSON-ready.
#[derive(Clone, Debug, Serialize)]
pub struct HitDto {
    pub doc_id: String,
    pub title: String,
    pub line_no: usize,
    pub col: usize,
    pub excerpt: String,
}

/// Statistics dialog payload (computed on demand, never per frame).
#[derive(Clone, Debug, Serialize)]
pub struct ChapterWords {
    pub id: String,
    pub title: String,
    pub words: usize,
}

/// Statistics dialog payload (computed on demand, never per frame).
#[derive(Clone, Debug, Serialize)]
pub struct StatsDto {
    pub words: usize,
    pub characters_no_spaces: usize,
    pub paragraphs: usize,
    pub docs_with_text: usize,
    pub per_chapter: Vec<ChapterWords>,
    pub total_counts: WordCounts,
}

/// Builds state, opening the path given on the command line when there is one.
///
/// Windows file associations and "Open with" hand the file this way; the
/// frontend asks for it once (`initial_project`) and shows the editor.
pub fn from_env() -> (AppState, Option<InitialOpen>) {
    let mut state = AppState::default();
    let Some(arg) = std::env::args_os().nth(1).map(PathBuf::from) else {
        return (state, None);
    };
    if !arg.exists() {
        return (state, None);
    }
    let real = plotvale_project::find_recovery(&arg).unwrap_or_else(|| arg.clone());
    let Ok(opened) = plotvale_project::open_any(&real) else {
        return (state, None);
    };
    state.last_total_words = opened.project.total_words();
    state.profiles = opened.profiles;
    state.project = Some(opened.project);
    state.folder_mode = arg.is_dir();
    state.path = Some(arg.clone());
    let initial = project_dto(&state).ok().map(|project| InitialOpen {
        project,
        path: arg.to_string_lossy().to_string(),
        folder_mode: state.folder_mode,
    });
    (state, initial)
}

/// Parses a ULID string from the frontend (never panics on user input).
pub fn parse_id(raw: &str) -> Result<NodeId, String> {
    raw.parse::<NodeId>()
        .map_err(|_| format!("invalid item id: {raw}"))
}

/// Parses a node kind from the frontend (`scene`, `folder`, …).
pub fn parse_kind(raw: &str) -> Result<NodeKind, String> {
    match raw.to_lowercase().as_str() {
        "scene" => Ok(NodeKind::Scene),
        "folder" => Ok(NodeKind::Folder),
        "note" => Ok(NodeKind::Note),
        "research_item" | "researchitem" | "research" => Ok(NodeKind::ResearchItem),
        "character" | "character_sheet" | "charactersheet" => Ok(NodeKind::CharacterSheet),
        "location" | "location_sheet" | "locationsheet" => Ok(NodeKind::LocationSheet),
        other => Err(format!("unknown kind: {other}")),
    }
}

/// Builds the full DTO from backend state (project must be open).
pub fn project_dto(state: &AppState) -> Result<ProjectDto, String> {
    let project = state.project.as_ref().ok_or_else(|| "no project open".to_string())?;
    let mut nodes: Vec<NodeDto> = project
        .tree
        .ids()
        .filter_map(|id| project.tree.get(id).ok())
        .map(NodeDto::of)
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let docs: HashMap<String, String> = project
        .docs
        .iter()
        .map(|(id, doc)| (id.to_string(), doc.text.clone()))
        .collect();
    Ok(ProjectDto {
        manifest: project.manifest.clone(),
        nodes,
        draft_root: project.tree.draft_root.to_string(),
        research_root: project.tree.research_root.to_string(),
        trash: project.tree.trash.to_string(),
        docs,
        meta: project.meta.clone(),
        characters: project.sheets.characters.clone(),
        locations: project.sheets.locations.clone(),
        targets: project.targets.clone(),
        snapshots: project.snapshots.clone(),
        profiles: state.profiles.clone(),
    })
}
