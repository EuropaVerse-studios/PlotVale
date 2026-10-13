//! Property test: random operation sequences never break the invariants.
//!
//! Whatever the user does in the binder — even the weirdest thing — the tree must
//! stay consistent, or the operation must fail with an error (never panic, never cycles).

use plotvale_model::{NodeId, NodeKind, Tree};
use proptest::prelude::*;

/// A random operation applicable to a test tree.
#[derive(Clone, Debug)]
enum Op {
    Create { parent_idx: usize, kind: u8 },
    Rename { node_idx: usize },
    Move { node_idx: usize, parent_idx: usize },
    Reorder { node_idx: usize, up: bool },
    Indent { node_idx: usize },
    Outdent { node_idx: usize },
    Trash { node_idx: usize },
    Restore { node_idx: usize },
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0usize..8, 0u8..4).prop_map(|(parent_idx, kind)| Op::Create { parent_idx, kind }),
        (0usize..8).prop_map(|node_idx| Op::Rename { node_idx }),
        (0usize..8, 0usize..8).prop_map(|(node_idx, parent_idx)| Op::Move {
            node_idx,
            parent_idx
        }),
        (0usize..8, any::<bool>()).prop_map(|(node_idx, up)| Op::Reorder { node_idx, up }),
        (0usize..8).prop_map(|node_idx| Op::Indent { node_idx }),
        (0usize..8).prop_map(|node_idx| Op::Outdent { node_idx }),
        (0usize..8).prop_map(|node_idx| Op::Trash { node_idx }),
        (0usize..8).prop_map(|node_idx| Op::Restore { node_idx }),
    ]
}

fn kind_from(byte: u8) -> NodeKind {
    match byte % 4 {
        0 => NodeKind::Folder,
        1 => NodeKind::Scene,
        2 => NodeKind::Note,
        _ => NodeKind::ResearchItem,
    }
}

fn pick(tree: &Tree, idx: usize) -> Option<NodeId> {
    let mut ids: Vec<NodeId> = tree.ids().collect();
    ids.sort();
    ids.get(idx % ids.len().max(1)).copied()
}

fn apply(tree: &mut Tree, op: &Op, counter: &mut usize) {
    *counter += 1;
    let title = format!("N{counter}");
    match op {
        Op::Create { parent_idx, kind } => {
            if let Some(parent) = pick(tree, *parent_idx) {
                let _ = tree.create(parent, kind_from(*kind), &title);
            }
        }
        Op::Rename { node_idx } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.rename(id, &title);
            }
        }
        Op::Move {
            node_idx,
            parent_idx,
        } => {
            if let (Some(id), Some(parent)) = (pick(tree, *node_idx), pick(tree, *parent_idx)) {
                let _ = tree.move_node(id, parent, parent_idx % 4);
            }
        }
        Op::Reorder { node_idx, up } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.reorder(id, *up);
            }
        }
        Op::Indent { node_idx } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.indent(id);
            }
        }
        Op::Outdent { node_idx } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.outdent(id);
            }
        }
        Op::Trash { node_idx } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.move_to_trash(id);
            }
        }
        Op::Restore { node_idx } => {
            if let Some(id) = pick(tree, *node_idx) {
                let _ = tree.restore(id);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn random_ops_keep_invariants(ops in proptest::collection::vec(op_strategy(), 0..60)) {
        let mut tree = Tree::new("Manoscritto", "Ricerca", "Cestino");
        let mut counter = 0usize;
        for op in &ops {
            apply(&mut tree, op, &mut counter);
            tree.check_invariants()
                .expect("invariants violated after a sequence of legal ops");
        }
        // Final cleanup: everything left must be deletable without errors.
        let trash_children = tree.children_of(tree.trash).unwrap();
        for child in trash_children {
            tree.delete_permanently(child).unwrap();
        }
        tree.check_invariants().unwrap();
    }

    #[test]
    fn trash_always_restores_or_reports(ops in proptest::collection::vec(op_strategy(), 0..40)) {
        let mut tree = Tree::new("Manoscritto", "Ricerca", "Cestino");
        let mut counter = 0usize;
        for op in &ops {
            apply(&mut tree, op, &mut counter);
        }
        // Every trashed item must be restorable.
        let trash_children = tree.children_of(tree.trash).unwrap();
        for child in trash_children {
            tree.restore(child)
                .expect("a trashed item must always be restorable");
            assert!(!tree.is_in_trash(child));
        }
        tree.check_invariants().unwrap();
    }
}
