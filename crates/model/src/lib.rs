//! `plotvale-model` — the data heart of PlotVale.
//!
//! Home of the project tree, documents, sheets, goals, and word-count rules.
//! This crate touches no files, network, or GUI: data, invariants, and tests only.
//! Everything the user writes flows through here.

mod count;
mod project;
mod tree;

pub use count::{WordCounts, count_words};
pub use project::{
    CharacterSheet, CollectionDef, CustomField, Doc, LocationSheet, Manifest, Project, ProjectMeta,
    SCHEMA_VERSION, Sheets, SnapshotMeta, Targets, now_stamp,
};
pub use tree::{DocStatus, Node, NodeKind, NodeMeta, Tree, TreeError};

/// Stable identifier of a node. It never changes, even if you rename or move
/// the item: the basis for reliable snapshots, backups, and `testi/<id>.md` files.
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct NodeId(pub ulid::Ulid);

impl NodeId {
    /// New random identifier, sortable by creation time.
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for NodeId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ulid::Ulid::from_string(s).map(Self)
    }
}
