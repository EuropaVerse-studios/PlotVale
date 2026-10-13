//! `plotvale-model` — il cuore dati di PlotVale.
//!
//! Qui vivono l'albero del progetto, i documenti, le schede, gli obiettivi e le
//! regole di conteggio parole. Questo crate non tocca file, rete o GUI: solo dati,
//! invarianti e test. Tutto ciò che l'utente scrive passa di qui.

mod count;
mod project;
mod tree;

pub use count::{WordCounts, count_words};
pub use project::{
    CharacterSheet, CollectionDef, CustomField, Doc, LocationSheet, Manifest, Project, ProjectMeta,
    SCHEMA_VERSION, Sheets, SnapshotMeta, Targets, now_stamp,
};
pub use tree::{DocStatus, Node, NodeKind, NodeMeta, Tree, TreeError};

/// Identificatore stabile di un nodo. Non cambia mai, nemmeno se rinomini o sposti
/// l'elemento: è la base di snapshot, backup e file `testi/<id>.md` affidabili.
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct NodeId(pub ulid::Ulid);

impl NodeId {
    /// Nuovo identificatore casuale ordinabile per tempo di creazione.
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
