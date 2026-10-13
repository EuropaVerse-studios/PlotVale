//! Property test: sequenze casuali di operazioni non rompono mai gli invarianti.
//!
//! Qualunque cosa faccia l'utente nel binder — anche la più strana — l'albero deve
//! restare coerente, o l'operazione deve fallire con errore (mai panic, mai cicli).

use proptest::prelude::*;
use plotvale_model::{NodeId, NodeKind, Tree};

/// Un'operazione casuale applicabile a un albero di prova.
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
                .expect("invarianti violati dopo una sequenza di operazioni lecite");
        }
        // Pulizia finale: tutto ciò che resta deve essere eliminabile senza errori.
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
        // Ogni elemento nel cestino deve essere ripristinabile.
        let trash_children = tree.children_of(tree.trash).unwrap();
        for child in trash_children {
            tree.restore(child)
                .expect("un elemento nel cestino deve essere sempre ripristinabile");
            assert!(!tree.is_in_trash(child));
        }
        tree.check_invariants().unwrap();
    }
}
