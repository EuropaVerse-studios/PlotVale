//! Project tree: the PlotVale outline (binder).
//!
//! Every item — folder, scene, note, sheet — is a [`Node`] with a stable
//! [`NodeId`]. Operations (`create`, `move_node`, `move_to_trash`, …) uphold the
//! invariants and fail with [`TreeError`] instead of corrupting data.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::NodeId;

/// Tree operation errors. Never a panic on user input.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeError {
    /// The requested node does not exist.
    #[error("item not found: {0}")]
    NotFound(NodeId),
    /// Forbidden operation (e.g. moving or deleting a system root).
    #[error("operation not allowed: {0}")]
    Forbidden(String),
    /// The move would create a cycle (a node inside itself).
    #[error("cannot move: it would create a cycle")]
    WouldCycle,
}

/// Kind of a tree node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Manuscript root (system, neither movable nor deletable).
    DraftRoot,
    /// Generic folder in the manuscript.
    Folder,
    /// Manuscript text unit.
    Scene,
    /// Free note (not exported by default).
    Note,
    /// Research material root (system).
    ResearchFolder,
    /// Research attachment or memo.
    ResearchItem,
    /// Character sheet.
    CharacterSheet,
    /// Location sheet.
    LocationSheet,
    /// Trash (system).
    Trash,
}

impl NodeKind {
    /// System nodes always exist and cannot be moved or deleted.
    pub fn is_system(self) -> bool {
        matches!(
            self,
            NodeKind::DraftRoot | NodeKind::ResearchFolder | NodeKind::Trash
        )
    }

    /// Nodes that can hold editable text.
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

/// Progress status of a document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    /// Yet to write.
    #[default]
    Todo,
    /// First draft.
    Draft,
    /// Revised.
    Revised,
    /// Final.
    Final,
}

fn default_true() -> bool {
    true
}

/// Metadata of a node: everything that is neither title nor position.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMeta {
    /// Progress status.
    pub status: DocStatus,
    /// Free label (e.g. "point of view: Mary").
    #[serde(default)]
    pub label: String,
    /// Keywords for search and grouping.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Summary shown on the corkboard.
    #[serde(default)]
    pub synopsis: String,
    /// If `false`, the document is excluded from export.
    #[serde(default = "default_true")]
    pub include_in_export: bool,
    /// Optional color (`#rrggbb`) for corkboard and labels.
    #[serde(default)]
    pub color: Option<String>,
    /// User-defined custom fields (`metadati.json`).
    #[serde(default)]
    pub custom: HashMap<String, String>,
    /// Parent before trashing (for restore).
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

/// One tree item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Stable identifier: never changes.
    pub id: NodeId,
    /// Parent (`None` only for the three system roots).
    pub parent: Option<NodeId>,
    /// Children in display and export order.
    pub children: Vec<NodeId>,
    /// Item kind.
    pub kind: NodeKind,
    /// Visible title.
    pub title: String,
    /// Metadata.
    #[serde(default)]
    pub meta: NodeMeta,
}

/// The project tree with its three system roots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    nodes: HashMap<NodeId, Node>,
    /// Manuscript root.
    pub draft_root: NodeId,
    /// Research root.
    pub research_root: NodeId,
    /// Trash.
    pub trash: NodeId,
}

impl Tree {
    /// Creates an empty tree with the three system roots.
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

    /// Returns a node by id.
    pub fn get(&self, id: NodeId) -> Result<&Node, TreeError> {
        self.nodes.get(&id).ok_or(TreeError::NotFound(id))
    }

    /// Returns a mutable node by id.
    pub fn get_mut(&mut self, id: NodeId) -> Result<&mut Node, TreeError> {
        self.nodes.get_mut(&id).ok_or(TreeError::NotFound(id))
    }

    /// Number of nodes (roots included).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// `true` if the tree holds only the three roots.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 3
    }

    /// Iterates over all node ids.
    pub fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Creates a child appended to the parent.
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

    /// Like [`Tree::create`], but with a caller-assigned id (handy in tests).
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
                "cannot create inside trash: use Restore".to_string(),
            ));
        }
        if kind.is_system() {
            return Err(TreeError::Forbidden(
                "system roots cannot be created".to_string(),
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

    /// Renames a node.
    pub fn rename(&mut self, id: NodeId, title: &str) -> Result<(), TreeError> {
        self.get_mut(id)?.title = title.to_string();
        Ok(())
    }

    /// Moves a node under a new parent, at position `pos` (clamped to the children).
    pub fn move_node(
        &mut self,
        id: NodeId,
        new_parent: NodeId,
        pos: usize,
    ) -> Result<(), TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "system roots cannot be moved".to_string(),
            ));
        }
        self.get(new_parent)?; // esiste?
        if id == new_parent || self.is_descendant(new_parent, id) {
            return Err(TreeError::WouldCycle);
        }
        if self.is_in_trash(id) || self.is_in_trash(new_parent) {
            return Err(TreeError::Forbidden(
                "use trash (Move to trash / Restore) for these items".to_string(),
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

    /// Moves a node one slot among siblings (`up = true` = upwards).
    /// Returns `false` if already at the edge.
    pub fn reorder(&mut self, id: NodeId, up: bool) -> Result<bool, TreeError> {
        let parent = self
            .get(id)?
            .parent
            .ok_or_else(|| TreeError::Forbidden("roots cannot be reordered".to_string()))?;
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

    /// Indents the node under the previous sibling. `false` if there is none.
    pub fn indent(&mut self, id: NodeId) -> Result<bool, TreeError> {
        let parent = self
            .get(id)?
            .parent
            .ok_or_else(|| TreeError::Forbidden("roots cannot be indented".to_string()))?;
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

    /// Outdents the node to the parent level, right after it.
    /// `false` if already at the top level.
    pub fn outdent(&mut self, id: NodeId) -> Result<bool, TreeError> {
        let parent = self
            .get(id)?
            .parent
            .ok_or_else(|| TreeError::Forbidden("roots cannot be outdented".to_string()))?;
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

    /// Moves a node (and its children) to trash, remembering the parent.
    pub fn move_to_trash(&mut self, id: NodeId) -> Result<(), TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "system roots cannot be trashed".to_string(),
            ));
        }
        if self.is_in_trash(id) {
            return Err(TreeError::Forbidden(
                "the item is already in trash".to_string(),
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

    /// Restores an item from trash to its previous parent
    /// (or the manuscript root if no longer valid).
    pub fn restore(&mut self, id: NodeId) -> Result<(), TreeError> {
        if self.get(id)?.kind.is_system() {
            return Err(TreeError::Forbidden(
                "system roots cannot be restored".to_string(),
            ));
        }
        if !self.is_in_trash(id) {
            return Err(TreeError::Forbidden(
                "only trashed items can be restored".to_string(),
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

    /// Permanently deletes a node and its whole subtree.
    /// Returns the deleted ids (to clean texts and snapshots).
    pub fn delete_permanently(&mut self, id: NodeId) -> Result<Vec<NodeId>, TreeError> {
        let node = self.get(id)?.clone();
        if node.kind.is_system() {
            return Err(TreeError::Forbidden(
                "system roots cannot be deleted".to_string(),
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

    /// Empties the trash. Returns the deleted ids.
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

    /// Direct children of a node, in order.
    pub fn children_of(&self, id: NodeId) -> Result<Vec<NodeId>, TreeError> {
        Ok(self.get(id)?.children.clone())
    }

    /// All ids in the subtree (node included), in visit order.
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

    /// Titles from the root to the node (for breadcrumbs and diagnostics).
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

    /// `true` if `maybe_child` is inside `ancestor`'s subtree (endpoints excluded).
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

    /// `true` if the node is in trash.
    pub fn is_in_trash(&self, id: NodeId) -> bool {
        id == self.trash || self.is_descendant(id, self.trash)
    }

    /// Checks all invariants: consistent parentage, no cycles, intact roots,
    /// every node reachable from a root. Used by tests and the `verify` command.
    pub fn check_invariants(&self) -> Result<(), TreeError> {
        // 1. The three roots exist, are roots, and have the right kind.
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
        // 2. Parent/child consistency in both directions.
        for (id, node) in &self.nodes {
            if let Some(parent) = node.parent {
                let parent_node = self.get(parent)?;
                if !parent_node.children.contains(id) {
                    return Err(TreeError::Forbidden(format!(
                        "orphan child: {id} names {parent}, which does not list it"
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
        // 3. No cycles and everything reachable: visit from the roots.
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
                "there are nodes unreachable from the roots".to_string(),
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
                "system roots cannot be moved".to_string()
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
