//! Albero del progetto: la "struttura" (binder) di PlotVale.
//!
//! Ogni elemento — cartella, scena, nota, scheda — è un [`Node`] con un [`NodeId`]
//! stabile. Le operazioni (`create`, `move_node`, `move_to_trash`, …) mantengono gli
//! invarianti e falliscono con [`TreeError`] invece di corrompere i dati.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::NodeId;

/// Errori delle operazioni sull'albero. Mai un panic su input utente.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeError {
    /// Il nodo richiesto non esiste.
    #[error("elemento non trovato: {0}")]
    NotFound(NodeId),
    /// Operazione vietata (es. spostare o eliminare una radice di sistema).
    #[error("operazione non consentita: {0}")]
    Forbidden(String),
    /// Lo spostamento creerebbe un ciclo (un nodo dentro sé stesso).
    #[error("spostamento non possibile: creerebbe un ciclo")]
    WouldCycle,
}

/// Tipo di un nodo dell'albero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Radice del manoscritto (sistema, non spostabile né eliminabile).
    DraftRoot,
    /// Cartella generica nel manoscritto.
    Folder,
    /// Unità di testo del manoscritto.
    Scene,
    /// Nota libera (non esportata di default).
    Note,
    /// Radice del materiale di ricerca (sistema).
    ResearchFolder,
    /// Allegato o appunto di ricerca.
    ResearchItem,
    /// Scheda personaggio.
    CharacterSheet,
    /// Scheda luogo.
    LocationSheet,
    /// Cestino (sistema).
    Trash,
}

impl NodeKind {
    /// I nodi di sistema esistono sempre e non si possono spostare né eliminare.
    pub fn is_system(self) -> bool {
        matches!(
            self,
            NodeKind::DraftRoot | NodeKind::ResearchFolder | NodeKind::Trash
        )
    }

    /// I nodi che possono contenere testo modificabile.
    pub fn can_have_text(self) -> bool {
        matches!(
            self,
            NodeKind::Scene
                | NodeKind::Note
                | NodeKind::ResearchItem
                | NodeKind::CharacterSheet
                | NodeKind::LocationSheet
        )
    }
}

/// Stato di avanzamento di un documento.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    /// Ancora da scrivere.
    #[default]
    Todo,
    /// Prima stesura.
    Draft,
    /// Rivisto.
    Revised,
    /// Finale.
    Final,
}

fn default_true() -> bool {
    true
}

/// Metadati di un nodo: tutto ciò che non è né titolo né posizione.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMeta {
    /// Stato di avanzamento.
    pub status: DocStatus,
    /// Etichetta libera (es. "punto di vista: Maria").
    #[serde(default)]
    pub label: String,
    /// Parole chiave per ricerca e raggruppamento.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Riassunto mostrato nel cartellone.
    #[serde(default)]
    pub synopsis: String,
    /// Se `false`, il documento è escluso dall'esportazione.
    #[serde(default = "default_true")]
    pub include_in_export: bool,
    /// Colore opzionale (`#rrggbb`) per cartellone ed etichette.
    #[serde(default)]
    pub color: Option<String>,
    /// Campi personalizzati definiti dall'utente (`metadati.json`).
    #[serde(default)]
    pub custom: HashMap<String, String>,
    /// Genitore precedente alla cestinatura (per il ripristino).
    #[serde(default)]
    pub trashed_from: Option<NodeId>,
}

impl Default for NodeMeta {
    fn default() -> Self {
        Self {
            status: DocStatus::default(),
            label: String::new(),
            keywords: Vec::new(),
            synopsis: String::new(),
            include_in_export: true,
            color: None,
            custom: HashMap::new(),
            trashed_from: None,
        }
    }
}

/// Un elemento dell'albero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Identificatore stabile: non cambia mai.
    pub id: NodeId,
    /// Genitore (`None` solo per le tre radici di sistema).
    pub parent: Option<NodeId>,
    /// Figli in ordine di visualizzazione ed esportazione.
    pub children: Vec<NodeId>,
    /// Tipo di elemento.
    pub kind: NodeKind,
    /// Titolo visibile.
    pub title: String,
    /// Metadati.
    #[serde(default)]
    pub meta: NodeMeta,
}

/// L'albero del progetto con le tre radici di sistema.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    nodes: HashMap<NodeId, Node>,
    /// Radice del manoscritto.
    pub draft_root: NodeId,
    /// Radice della ricerca.
    pub research_root: NodeId,
    /// Cestino.
    pub trash: NodeId,
}

impl Tree {
    /// Crea un albero vuoto con le tre radici di sistema.
    pub fn new(draft_title: &str, research_title: &str, trash_title: &str) -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            draft_root: NodeId::new(),
            research_root: NodeId::new(),
            trash: NodeId::new(),
        };
        for (id, kind, title) in [
            (tree.draft_root, NodeKind::DraftRoot, draft_title),
            (tree.research_root, NodeKind::ResearchFolder, research_title),
            (tree.trash, NodeKind::Trash, trash_title),
        ] {
            tree.nodes.insert(
                id,
                Node {
                    id,
                    parent: None,
                    children: Vec::new(),
                    kind,
                    title: title.to_string(),
                    meta: NodeMeta::default(),
                },
            );
        }
        tree
    }

    /// Restituisce un nodo per id.
    pub fn get(&self, id: NodeId) -> Result<&Node, TreeError> {
        self.nodes.get(&id).ok_or(TreeError::NotFound(id))
    }

    /// Restituisce un nodo modificabile per id.
    pub fn get_mut(&mut self, id: NodeId) -> Result<&mut Node, TreeError> {
        self.nodes.get_mut(&id).ok_or(TreeError::NotFound(id))
    }

    /// Numero di nodi (radici incluse).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// `true` se l'albero contiene solo le tre radici.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 3
    }

    /// Itera su tutti gli id dei nodi.
    pub fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Crea un figlio in coda al genitore.
    pub fn create(
        &mut self,
        parent: NodeId,
        kind: NodeKind,
        title: &str,
    ) -> Result<NodeId, TreeError> {
        let id = NodeId::new();
        self.create_with_id(id, parent, kind, title)?;
        Ok(id)
    }

    /// Come [`Tree::create`], ma con id assegnato dal chiamante (utile nei test).
    pub fn create_with_id(
        &mut self,
        id: NodeId,
        parent: NodeId,
        kind: NodeKind,
        title: &str,
    ) -> Result<(), TreeError> {
        if self.nodes.contains_key(&id) {
            return Err(TreeError::Forbidden(format!("id duplicato: {id}")));
        }
        let parent_node = self.get(parent)?;
        if parent_node.kind == NodeKind::Trash {
            return Err(TreeError::Forbidden(
                "non si può creare dentro il cestino: usa Ripristina".to_string(),
            ));
        }
        if kind.is_system() {
            return Err(TreeError::Forbidden(
                "le radici di sistema non si possono creare".to_string(),
            ));
        }
        self.nodes.insert(
            id,
            Node {
                id,
                parent: Some(parent),
                children: Vec::new(),
                kind,
                title: title.to_string(),
                meta: NodeMeta::default(),
            },
        );
        self.get_mut(parent)?.children.push(id);
        Ok(())
    }

    /// Cambia il titolo di un nodo.
    pub fn rename(&mut self, id: NodeId, title: &str) -> Result<(), TreeError> {
        self.get_mut(id)?.title = title.to_string();
        Ok(())
    }

    /// Sposta un nodo sotto un nuovo genitore, in posizione `pos` (limitata ai figli).
    pub fn move_node(
        &mut self,
        id: NodeId,
        new_parent: NodeId,
        pos: usize,
    ) -> Result<(), TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "le radici di sistema non si possono spostare".to_string(),
            ));
        }
        self.get(new_parent)?; // esiste?
        if id == new_parent || self.is_descendant(new_parent, id) {
            return Err(TreeError::WouldCycle);
        }
        if self.is_in_trash(id) || self.is_in_trash(new_parent) {
            return Err(TreeError::Forbidden(
                "usa il cestino (Sposta nel cestino / Ripristina) per questi elementi".to_string(),
            ));
        }
        if let Some(old_parent) = node.parent {
            self.get_mut(old_parent)?
                .children
                .retain(|child| *child != id);
        }
        let siblings = &mut self.get_mut(new_parent)?.children;
        let pos = pos.min(siblings.len());
        siblings.insert(pos, id);
        self.get_mut(id)?.parent = Some(new_parent);
        Ok(())
    }

    /// Sposta un nodo di una posizione tra i fratelli (`up = true` = verso l'alto).
    /// Restituisce `false` se è già all'estremo.
    pub fn reorder(&mut self, id: NodeId, up: bool) -> Result<bool, TreeError> {
        let parent = self.get(id)?.parent.ok_or_else(|| {
            TreeError::Forbidden("le radici non si possono riordinare".to_string())
        })?;
        let siblings = &mut self.get_mut(parent)?.children;
        let Some(pos) = siblings.iter().position(|child| *child == id) else {
            return Err(TreeError::NotFound(id));
        };
        if up && pos > 0 {
            siblings.swap(pos, pos - 1);
            Ok(true)
        } else if !up && pos + 1 < siblings.len() {
            siblings.swap(pos, pos + 1);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Rientra il nodo sotto il fratello precedente. `false` se non ha un precedente.
    pub fn indent(&mut self, id: NodeId) -> Result<bool, TreeError> {
        let parent = self.get(id)?.parent.ok_or_else(|| {
            TreeError::Forbidden("le radici non si possono rientrare".to_string())
        })?;
        let pos = self
            .get(parent)?
            .children
            .iter()
            .position(|child| *child == id)
            .ok_or(TreeError::NotFound(id))?;
        if pos == 0 {
            return Ok(false);
        }
        let prev = self.get(parent)?.children[pos - 1];
        let dest_len = self.get(prev)?.children.len();
        self.move_node(id, prev, dest_len)?;
        Ok(true)
    }

    /// Riporta il nodo al livello del genitore, subito dopo di lui.
    /// `false` se è già al primo livello.
    pub fn outdent(&mut self, id: NodeId) -> Result<bool, TreeError> {
        let parent = self.get(id)?.parent.ok_or_else(|| {
            TreeError::Forbidden("le radici non si possono spostare di livello".to_string())
        })?;
        let grandparent = match self.get(parent)?.parent {
            Some(grandparent) => grandparent,
            None => return Ok(false),
        };
        let parent_pos = self
            .get(grandparent)?
            .children
            .iter()
            .position(|child| *child == parent)
            .ok_or(TreeError::NotFound(parent))?;
        self.move_node(id, grandparent, parent_pos + 1)?;
        Ok(true)
    }

    /// Sposta un nodo (e i suoi figli) nel cestino, ricordando il genitore.
    pub fn move_to_trash(&mut self, id: NodeId) -> Result<(), TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "le radici di sistema non si possono cestinare".to_string(),
            ));
        }
        if self.is_in_trash(id) {
            return Err(TreeError::Forbidden(
                "l'elemento è già nel cestino".to_string(),
            ));
        }
        if let Some(old_parent) = node.parent {
            self.get_mut(old_parent)?
                .children
                .retain(|child| *child != id);
        }
        let trash = self.trash;
        self.get_mut(trash)?.children.push(id);
        let trashed = self.get_mut(id)?;
        trashed.meta.trashed_from = node.parent;
        trashed.parent = Some(trash);
        Ok(())
    }

    /// Ripristina un elemento dal cestino al genitore precedente
    /// (o alla radice del manoscritto se non più valido).
    pub fn restore(&mut self, id: NodeId) -> Result<(), TreeError> {
        if self.get(id)?.kind.is_system() {
            return Err(TreeError::Forbidden(
                "le radici di sistema non si possono ripristinare".to_string(),
            ));
        }
        if !self.is_in_trash(id) {
            return Err(TreeError::Forbidden(
                "solo gli elementi nel cestino si possono ripristinare".to_string(),
            ));
        }
        let wanted = self.get(id)?.meta.trashed_from;
        let dest = match wanted {
            Some(parent) if self.nodes.contains_key(&parent) && !self.is_in_trash(parent) => parent,
            _ => self.draft_root,
        };
        if let Some(old_parent) = self.get(id)?.parent {
            self.get_mut(old_parent)?
                .children
                .retain(|child| *child != id);
        }
        let dest_len = self.get(dest)?.children.len();
        self.get_mut(dest)?.children.push(id);
        let node = self.get_mut(id)?;
        node.parent = Some(dest);
        node.meta.trashed_from = None;
        let _ = dest_len;
        Ok(())
    }

    /// Elimina definitivamente un nodo e tutto il suo sottoalbero.
    /// Restituisce gli id eliminati (per pulire testi e snapshot).
    pub fn delete_permanently(&mut self, id: NodeId) -> Result<Vec<NodeId>, TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "le radici di sistema non si possono eliminare".to_string(),
            ));
        }
        if let Some(parent) = node.parent {
            self.get_mut(parent)?.children.retain(|child| *child != id);
        }
        let removed = self.subtree_ids(id);
        for gone in &removed {
            self.nodes.remove(gone);
        }
        Ok(removed)
    }

    /// Svuota il cestino. Restituisce gli id eliminati.
    pub fn empty_trash(&mut self) -> Vec<NodeId> {
        let children = self
            .get(self.trash)
            .map(|trash| trash.children.clone())
            .unwrap_or_default();
        let mut removed = Vec::new();
        for child in children {
            if let Ok(mut gone) = self.delete_permanently(child) {
                removed.append(&mut gone);
            }
        }
        removed
    }

    /// Figli diretti di un nodo, in ordine.
    pub fn children_of(&self, id: NodeId) -> Result<Vec<NodeId>, TreeError> {
        Ok(self.get(id)?.children.clone())
    }

    /// Tutti gli id del sottoalbero (nodo incluso), in ordine di visita.
    pub fn subtree_ids(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = vec![id];
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for child in node.children.iter().rev() {
                    out.push(*child);
                    stack.push(*child);
                }
            }
        }
        out
    }

    /// Titoli dalla radice al nodo (per breadcrumb e diagnostica).
    pub fn path_titles(&self, id: NodeId) -> Result<Vec<String>, TreeError> {
        let mut titles = vec![self.get(id)?.title.clone()];
        let mut current = self.get(id)?.parent;
        while let Some(parent) = current {
            let node = self.get(parent)?;
            titles.push(node.title.clone());
            current = node.parent;
        }
        titles.reverse();
        Ok(titles)
    }

    /// `true` se `maybe_child` è dentro il sottoalbero di `ancestor` (estremi esclusi).
    pub fn is_descendant(&self, maybe_child: NodeId, ancestor: NodeId) -> bool {
        let mut current = self.nodes.get(&maybe_child).and_then(|node| node.parent);
        while let Some(parent) = current {
            if parent == ancestor {
                return true;
            }
            current = self.nodes.get(&parent).and_then(|node| node.parent);
        }
        false
    }

    /// `true` se il nodo è nel cestino.
    pub fn is_in_trash(&self, id: NodeId) -> bool {
        id == self.trash || self.is_descendant(id, self.trash)
    }

    /// Verifica tutti gli invarianti: parentele coerenti, niente cicli, radici intatte,
    /// ogni nodo raggiungibile da una radice. Usata dai test e dal comando `verify`.
    pub fn check_invariants(&self) -> Result<(), TreeError> {
        // 1. Le tre radici esistono, sono radici e hanno il kind giusto.
        for (id, kind) in [
            (self.draft_root, NodeKind::DraftRoot),
            (self.research_root, NodeKind::ResearchFolder),
            (self.trash, NodeKind::Trash),
        ] {
            let root = self.get(id)?;
            if root.parent.is_some() {
                return Err(TreeError::Forbidden(format!("radice spostata: {id}")));
            }
            if root.kind != kind {
                return Err(TreeError::Forbidden(format!("radice alterata: {id}")));
            }
        }
        // 2. Coerenza padre/figli in entrambe le direzioni.
        for (id, node) in &self.nodes {
            if let Some(parent) = node.parent {
                let parent_node = self.get(parent)?;
                if !parent_node.children.contains(id) {
                    return Err(TreeError::Forbidden(format!(
                        "figlio orfano: {id} dichiara {parent} che non lo elenca"
                    )));
                }
            }
            for child in &node.children {
                let child_node = self.get(*child)?;
                if child_node.parent != Some(*id) {
                    return Err(TreeError::Forbidden(format!(
                        "genitore incoerente per {child}"
                    )));
                }
            }
        }
        // 3. Niente cicli e tutto raggiungibile: visita dalle radici.
        let mut seen = HashSet::new();
        let mut stack = vec![self.draft_root, self.research_root, self.trash];
        while let Some(current) = stack.pop() {
            if !seen.insert(current) {
                return Err(TreeError::Forbidden(format!(
                    "ciclo o doppio link: {current}"
                )));
            }
            let node = self.get(current)?;
            stack.extend(node.children.iter().rev().copied());
        }
        if seen.len() != self.nodes.len() {
            return Err(TreeError::Forbidden(
                "esistono nodi non raggiungibili dalle radici".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        let mut tree = Tree::new("Manoscritto", "Ricerca", "Cestino");
        let cap = tree
            .create(tree.draft_root, NodeKind::Folder, "Cap 1")
            .unwrap();
        tree.create(cap, NodeKind::Scene, "Scena 1").unwrap();
        tree.create(cap, NodeKind::Scene, "Scena 2").unwrap();
        tree
    }

    #[test]
    fn create_and_invariants() {
        let tree = sample();
        tree.check_invariants().unwrap();
        assert_eq!(tree.len(), 6);
    }

    #[test]
    fn cannot_cycle() {
        let mut tree = sample();
        let cap = tree.children_of(tree.draft_root).unwrap()[0];
        assert_eq!(
            tree.move_node(tree.draft_root, cap, 0),
            Err(TreeError::Forbidden(
                "le radici di sistema non si possono spostare".to_string()
            ))
        );
        assert_eq!(tree.move_node(cap, cap, 0), Err(TreeError::WouldCycle));
    }

    #[test]
    fn trash_and_restore_roundtrip() {
        let mut tree = sample();
        let cap = tree.children_of(tree.draft_root).unwrap()[0];
        tree.move_to_trash(cap).unwrap();
        assert!(tree.is_in_trash(cap));
        tree.check_invariants().unwrap();
        tree.restore(cap).unwrap();
        assert!(!tree.is_in_trash(cap));
        assert_eq!(tree.get(cap).unwrap().parent, Some(tree.draft_root));
        tree.check_invariants().unwrap();
    }

    #[test]
    fn reorder_indent_outdent() {
        let mut tree = sample();
        let cap = tree.children_of(tree.draft_root).unwrap()[0];
        let scenes = tree.children_of(cap).unwrap();
        assert!(tree.reorder(scenes[1], true).unwrap());
        assert_eq!(tree.children_of(cap).unwrap()[0], scenes[1]);
        assert!(tree.indent(scenes[0]).unwrap());
        assert_eq!(tree.get(scenes[0]).unwrap().parent, Some(scenes[1]));
        assert!(tree.outdent(scenes[0]).unwrap());
        assert_eq!(tree.get(scenes[0]).unwrap().parent, Some(cap));
        tree.check_invariants().unwrap();
    }

    #[test]
    fn delete_is_permanent_and_reported() {
        let mut tree = sample();
        let cap = tree.children_of(tree.draft_root).unwrap()[0];
        let removed = tree.delete_permanently(cap).unwrap();
        assert_eq!(removed.len(), 3);
        tree.check_invariants().unwrap();
    }
}
