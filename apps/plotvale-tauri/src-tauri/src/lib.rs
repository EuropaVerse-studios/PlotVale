//! PlotVale Tauri shell: owns the project via the GUI-less core crates and
//! exposes it to the React frontend as typed commands (see `commands`).

mod commands;
mod state;

use std::sync::Mutex;

use state::{AppState, InitialOpen};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // A path on the command line (file association, "Open with") opens at once;
    // the frontend collects it through `initial_project`.
    let (state, initial) = state::from_env();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(state))
        .manage(InitialProject(initial))
        .invoke_handler(tauri::generate_handler![
            commands::initial_project,
            commands::new_project,
            commands::open_project,
            commands::save_project,
            commands::save_project_as,
            commands::close_project,
            commands::project_status,
            commands::create_node,
            commands::rename_node,
            commands::move_node,
            commands::reorder_node,
            commands::indent_node,
            commands::outdent_node,
            commands::trash_node,
            commands::restore_node,
            commands::empty_trash,
            commands::delete_node,
            commands::get_tree,
            commands::get_node,
            commands::get_doc_text,
            commands::set_doc_text,
            commands::update_node_meta,
            commands::get_project_meta,
            commands::set_project_meta,
            commands::upsert_character,
            commands::upsert_location,
            commands::get_targets,
            commands::set_targets,
            commands::list_snapshots,
            commands::create_snapshot,
            commands::get_snapshot_text,
            commands::restore_snapshot,
            commands::delete_snapshot,
            commands::search_project,
            commands::project_stats,
            commands::doc_words,
            commands::total_words,
            commands::subtree_words,
            commands::list_profiles,
            commands::set_active_profile,
            commands::update_profile,
            commands::compile_preview,
            commands::export_manuscript,
            commands::list_collections,
            commands::add_collection,
            commands::remove_collection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Project opened before the window existed (see `state::from_env`).
///
/// Held outside `AppState` so `initial_project` can hand it over exactly once
/// while `AppState` keeps owning the same project for every other command.
struct InitialProject(Option<InitialOpen>);

