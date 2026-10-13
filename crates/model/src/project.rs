//! Documento di progetto: manifest, testi, schede, obiettivi, snapshot.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{NodeId, Tree, WordCounts, count_words};

/// Versione corrente del formato (vedi `docs/format-spec.md`).
pub const SCHEMA_VERSION: u32 = 1;

/// Timestamp RFC 3339 in UTC, per manifest, snapshot e documenti.
pub fn now_stamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Intestazione del progetto (`plotvale.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Versione dello schema del formato.
    pub schema_version: u32,
    /// Versione dell'app che ha salvato per ultima.
    pub app_version: String,
    /// Titolo dell'opera.
    pub title: String,
    /// Autore.
    #[serde(default)]
    pub author: String,
    /// Lingua principale (`it`, `en`, …).
    #[serde(default = "default_language")]
    pub language: String,
    /// Creazione del progetto.
    pub created_at: String,
    /// Ultimo salvataggio.
    pub updated_at: String,
}

fn default_language() -> String {
    "it".to_string()
}

/// Testo di un documento, con data di ultima modifica.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doc {
    /// Testo Markdown del documento.
    #[serde(default)]
    pub text: String,
    /// Ultima modifica (RFC 3339).
    #[serde(default)]
    pub updated_at: String,
}

impl Doc {
    /// Crea un documento con il testo dato.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            updated_at: now_stamp(),
        }
    }

    /// Statistiche del documento.
    pub fn counts(&self) -> WordCounts {
        count_words(&self.text)
    }
}

/// Definizione di un campo personalizzato (`metadati.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    /// Nome del campo.
    pub name: String,
    /// Suggerimento mostrato nell'inspector.
    #[serde(default)]
    pub hint: String,
}

/// Ricerca salvata (collezione dinamica) persistita in `metadati.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDef {
    /// Nome mostrato nella lista.
    pub name: String,
    /// Testo o regex da cercare.
    #[serde(default)]
    pub pattern: String,
    /// Se `true`, è un'espressione regolare.
    #[serde(default)]
    pub regex: bool,
    /// Se `true`, distingue maiuscole/minuscole.
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Metadati globali del progetto (`metadati.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMeta {
    /// Parole chiave disponibili in tutto il progetto.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Ricerche salvate (collezioni).
    #[serde(default)]
    pub collections: Vec<CollectionDef>,
    /// Campi personalizzati definibili dall'utente.
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
    /// Note libere sul progetto.
    #[serde(default)]
    pub notes: String,
}

/// Scheda di un personaggio.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSheet {
    /// Id del nodo scheda corrispondente nell'albero.
    pub node_id: NodeId,
    /// Nome del personaggio.
    pub name: String,
    /// Ruolo nella storia.
    #[serde(default)]
    pub role: String,
    /// Descrizione libera.
    #[serde(default)]
    pub description: String,
    /// Obiettivo del personaggio.
    #[serde(default)]
    pub goal: String,
    /// Conflitto principale.
    #[serde(default)]
    pub conflict: String,
    /// Risposte al questionario guidato (domanda → risposta).
    #[serde(default)]
    pub questionnaire: HashMap<String, String>,
}

/// Scheda di un luogo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationSheet {
    /// Id del nodo scheda corrispondente nell'albero.
    pub node_id: NodeId,
    /// Nome del luogo.
    pub name: String,
    /// Descrizione libera.
    #[serde(default)]
    pub description: String,
    /// Dettagli sensoriali (visti, suoni, odori…).
    #[serde(default)]
    pub sensory: String,
    /// Appunti liberi.
    #[serde(default)]
    pub notes: String,
}

/// Schede del progetto.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sheets {
    /// Schede personaggio.
    #[serde(default)]
    pub characters: Vec<CharacterSheet>,
    /// Schede luogo.
    #[serde(default)]
    pub locations: Vec<LocationSheet>,
}

/// Obiettivi di scrittura (`obiettivi.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Targets {
    /// Obiettivo del progetto in parole (`None` = disattivato).
    #[serde(default)]
    pub project_goal_words: Option<u64>,
    /// Obiettivo giornaliero in parole (`None` = disattivato).
    #[serde(default)]
    pub daily_goal_words: Option<u64>,
    /// Parole scritte nella data indicata.
    #[serde(default)]
    pub words_today: u64,
    /// Data (AAAA-MM-GG) a cui si riferisce `words_today`.
    #[serde(default)]
    pub words_today_date: Option<String>,
}

/// Voce di versione salvata di un documento (lo snapshot vero è in `snapshot/`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Id dello snapshot (usato nel nome file).
    pub id: String,
    /// Documento a cui appartiene.
    pub doc_id: NodeId,
    /// Creazione (RFC 3339).
    pub created_at: String,
    /// Etichetta data dall'utente.
    #[serde(default)]
    pub label: String,
}

/// Un progetto PlotVale completo, in memoria.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Intestazione.
    pub manifest: Manifest,
    /// Albero dei nodi.
    pub tree: Tree,
    /// Testi indicizzati per id di nodo.
    pub docs: HashMap<NodeId, Doc>,
    /// Metadati globali.
    pub meta: ProjectMeta,
    /// Schede personaggi e luoghi.
    pub sheets: Sheets,
    /// Obiettivi.
    pub targets: Targets,
    /// Elenco snapshot (i testi viaggiano a parte, vedi formato).
    #[serde(default)]
    pub snapshots: Vec<SnapshotMeta>,
    /// Testi degli snapshot per id (non serializzati nel JSON: vivono in `snapshot/`).
    #[serde(skip)]
    pub snapshot_texts: HashMap<String, String>,
}

impl Project {
    /// Crea un progetto con manoscritto, ricerca e cestino, più una scena d'esempio.
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
            .create(draft, crate::NodeKind::Folder, "Capitolo 1")
            .expect("albero appena creato");
        let scene = project
            .tree
            .create(chapter, crate::NodeKind::Scene, "Scena 1")
            .expect("albero appena creato");
        project.docs.insert(
            scene,
            Doc::new(
                "Era una notte buia e tempestosa…\n\nTocca a te: cancella e inizia la tua storia.",
            ),
        );
        project
    }

    /// Testo di un documento (stringa vuota se il nodo non ha testo).
    pub fn doc_text(&self, id: NodeId) -> &str {
        self.docs
            .get(&id)
            .map(|doc| doc.text.as_str())
            .unwrap_or("")
    }

    /// Imposta il testo di un documento, creandolo se serve.
    pub fn set_doc_text(&mut self, id: NodeId, text: &str) {
        self.docs.entry(id).or_insert_with(|| Doc::new("")).text = text.to_string();
        if let Some(doc) = self.docs.get_mut(&id) {
            doc.updated_at = now_stamp();
        }
        self.touch();
    }

    /// Parole totali di tutti i documenti fuori dal cestino.
    pub fn total_words(&self) -> usize {
        self.tree
            .ids()
            .filter(|id| !self.tree.is_in_trash(*id))
            .map(|id| count_words(self.doc_text(id)).words)
            .sum()
    }

    /// Parole di un sottoalbero (capitolo, cartella…), cestino escluso.
    pub fn subtree_words(&self, id: NodeId) -> usize {
        self.tree
            .subtree_ids(id)
            .into_iter()
            .filter(|child| !self.tree.is_in_trash(*child))
            .map(|child| count_words(self.doc_text(child)).words)
            .sum()
    }

    /// Aggiorna il timestamp di ultima modifica.
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
