//! Project document: manifest, texts, sheets, goals, snapshots.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{NodeId, Tree, WordCounts, count_words};

/// Current format version (see `docs/format-spec.md`).
pub const SCHEMA_VERSION: u32 = 1;

/// RFC 3339 UTC timestamp, for manifest, snapshots, and documents.
pub fn now_stamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Project header (`plotvale.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Format schema version.
    pub schema_version: u32,
    /// Version of the app that saved last.
    pub app_version: String,
    /// Title of the work.
    pub title: String,
    /// Author.
    #[serde(default)]
    pub author: String,
    /// Primary language (`it`, `en`, …).
    #[serde(default = "default_language")]
    pub language: String,
    /// Project creation.
    pub created_at: String,
    /// Last save.
    pub updated_at: String,
}

fn default_language() -> String {
    "it".to_string()
}

/// A document's text, with last-modified date.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doc {
    /// Document Markdown text.
    #[serde(default)]
    pub text: String,
    /// Last modified (RFC 3339).
    #[serde(default)]
    pub updated_at: String,
}

impl Doc {
    /// Creates a document with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            updated_at: now_stamp(),
        }
    }

    /// Document statistics.
    pub fn counts(&self) -> WordCounts {
        count_words(&self.text)
    }
}

/// Custom field definition (`metadati.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    /// Field name.
    pub name: String,
    /// Hint shown in the inspector.
    #[serde(default)]
    pub hint: String,
}

/// Saved search (dynamic collection) persisted in `metadati.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDef {
    /// Name shown in the list.
    pub name: String,
    /// Text or regex to search.
    #[serde(default)]
    pub pattern: String,
    /// If `true`, a regular expression.
    #[serde(default)]
    pub regex: bool,
    /// If `true`, case sensitive.
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Project-global metadata (`metadati.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMeta {
    /// Keywords available across the project.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Saved searches (collections).
    #[serde(default)]
    pub collections: Vec<CollectionDef>,
    /// User-definable custom fields.
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
    /// Free notes on the project.
    #[serde(default)]
    pub notes: String,
}

/// A character's sheet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSheet {
    /// Id of the matching sheet node in the tree.
    pub node_id: NodeId,
    /// Character name.
    pub name: String,
    /// Role in the story.
    #[serde(default)]
    pub role: String,
    /// Free description.
    #[serde(default)]
    pub description: String,
    /// Character goal.
    #[serde(default)]
    pub goal: String,
    /// Main conflict.
    #[serde(default)]
    pub conflict: String,
    /// Guided-questionnaire answers (question → answer).
    #[serde(default)]
    pub questionnaire: HashMap<String, String>,
}

/// A location's sheet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationSheet {
    /// Id of the matching sheet node in the tree.
    pub node_id: NodeId,
    /// Location name.
    pub name: String,
    /// Free description.
    #[serde(default)]
    pub description: String,
    /// Sensory details (sights, sounds, smells…).
    #[serde(default)]
    pub sensory: String,
    /// Free memos.
    #[serde(default)]
    pub notes: String,
}

/// Project sheets.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sheets {
    /// Character sheets.
    #[serde(default)]
    pub characters: Vec<CharacterSheet>,
    /// Location sheets.
    #[serde(default)]
    pub locations: Vec<LocationSheet>,
}

/// Writing goals (`obiettivi.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Targets {
    /// Project goal in words (`None` = disabled).
    #[serde(default)]
    pub project_goal_words: Option<u64>,
    /// Daily goal in words (`None` = disabled).
    #[serde(default)]
    pub daily_goal_words: Option<u64>,
    /// Words written on the given date.
    #[serde(default)]
    pub words_today: u64,
    /// Date (YYYY-MM-DD) that `words_today` refers to.
    #[serde(default)]
    pub words_today_date: Option<String>,
}

/// Saved version entry of a document (the real snapshot lives in `snapshot/`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Snapshot id (used in the file name).
    pub id: String,
    /// Document it belongs to.
    pub doc_id: NodeId,
    /// Creation (RFC 3339).
    pub created_at: String,
    /// User-given label.
    #[serde(default)]
    pub label: String,
}

/// A complete PlotVale project, in memory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Header.
    pub manifest: Manifest,
    /// Node tree.
    pub tree: Tree,
    /// Texts indexed by node id.
    pub docs: HashMap<NodeId, Doc>,
    /// Global metadata.
    pub meta: ProjectMeta,
    /// Character and location sheets.
    pub sheets: Sheets,
    /// Goals.
    pub targets: Targets,
    /// Snapshot list (texts travel separately, see format).
    #[serde(default)]
    pub snapshots: Vec<SnapshotMeta>,
    /// Snapshot texts by id (not serialized in JSON: they live in `snapshot/`).
    #[serde(skip)]
    pub snapshot_texts: HashMap<String, String>,
}

impl Project {
    /// Creates a project with manuscript, research, and trash, plus an example scene.
    pub fn new(title: &str, author: &str, app_version: &str) -> Self {
        let tree = Tree::new("Manoscritto", "Ricerca", "Cestino");
        let stamp = now_stamp();
        let mut project = Self {
            manifest: Manifest {
                schema_version: SCHEMA_VERSION,
                app_version: app_version.to_string(),
                title: title.to_string(),
                author: author.to_string(),
                language: default_language(),
                created_at: stamp.clone(),
                updated_at: stamp,
            },
            tree,
            docs: HashMap::new(),
            meta: ProjectMeta::default(),
            sheets: Sheets::default(),
            targets: Targets::default(),
            snapshots: Vec::new(),
            snapshot_texts: HashMap::new(),
        };
        let draft = project.tree.draft_root;
        let chapter = project
            .tree
            .create(draft, crate::NodeKind::Folder, "Chapter 1")
            .expect("freshly created tree");
        let scene = project
            .tree
            .create(chapter, crate::NodeKind::Scene, "Scene 1")
            .expect("freshly created tree");
        project.docs.insert(
            scene,
            Doc::new(
                "It was a dark and stormy night…\n\nYour turn: delete this and start your story.",
            ),
        );
        project
    }

    /// A document's text (empty string if the node has no text).
    pub fn doc_text(&self, id: NodeId) -> &str {
        self.docs
            .get(&id)
            .map(|doc| doc.text.as_str())
            .unwrap_or("")
    }

    /// Sets a document's text, creating it if needed.
    pub fn set_doc_text(&mut self, id: NodeId, text: &str) {
        self.docs.entry(id).or_insert_with(|| Doc::new("")).text = text.to_string();
        if let Some(doc) = self.docs.get_mut(&id) {
            doc.updated_at = now_stamp();
        }
        self.touch();
    }

    /// Total words of all documents outside trash.
    pub fn total_words(&self) -> usize {
        self.tree
            .ids()
            .filter(|id| !self.tree.is_in_trash(*id))
            .map(|id| count_words(self.doc_text(id)).words)
            .sum()
    }

    /// Words of a subtree (chapter, folder…), trash excluded.
    pub fn subtree_words(&self, id: NodeId) -> usize {
        self.tree
            .subtree_ids(id)
            .into_iter()
            .filter(|child| !self.tree.is_in_trash(*child))
            .map(|child| count_words(self.doc_text(child)).words)
            .sum()
    }

    /// Refreshes the last-modified timestamp.
    pub fn touch(&mut self) {
        self.manifest.updated_at = now_stamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_has_example_scene() {
        let project = Project::new("Prova", "Autore", "0.1.0");
        assert!(project.total_words() > 0);
        project.tree.check_invariants().unwrap();
        assert_eq!(project.manifest.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn trash_is_excluded_from_counts() {
        let mut project = Project::new("Prova", "Autore", "0.1.0");
        let before = project.total_words();
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        project.tree.move_to_trash(cap).unwrap();
        assert_eq!(project.total_words(), 0);
        assert!(before > 0);
    }
}
