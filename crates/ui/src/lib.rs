//! `plotvale-ui` — l'applicazione grafica di PlotVale (eframe/egui).
//!
//! Layout: struttura a sinistra, editor al centro (con cartellone e ricerca come
//! schede), scheda a destra. Tutto ciò che tocca i file passa per
//! `plotvale-project`; qui c'è solo stato dell'interfaccia.

pub mod brand;
mod diff;
mod i18n;
mod markdown;
mod theme;

pub use i18n::Locale;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use plotvale_compile::{CompileProfile, compile, expand_selection};
use plotvale_export::{ExportFormat, export_to_file};
use plotvale_model::{
    CharacterSheet, CollectionDef, DocStatus, LocationSheet, NodeId, NodeKind, Project, count_words,
};
use plotvale_project::{SaveOptions, find_recovery, open_any, save_any};
use plotvale_search::{Hit, Index, SearchOptions};

/// Versione dell'app (dal Cargo.toml).
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Secondi di inattività prima dell'autosalvataggio.
const AUTOSAVE_SECS: u64 = 3;
/// Domande guidate per le schede personaggio (amatoriali benvenuti).
const SHEET_QUESTIONS: &[&str] = &[
    "Aspetto e modo di fare",
    "Passato in poche righe",
    "La sua paura più grande",
    "Cosa vuole davvero",
];

/// Un progetto aperto con il suo percorso.
pub struct DocState {
    project: Project,
    profiles: plotvale_compile::ProfilesFile,
    path: Option<PathBuf>,
    folder_mode: bool,
}

/// Testo aperto nell'editor (specchio del documento nel modello).
#[derive(Clone, Debug, Default)]
struct DocView {
    text: String,
}

/// Schede del pannello centrale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tab {
    Welcome,
    Editor,
    Corkboard,
    Search,
}

/// Azione in attesa di conferma per modifiche non salvate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    New,
    OpenZip,
    OpenFolder,
    Quit,
}

/// Stato dell'interfaccia (niente I/O qui dentro).
struct UiState {
    selection: Option<NodeId>,
    open_doc: Option<NodeId>,
    views: HashMap<NodeId, DocView>,
    scrivenings: bool,
    collapsed: HashSet<NodeId>,
    renaming: Option<NodeId>,
    rename_buf: String,
    new_name_buf: String,
    delete_confirm: Option<NodeId>,
    pending: Option<PendingAction>,
    dirty: bool,
    last_edit: Option<Instant>,
    status: String,
    expert: bool,
    show_goals: bool,
    show_stats: bool,
    show_export: bool,
    show_about: bool,
    show_manual: bool,
    search_query: String,
    search_regex: bool,
    search_case: bool,
    search_results: Vec<Hit>,
    search_done: bool,
    search_error: Option<String>,
    new_query_name: String,
    snapshot_label: String,
    show_snapshot: Option<(NodeId, usize)>,
    export_preset: String,
    export_notes: bool,
    custom_name: String,
    custom_value: String,
    last_total_words: usize,
    want_quit: bool,
    confirm_empty_trash: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selection: None,
            open_doc: None,
            views: HashMap::new(),
            scrivenings: false,
            collapsed: HashSet::new(),
            renaming: None,
            rename_buf: String::new(),
            new_name_buf: String::new(),
            delete_confirm: None,
            pending: None,
            dirty: false,
            last_edit: None,
            status: String::new(),
            expert: false,
            show_goals: false,
            show_stats: false,
            show_export: false,
            show_about: false,
            show_manual: false,
            search_query: String::new(),
            search_regex: false,
            search_case: false,
            search_results: Vec::new(),
            search_done: false,
            search_error: None,
            new_query_name: String::new(),
            snapshot_label: String::new(),
            show_snapshot: None,
            export_preset: "manoscritto".to_string(),
            export_notes: false,
            custom_name: String::new(),
            custom_value: String::new(),
            last_total_words: 0,
            want_quit: false,
            confirm_empty_trash: false,
        }
    }
}

/// L'applicazione PlotVale.
pub struct PlotValeApp {
    doc: Option<DocState>,
    locale: Locale,
    dock: egui_dock::DockState<Tab>,
    ui: UiState,
    logo: Option<egui::TextureHandle>,
}

impl PlotValeApp {
    /// Crea l'app (chiamata da `eframe::run_native`).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(theme::brand_visuals());
        let logo = brand::decode_logo(192).map(|img| {
            cc.egui_ctx.load_texture(
                "plotvale-logo",
                img,
                egui::TextureOptions::LINEAR,
            )
        });
        Self {
            doc: None,
            locale: Locale::italian(),
            logo,
            dock: egui_dock::DockState::new(vec![
                Tab::Welcome,
                Tab::Editor,
                Tab::Corkboard,
                Tab::Search,
            ]),
            ui: UiState::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Azioni sui documenti (niente widget qui: solo logica + I/O)
// ---------------------------------------------------------------------------

/// Riversa gli editor aperti nel modello.
fn flush_views(doc: &mut DocState, ui: &mut UiState) {
    for (id, view) in &ui.views {
        doc.project.set_doc_text(*id, &view.text);
    }
}

/// Salva il progetto. Restituisce `true` se salvato.
fn do_save(doc: &mut DocState, ui: &mut UiState, loc: &Locale) -> bool {
    flush_views(doc, ui);
    let Some(path) = doc.path.clone() else {
        return false;
    };
    match save_any(
        &mut doc.project,
        &doc.profiles,
        &path,
        &SaveOptions::default(),
    ) {
        Ok(()) => {
            ui.dirty = false;
            ui.status = loc.get("status_saved").to_string();
            record_words(doc, ui);
            true
        }
        Err(e) => {
            ui.status = format!("{}: {e}", loc.get("err_save"));
            false
        }
    }
}

/// Aggiorna le parole odierne dopo un salvataggio.
fn record_words(doc: &mut DocState, ui: &mut UiState) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if doc.project.targets.words_today_date.as_deref() != Some(&today) {
        doc.project.targets.words_today = 0;
        doc.project.targets.words_today_date = Some(today);
    }
    let total = doc.project.total_words();
    if total > ui.last_total_words {
        doc.project.targets.words_today += (total - ui.last_total_words) as u64;
    }
    ui.last_total_words = total;
}

/// Apre un progetto, gestendo il recupero dopo crash.
fn do_open(path: PathBuf, doc_slot: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    // Recupero dopo crash: un .tmp più recente vale più dell'originale.
    let mut real = path.clone();
    let mut recovered = false;
    if let Some(tmp) = find_recovery(&path) {
        real = tmp;
        recovered = true;
    }
    match open_any(&real) {
        Ok(opened) => {
            let folder_mode = path.is_dir();
            let total = opened.project.total_words();
            *doc_slot = Some(DocState {
                project: opened.project,
                profiles: opened.profiles,
                path: Some(path),
                folder_mode,
            });
            *ui = UiState {
                last_total_words: total,
                export_preset: doc_slot
                    .as_ref()
                    .map(|d| d.profiles.active.clone())
                    .unwrap_or_default(),
                status: if recovered {
                    "Recuperato dopo la chiusura inattesa — salva per consolidare".to_string()
                } else {
                    String::new()
                },
                dirty: recovered,
                expert: ui.expert,
                ..UiState::default()
            };
            // Seleziona la radice del manoscritto.
            if let Some(doc) = doc_slot {
                ui.selection = Some(doc.project.tree.draft_root);
            }
        }
        Err(e) => {
            ui.status = format!("{}: {e}", loc.get("err_open"));
        }
    }
}

/// Nuovo progetto vuoto in memoria.
fn fresh_doc(doc_slot: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    let project = Project::new(loc.get("untitled"), "", APP_VERSION);
    let total = project.total_words();
    let draft = project.tree.draft_root;
    *doc_slot = Some(DocState {
        project,
        profiles: plotvale_compile::ProfilesFile::default(),
        path: None,
        folder_mode: false,
    });
    let expert = ui.expert;
    *ui = UiState {
        last_total_words: total,
        selection: Some(draft),
        expert,
        ..UiState::default()
    };
    ui.dirty = true;
    ui.status = loc.get("status_unsaved").to_string();
}

/// Assicura che il documento abbia una vista editor aperta.
fn ensure_view(doc: &DocState, ui: &mut UiState, id: NodeId) {
    ui.views.entry(id).or_insert_with(|| DocView {
        text: doc.project.doc_text(id).to_string(),
    });
}

/// Elimina definitivamente un sottoalbero e pulisce testi, viste, schede e snapshot.
fn purge_subtree(doc: &mut DocState, ui: &mut UiState, removed: &[NodeId]) {
    for id in removed {
        doc.project.docs.remove(id);
        ui.views.remove(id);
        if Some(*id) == ui.open_doc {
            ui.open_doc = None;
        }
        if Some(*id) == ui.selection {
            ui.selection = None;
        }
    }
    let gone: HashSet<NodeId> = removed.iter().copied().collect();
    doc.project
        .sheets
        .characters
        .retain(|s| !gone.contains(&s.node_id));
    doc.project
        .sheets
        .locations
        .retain(|s| !gone.contains(&s.node_id));
    let mut snap_gone = Vec::new();
    doc.project.snapshots.retain(|s| {
        let keep = !gone.contains(&s.doc_id);
        if !keep {
            snap_gone.push(s.id.clone());
        }
        keep
    });
    for id in snap_gone {
        doc.project.snapshot_texts.remove(&id);
    }
    ui.dirty = true;
}

/// Crea un nodo testuale (e la scheda collegata, se serve).
fn create_node(
    doc: &mut DocState,
    ui: &mut UiState,
    loc: &Locale,
    kind: NodeKind,
    default_title: &str,
) {
    let parent = ui
        .selection
        .filter(|id| {
            doc.project.tree.get(*id).is_ok_and(|n| {
                !doc.project.tree.is_in_trash(*id)
                    && matches!(
                        n.kind,
                        NodeKind::DraftRoot
                            | NodeKind::Folder
                            | NodeKind::ResearchFolder
                            | NodeKind::ResearchItem
                    )
            })
        })
        .unwrap_or(doc.project.tree.draft_root);
    // La ricerca vive sotto la ricerca, il resto sotto il manoscritto.
    let parent = match kind {
        NodeKind::ResearchItem if parent == doc.project.tree.draft_root => {
            doc.project.tree.research_root
        }
        _ => parent,
    };
    let title = if ui.new_name_buf.trim().is_empty() {
        default_title.to_string()
    } else {
        ui.new_name_buf.trim().to_string()
    };
    ui.new_name_buf.clear();
    if let Ok(id) = doc.project.tree.create(parent, kind, &title) {
        match kind {
            NodeKind::CharacterSheet => {
                doc.project.sheets.characters.push(CharacterSheet {
                    node_id: id,
                    name: title.clone(),
                    role: String::new(),
                    description: String::new(),
                    goal: String::new(),
                    conflict: String::new(),
                    questionnaire: HashMap::new(),
                });
            }
            NodeKind::LocationSheet => {
                doc.project.sheets.locations.push(LocationSheet {
                    node_id: id,
                    name: title.clone(),
                    description: String::new(),
                    sensory: String::new(),
                    notes: String::new(),
                });
            }
            _ => {}
        }
        ui.collapsed.remove(&parent);
        ui.selection = Some(id);
        if kind.can_have_text() {
            ensure_view(doc, ui, id);
            ui.open_doc = Some(id);
        }
        ui.dirty = true;
        ui.status = String::new();
    } else {
        ui.status = loc.get("err_invalid_project").to_string();
    }
}

// ---------------------------------------------------------------------------
// App: frame principale
// ---------------------------------------------------------------------------

impl eframe::App for PlotValeApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        // Scorciatoie globali.
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) && self.doc.is_some()
        {
            if self.doc.as_ref().is_some_and(|d| d.path.is_none()) {
                if let Some(doc) = self.doc.as_mut() {
                    save_as_dialog(doc, &mut self.ui, &self.locale);
                }
            } else if let Some(doc) = self.doc.as_mut() {
                do_save(doc, &mut self.ui, &self.locale);
            }
        }

        let Self {
            doc,
            locale,
            dock,
            ui: uistate,
            logo,
        } = self;
        menu_bar(root, doc, uistate, locale);
        // Cambio lingua richiesto dal menu.
        if uistate.status == "lang:it" {
            *locale = Locale::italian();
            uistate.status.clear();
        } else if uistate.status == "lang:en" {
            *locale = Locale::english();
            uistate.status.clear();
        }
        status_bar(root, doc, uistate, locale);
        dialogs(&ctx, doc, uistate, locale);

        // Struttura e scheda sono pannelli fissi; il centro è agganciabile.
        if doc.is_some() {
            egui::Panel::left("binder")
                .resizable(true)
                .default_size(260.0)
                .show(root, |panel| {
                    binder_panel(doc, uistate, locale, panel);
                });
            egui::Panel::right("inspector")
                .resizable(true)
                .default_size(300.0)
                .show(root, |panel| {
                    inspector_panel(doc, uistate, locale, panel);
                });
        }
        egui::CentralPanel::default().show(root, |center| {
            let mut viewer = Viewer {
                doc,
                ui: uistate,
                loc: locale,
                logo: logo.clone(),
            };
            egui_dock::DockArea::new(dock).show_inside(center, &mut viewer);
        });

        // Autosalvataggio dopo 3 s di inattività.
        let need_save = uistate.dirty
            && uistate
                .last_edit
                .is_some_and(|t| t.elapsed() > Duration::from_secs(AUTOSAVE_SECS))
            && doc.as_ref().is_some_and(|d| d.path.is_some());
        if need_save && let Some(d) = doc.as_mut() {
            do_save(d, uistate, locale);
        }
        if uistate.dirty {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}

/// Dialogo "Salva con nome" (solo bundle `.plotvale`).
fn save_as_dialog(doc: &mut DocState, ui: &mut UiState, loc: &Locale) {
    let file = rfd::FileDialog::new()
        .set_title(loc.get("save_title"))
        .add_filter(loc.get("filter_project"), &["plotvale"])
        .set_file_name(format!("{}.plotvale", doc.project.manifest.title))
        .save_file();
    if let Some(path) = file {
        flush_views(doc, ui);
        doc.path = Some(path);
        doc.folder_mode = false;
        do_save(doc, ui, loc);
    }
}

/// Vista per `egui_dock`: tutto lo stato mutabile passa di qui.
struct Viewer<'a> {
    doc: &'a mut Option<DocState>,
    ui: &'a mut UiState,
    loc: &'a Locale,
    logo: Option<egui::TextureHandle>,
}

impl egui_dock::TabViewer for Viewer<'_> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn is_closeable(&self, _tab: &Tab) -> bool {
        false
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        self.loc
            .get(match tab {
                Tab::Welcome => "tab_welcome",
                Tab::Editor => "tab_editor",
                Tab::Corkboard => "tab_corkboard",
                Tab::Search => "tab_search",
            })
            .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab {
            Tab::Welcome => view_welcome(self.doc, self.ui, self.loc, self.logo.clone(), ui),
            Tab::Editor => view_editor(self.doc, self.ui, self.loc, ui),
            Tab::Corkboard => view_corkboard(self.doc, self.ui, self.loc, ui),
            Tab::Search => view_search(self.doc, self.ui, self.loc, ui),
        }
    }
}

// ---------------------------------------------------------------------------
// Menu, barra di stato, dialoghi
// ---------------------------------------------------------------------------

/// Esegue l'azione o la mette in attesa se ci sono modifiche non salvate.
fn request_action(
    doc: &mut Option<DocState>,
    ui: &mut UiState,
    loc: &Locale,
    action: PendingAction,
) {
    if matches!(action, PendingAction::Quit) && (!ui.dirty || doc.is_none()) {
        ui.want_quit = true;
        return;
    }
    if ui.dirty && doc.is_some() {
        ui.pending = Some(action);
    } else {
        run_pending(doc, ui, loc, action);
    }
}

fn run_pending(doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale, action: PendingAction) {
    match action {
        PendingAction::New => fresh_doc(doc, ui, loc),
        PendingAction::OpenZip => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(loc.get("open_title_file"))
                .add_filter(loc.get("filter_project"), &["plotvale"])
                .pick_file()
            {
                do_open(path, doc, ui, loc);
            }
        }
        PendingAction::OpenFolder => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(loc.get("open_title_folder"))
                .pick_folder()
            {
                do_open(path, doc, ui, loc);
            }
        }
        PendingAction::Quit => ui.want_quit = true,
    }
}

fn menu_bar(root: &mut egui::Ui, doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    egui::Panel::top("menu").show(root, |bar| {
        bar.horizontal(|bar| {
            bar.menu_button(loc.get("menu_file"), |menu| {
                if menu.button(loc.get("menu_new")).clicked() {
                    request_action(doc, ui, loc, PendingAction::New);
                    menu.close();
                }
                if menu.button(loc.get("menu_open_file")).clicked() {
                    request_action(doc, ui, loc, PendingAction::OpenZip);
                    menu.close();
                }
                if menu.button(loc.get("menu_open_folder")).clicked() {
                    request_action(doc, ui, loc, PendingAction::OpenFolder);
                    menu.close();
                }
                menu.separator();
                let has_doc = doc.is_some();
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_save")))
                    .clicked()
                {
                    if doc.as_ref().is_some_and(|d| d.path.is_none()) {
                        if let Some(d) = doc.as_mut() {
                            save_as_dialog(d, ui, loc);
                        }
                    } else if let Some(d) = doc.as_mut() {
                        do_save(d, ui, loc);
                    }
                    menu.close();
                }
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_save_as")))
                    .clicked()
                {
                    if let Some(d) = doc.as_mut() {
                        save_as_dialog(d, ui, loc);
                    }
                    menu.close();
                }
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_export")))
                    .clicked()
                {
                    ui.show_export = true;
                    menu.close();
                }
                menu.separator();
                if menu.button(loc.get("menu_quit")).clicked() {
                    request_action(doc, ui, loc, PendingAction::Quit);
                    menu.close();
                }
            });
            bar.menu_button(loc.get("menu_project"), |menu| {
                let has_doc = doc.is_some();
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_new_scene")))
                    .clicked()
                {
                    if let Some(d) = doc.as_mut() {
                        create_node(d, ui, loc, NodeKind::Scene, loc.get("scene_label"));
                    }
                    menu.close();
                }
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_new_folder")))
                    .clicked()
                {
                    if let Some(d) = doc.as_mut() {
                        create_node(d, ui, loc, NodeKind::Folder, loc.get("chapter_label"));
                    }
                    menu.close();
                }
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_new_note")))
                    .clicked()
                {
                    if let Some(d) = doc.as_mut() {
                        create_node(d, ui, loc, NodeKind::Note, loc.get("kind_note"));
                    }
                    menu.close();
                }
                menu.separator();
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_goals")))
                    .clicked()
                {
                    ui.show_goals = true;
                    menu.close();
                }
                if menu
                    .add_enabled(has_doc, egui::Button::new(loc.get("menu_stats")))
                    .clicked()
                {
                    ui.show_stats = true;
                    menu.close();
                }
            });
            bar.menu_button(loc.get("menu_view"), |menu| {
                menu.checkbox(&mut ui.expert, loc.get("menu_expert"));
                menu.checkbox(&mut ui.scrivenings, loc.get("view_scrivenings"));
                menu.separator();
                menu.menu_button(loc.get("menu_language"), |lang| {
                    if lang.button(loc.get("lang_it")).clicked() {
                        ui.status = "lang:it".to_string();
                        lang.close();
                    }
                    if lang.button(loc.get("lang_en")).clicked() {
                        ui.status = "lang:en".to_string();
                        lang.close();
                    }
                });
            });
            bar.menu_button(loc.get("menu_help"), |menu| {
                if menu.button(loc.get("menu_manual")).clicked() {
                    ui.show_manual = true;
                    menu.close();
                }
                if menu.button(loc.get("menu_about")).clicked() {
                    ui.show_about = true;
                    menu.close();
                }
            });
        });
    });
}

fn status_bar(root: &mut egui::Ui, doc: &Option<DocState>, ui: &UiState, loc: &Locale) {
    egui::Panel::bottom("status").show(root, |bar| {
        bar.horizontal(|bar| {
            if let Some(d) = doc {
                let name = d
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| d.project.manifest.title.clone());
                bar.label(format!("{} {}", if ui.dirty { "●" } else { "○" }, name));
                if !ui.status.is_empty()
                    && !ui.status.starts_with("lang:")
                    && ui.status != loc.get("status_saved")
                {
                    bar.separator();
                    bar.label(&ui.status);
                }
                bar.with_layout(egui::Layout::right_to_left(egui::Align::Center), |bar| {
                    bar.small(format!("[{}]", ui_lang_name(loc)));
                    bar.separator();
                    let total = d.project.total_words();
                    bar.small(format!(
                        "{} {} {}",
                        total,
                        loc.get("status_words"),
                        loc.get("status_total")
                    ));
                    if let Some(open) = ui.open_doc {
                        let words = count_words(d.project.doc_text(open)).words;
                        bar.separator();
                        bar.small(format!("{} {}", words, loc.get("status_words")));
                    }
                });
            } else {
                bar.label(loc.get("status_no_project"));
            }
        });
    });
}

fn ui_lang_name(loc: &Locale) -> &str {
    if loc.code == "en" { "EN" } else { "IT" }
}

// ---------------------------------------------------------------------------
// Struttura (binder)
// ---------------------------------------------------------------------------

fn kind_icon(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::DraftRoot => "\u{1f4d6}",
        NodeKind::Folder => "\u{1f4c1}",
        NodeKind::Scene => "\u{1f4c4}",
        NodeKind::Note => "\u{1f4dd}",
        NodeKind::ResearchFolder => "\u{1f52c}",
        NodeKind::ResearchItem => "\u{1f4ce}",
        NodeKind::CharacterSheet => "\u{1f464}",
        NodeKind::LocationSheet => "\u{1f4cd}",
        NodeKind::Trash => "\u{1f5d1}",
    }
}

fn status_dot(status: DocStatus) -> (&'static str, egui::Color32) {
    match status {
        DocStatus::Todo => ("\u{25cb}", egui::Color32::GRAY),
        DocStatus::Draft => ("\u{25cf}", egui::Color32::YELLOW),
        DocStatus::Revised => ("\u{25cf}", egui::Color32::LIGHT_BLUE),
        DocStatus::Final => ("\u{25cf}", egui::Color32::GREEN),
    }
}

fn binder_panel(doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale, panel: &mut egui::Ui) {
    let Some(d) = doc.as_mut() else { return };
    panel.heading(loc.get("binder"));
    panel.horizontal(|row| {
        if row
            .menu_button(format!("+ {}", loc.get("dlg_new_title")), |menu| {
                menu.label(loc.get("dlg_new_name"));
                menu.text_edit_singleline(&mut ui.new_name_buf);
                menu.separator();
                let kinds = [
                    (NodeKind::Scene, loc.get("menu_new_scene")),
                    (NodeKind::Folder, loc.get("menu_new_folder")),
                    (NodeKind::Note, loc.get("menu_new_note")),
                    (NodeKind::ResearchItem, loc.get("ctx_new_research")),
                    (NodeKind::CharacterSheet, loc.get("ctx_new_character")),
                    (NodeKind::LocationSheet, loc.get("ctx_new_location")),
                ];
                for (kind, label) in kinds {
                    if menu.button(label).clicked() {
                        let title = if ui.new_name_buf.trim().is_empty() {
                            label.to_string()
                        } else {
                            ui.new_name_buf.trim().to_string()
                        };
                        ui.new_name_buf.clear();
                        // Parent adatto: la selezione se contenitore, altrimenti la radice.
                        let parent = ui.selection.filter(|id| {
                            d.project.tree.get(*id).is_ok_and(|n| {
                                !d.project.tree.is_in_trash(*id)
                                    && matches!(
                                        n.kind,
                                        NodeKind::DraftRoot
                                            | NodeKind::Folder
                                            | NodeKind::ResearchFolder
                                            | NodeKind::ResearchItem
                                    )
                            })
                        });
                        let mut parent = parent.unwrap_or(d.project.tree.draft_root);
                        if kind == NodeKind::ResearchItem
                            && d.project.tree.get(parent).is_ok_and(|n| {
                                matches!(n.kind, NodeKind::DraftRoot | NodeKind::Folder)
                            })
                        {
                            parent = d.project.tree.research_root;
                        }
                        if let Ok(id) = d.project.tree.create(parent, kind, &title) {
                            match kind {
                                NodeKind::CharacterSheet => {
                                    d.project.sheets.characters.push(CharacterSheet {
                                        node_id: id,
                                        name: title.clone(),
                                        role: String::new(),
                                        description: String::new(),
                                        goal: String::new(),
                                        conflict: String::new(),
                                        questionnaire: HashMap::new(),
                                    });
                                }
                                NodeKind::LocationSheet => {
                                    d.project.sheets.locations.push(LocationSheet {
                                        node_id: id,
                                        name: title.clone(),
                                        description: String::new(),
                                        sensory: String::new(),
                                        notes: String::new(),
                                    });
                                }
                                _ => {}
                            }
                            d.project.touch();
                            ui.collapsed.remove(&parent);
                            ui.selection = Some(id);
                            if kind.can_have_text() {
                                ensure_view(d, ui, id);
                                ui.open_doc = Some(id);
                            }
                            ui.dirty = true;
                        }
                        menu.close();
                    }
                }
            })
            .response
            .clicked()
        {}
    });
    panel.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(panel, |scroll| {
            let roots = d.project.tree.draft_root;
            let research = d.project.tree.research_root;
            let trash = d.project.tree.trash;
            for root in [roots, research, trash] {
                node_row(d, ui, loc, scroll, root);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn node_row(d: &mut DocState, ui: &mut UiState, loc: &Locale, egui_ui: &mut egui::Ui, id: NodeId) {
    let Ok(node) = d.project.tree.get(id).cloned() else {
        return;
    };
    // Rinomina inline.
    if ui.renaming == Some(id) {
        egui_ui.horizontal(|row| {
            let resp = row.text_edit_singleline(&mut ui.rename_buf);
            let commit = resp.lost_focus() && row.input(|i| i.key_pressed(egui::Key::Enter));
            let cancel = row.input(|i| i.key_pressed(egui::Key::Escape));
            if commit && !ui.rename_buf.trim().is_empty() {
                let title = ui.rename_buf.trim().to_string();
                let _ = d.project.tree.rename(id, &title);
                d.project.touch();
                ui.dirty = true;
                ui.renaming = None;
            } else if cancel || resp.lost_focus() {
                ui.renaming = None;
            }
        });
        return;
    }
    let selected = ui.selection == Some(id);
    let collapsed = ui.collapsed.contains(&id);
    let has_children = !node.children.is_empty();
    let in_trash = d.project.tree.is_in_trash(id);
    egui_ui.horizontal(|row| {
        if has_children {
            if row
                .small_button(if collapsed { "\u{25b8}" } else { "\u{25be}" })
                .clicked()
            {
                if collapsed {
                    ui.collapsed.remove(&id);
                } else {
                    ui.collapsed.insert(id);
                }
            }
        } else {
            row.add_space(18.0);
        }
        let (dot, color) = status_dot(node.meta.status);
        let label = if matches!(node.kind, NodeKind::Scene | NodeKind::Note) {
            egui::RichText::new(format!("{} {} {}", kind_icon(node.kind), dot, node.title)).color(
                if selected {
                    egui::Color32::WHITE
                } else {
                    row.visuals().widgets.noninteractive.fg_stroke.color
                },
            )
        } else {
            egui::RichText::new(format!("{} {}", kind_icon(node.kind), node.title)).color(
                if selected {
                    egui::Color32::WHITE
                } else {
                    row.visuals().widgets.noninteractive.fg_stroke.color
                },
            )
        };
        let _ = color;
        let resp = row.selectable_label(selected, label);
        if resp.clicked() {
            ui.selection = Some(id);
        }
        if resp.double_clicked() {
            if node.kind.can_have_text() {
                ensure_view(d, ui, id);
                ui.open_doc = Some(id);
            } else if has_children {
                if collapsed {
                    ui.collapsed.remove(&id);
                } else {
                    ui.collapsed.insert(id);
                }
            }
        }
        resp.context_menu(|menu| {
            node_context_menu(d, ui, loc, menu, id, in_trash);
        });
    });
    if has_children && !collapsed {
        egui_ui.horizontal(|row| {
            row.add_space(16.0);
            row.vertical(|col| {
                for child in node.children.clone() {
                    node_row(d, ui, loc, col, child);
                }
            });
        });
    }
}

fn node_context_menu(
    d: &mut DocState,
    ui: &mut UiState,
    loc: &Locale,
    menu: &mut egui::Ui,
    id: NodeId,
    in_trash: bool,
) {
    let kind = d.project.tree.get(id).map(|n| n.kind);
    if id == d.project.tree.trash {
        if menu.button(loc.get("ctx_empty_trash")).clicked() {
            ui.confirm_empty_trash = true;
            menu.close();
        }
        return;
    }
    if in_trash {
        if menu.button(loc.get("ctx_restore")).clicked() {
            let _ = d.project.tree.restore(id);
            d.project.touch();
            ui.dirty = true;
            menu.close();
        }
        if menu.button(loc.get("ctx_delete_forever")).clicked() {
            ui.delete_confirm = Some(id);
            menu.close();
        }
        return;
    }
    ui.selection = Some(id);
    if menu.button(loc.get("ctx_new_scene")).clicked() {
        create_node(d, ui, loc, NodeKind::Scene, loc.get("scene_label"));
        menu.close();
    }
    if menu.button(loc.get("ctx_new_folder")).clicked() {
        create_node(d, ui, loc, NodeKind::Folder, loc.get("chapter_label"));
        menu.close();
    }
    if menu.button(loc.get("ctx_new_note")).clicked() {
        create_node(d, ui, loc, NodeKind::Note, loc.get("kind_note"));
        menu.close();
    }
    if matches!(
        kind,
        Ok(NodeKind::ResearchFolder) | Ok(NodeKind::ResearchItem)
    ) && menu.button(loc.get("ctx_new_research")).clicked()
    {
        create_node(
            d,
            ui,
            loc,
            NodeKind::ResearchItem,
            loc.get("kind_research_item"),
        );
        menu.close();
    }
    if menu.button(loc.get("ctx_new_character")).clicked() {
        create_node(
            d,
            ui,
            loc,
            NodeKind::CharacterSheet,
            loc.get("kind_character"),
        );
        menu.close();
    }
    if menu.button(loc.get("ctx_new_location")).clicked() {
        create_node(
            d,
            ui,
            loc,
            NodeKind::LocationSheet,
            loc.get("kind_location"),
        );
        menu.close();
    }
    menu.separator();
    if menu.button(loc.get("ctx_rename")).clicked() {
        ui.renaming = Some(id);
        ui.rename_buf = d
            .project
            .tree
            .get(id)
            .map(|n| n.title.clone())
            .unwrap_or_default();
        menu.close();
    }
    if menu.button(loc.get("ctx_move_up")).clicked() {
        let _ = d.project.tree.reorder(id, true);
        d.project.touch();
        ui.dirty = true;
        menu.close();
    }
    if menu.button(loc.get("ctx_move_down")).clicked() {
        let _ = d.project.tree.reorder(id, false);
        d.project.touch();
        ui.dirty = true;
        menu.close();
    }
    if menu.button(loc.get("ctx_indent")).clicked() {
        let _ = d.project.tree.indent(id);
        d.project.touch();
        ui.dirty = true;
        menu.close();
    }
    if menu.button(loc.get("ctx_outdent")).clicked() {
        let _ = d.project.tree.outdent(id);
        d.project.touch();
        ui.dirty = true;
        menu.close();
    }
    menu.separator();
    if menu.button(loc.get("ctx_trash")).clicked() {
        if d.project.tree.move_to_trash(id).is_ok() {
            d.project.touch();
            ui.dirty = true;
        }
        menu.close();
    }
}

// ---------------------------------------------------------------------------
// Scheda (inspector)
// ---------------------------------------------------------------------------

fn inspector_panel(
    doc: &mut Option<DocState>,
    ui: &mut UiState,
    loc: &Locale,
    panel: &mut egui::Ui,
) {
    let Some(d) = doc.as_mut() else { return };
    panel.heading(loc.get("inspector"));
    let Some(sel) = ui.selection else {
        panel.label(loc.get("cork_empty"));
        return;
    };
    let Ok(node) = d.project.tree.get(sel).cloned() else {
        return;
    };
    panel.separator();
    // Titolo.
    panel.label(loc.get("insp_title"));
    let mut title = node.title.clone();
    if panel.text_edit_singleline(&mut title).changed() && !title.trim().is_empty() {
        let _ = d.project.tree.rename(sel, title.trim());
        d.project.touch();
        ui.dirty = true;
    }
    panel.label(format!(
        "{}: {}",
        loc.get("insp_kind"),
        kind_name(loc, node.kind)
    ));
    // Stato.
    panel.horizontal(|row| {
        row.label(loc.get("insp_status"));
        let current = node.meta.status;
        egui::ComboBox::from_id_salt(("status", sel))
            .selected_text(status_name(loc, current))
            .show_ui(row, |combo| {
                for st in [
                    DocStatus::Todo,
                    DocStatus::Draft,
                    DocStatus::Revised,
                    DocStatus::Final,
                ] {
                    let mut probe = current;
                    if combo
                        .selectable_value(&mut probe, st, status_name(loc, st))
                        .clicked()
                        && probe != current
                    {
                        if let Ok(n) = d.project.tree.get_mut(sel) {
                            n.meta.status = st;
                        }
                        d.project.touch();
                        ui.dirty = true;
                    }
                }
            });
    });
    // Etichetta.
    panel.label(loc.get("insp_label"));
    let mut label = node.meta.label.clone();
    if panel.text_edit_singleline(&mut label).changed()
        && let Ok(n) = d.project.tree.get_mut(sel)
    {
        n.meta.label = label;
        d.project.touch();
        ui.dirty = true;
    }
    // Sinossi.
    panel.label(loc.get("insp_synopsis"));
    let mut syn = node.meta.synopsis.clone();
    if panel.text_edit_multiline(&mut syn).changed()
        && let Ok(n) = d.project.tree.get_mut(sel)
    {
        n.meta.synopsis = syn;
        d.project.touch();
        ui.dirty = true;
    }
    // Keyword.
    panel.label(loc.get("insp_keywords"));
    let mut kw = node.meta.keywords.join(", ");
    if panel.text_edit_singleline(&mut kw).changed() {
        let kws: Vec<String> = kw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        if let Ok(n) = d.project.tree.get_mut(sel) {
            // Registra anche globalmente le nuove keyword.
            for k in &kws {
                if !d.project.meta.keywords.contains(k) {
                    d.project.meta.keywords.push(k.clone());
                }
            }
            n.meta.keywords = kws;
            d.project.touch();
            ui.dirty = true;
        }
    }
    // Includi nell'export.
    let mut incl = node.meta.include_in_export;
    if panel.checkbox(&mut incl, loc.get("insp_include")).changed()
        && let Ok(n) = d.project.tree.get_mut(sel)
    {
        n.meta.include_in_export = incl;
        d.project.touch();
        ui.dirty = true;
    }
    // Parole del documento.
    if node.kind.can_have_text() {
        let words = count_words(d.project.doc_text(sel)).words;
        panel.label(format!("{}: {words}", loc.get("insp_words")));
    }
    // Schede personaggio/luogo.
    match node.kind {
        NodeKind::CharacterSheet => sheet_character_editor(d, ui, loc, panel, sel),
        NodeKind::LocationSheet => sheet_location_editor(d, ui, loc, panel, sel),
        _ => {}
    }
    // Campi personalizzati (modalità esperto).
    if ui.expert {
        panel.separator();
        panel.collapsing(loc.get("insp_custom"), |col| {
            let customs: Vec<(String, String)> = node
                .meta
                .custom
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (name, value) in customs {
                col.horizontal(|row| {
                    row.label(&name);
                    let mut v = value.clone();
                    if row.text_edit_singleline(&mut v).changed()
                        && let Ok(n) = d.project.tree.get_mut(sel)
                    {
                        n.meta.custom.insert(name.clone(), v);
                        d.project.touch();
                        ui.dirty = true;
                    }
                    if row.small_button("\u{2715}").clicked()
                        && let Ok(n) = d.project.tree.get_mut(sel)
                    {
                        n.meta.custom.remove(&name);
                        d.project.touch();
                        ui.dirty = true;
                    }
                });
            }
            col.horizontal(|row| {
                row.text_edit_singleline(&mut ui.custom_name);
                row.text_edit_singleline(&mut ui.custom_value);
                if row.small_button("+").clicked()
                    && !ui.custom_name.trim().is_empty()
                    && let Ok(n) = d.project.tree.get_mut(sel)
                {
                    n.meta
                        .custom
                        .insert(ui.custom_name.trim().to_string(), ui.custom_value.clone());
                    ui.custom_name.clear();
                    ui.custom_value.clear();
                    d.project.touch();
                    ui.dirty = true;
                }
            });
        });
    }
    // Snapshot.
    if node.kind.can_have_text() {
        panel.separator();
        panel.heading(loc.get("insp_snapshots"));
        panel.horizontal(|row| {
            row.text_edit_singleline(&mut ui.snapshot_label);
            if row.button(loc.get("insp_snapshot_new")).clicked() {
                let label = if ui.snapshot_label.trim().is_empty() {
                    plotvale_model::now_stamp()
                } else {
                    ui.snapshot_label.trim().to_string()
                };
                ui.snapshot_label.clear();
                let id = ulid::Ulid::new().to_string();
                d.project.snapshots.push(plotvale_model::SnapshotMeta {
                    id: id.clone(),
                    doc_id: sel,
                    created_at: plotvale_model::now_stamp(),
                    label,
                });
                d.project
                    .snapshot_texts
                    .insert(id, d.project.doc_text(sel).to_string());
                d.project.touch();
                ui.dirty = true;
            }
        });
        let snaps: Vec<(usize, plotvale_model::SnapshotMeta)> = d
            .project
            .snapshots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.doc_id == sel)
            .map(|(i, s)| (i, s.clone()))
            .collect();
        for (idx, snap) in snaps {
            panel.horizontal(|row| {
                row.small(format!("{} {}", snap.created_at, snap.label));
                if row.small_button(loc.get("snapshot_show")).clicked() {
                    ui.show_snapshot = Some((sel, idx));
                }
                if row.small_button(loc.get("insp_snapshot_restore")).clicked()
                    && let Some(text) = d.project.snapshot_texts.get(&snap.id).cloned()
                {
                    d.project.set_doc_text(sel, &text);
                    ui.views.insert(sel, DocView { text });
                    d.project.touch();
                    ui.dirty = true;
                }
                if row.small_button(loc.get("insp_snapshot_delete")).clicked() {
                    d.project.snapshot_texts.remove(&snap.id);
                    d.project.snapshots.remove(idx);
                    d.project.touch();
                    ui.dirty = true;
                }
            });
        }
    }
}

fn status_name(loc: &Locale, st: DocStatus) -> &str {
    loc.get(match st {
        DocStatus::Todo => "status_todo",
        DocStatus::Draft => "status_draft",
        DocStatus::Revised => "status_revised",
        DocStatus::Final => "status_final",
    })
}

fn kind_name(loc: &Locale, kind: NodeKind) -> &str {
    loc.get(match kind {
        NodeKind::DraftRoot => "kind_draft_root",
        NodeKind::Folder => "kind_folder",
        NodeKind::Scene => "kind_scene",
        NodeKind::Note => "kind_note",
        NodeKind::ResearchFolder => "kind_research_folder",
        NodeKind::ResearchItem => "kind_research_item",
        NodeKind::CharacterSheet => "kind_character",
        NodeKind::LocationSheet => "kind_location",
        NodeKind::Trash => "kind_trash",
    })
}

fn sheet_character_editor(
    d: &mut DocState,
    ui: &mut UiState,
    loc: &Locale,
    panel: &mut egui::Ui,
    sel: NodeId,
) {
    let Some(idx) = d
        .project
        .sheets
        .characters
        .iter()
        .position(|s| s.node_id == sel)
    else {
        return;
    };
    panel.separator();
    panel.label(loc.get("sheet_name"));
    let mut name = d.project.sheets.characters[idx].name.clone();
    if panel.text_edit_singleline(&mut name).changed() {
        d.project.sheets.characters[idx].name = name.clone();
        let _ = d.project.tree.rename(sel, name.trim());
        d.project.touch();
        ui.dirty = true;
    }
    for (key, label_key) in [
        ("role", "sheet_role"),
        ("description", "sheet_description"),
        ("goal", "sheet_goal"),
        ("conflict", "sheet_conflict"),
    ] {
        panel.label(loc.get(label_key));
        let current = match key {
            "role" => d.project.sheets.characters[idx].role.clone(),
            "description" => d.project.sheets.characters[idx].description.clone(),
            "goal" => d.project.sheets.characters[idx].goal.clone(),
            _ => d.project.sheets.characters[idx].conflict.clone(),
        };
        let mut value = current;
        if panel.text_edit_multiline(&mut value).changed() {
            let sheet = &mut d.project.sheets.characters[idx];
            match key {
                "role" => sheet.role = value,
                "description" => sheet.description = value,
                "goal" => sheet.goal = value,
                _ => sheet.conflict = value,
            }
            d.project.touch();
            ui.dirty = true;
        }
    }
    panel.label(loc.get("sheet_q"));
    for q in SHEET_QUESTIONS {
        panel.small(*q);
        let mut ans = d.project.sheets.characters[idx]
            .questionnaire
            .get(*q)
            .cloned()
            .unwrap_or_default();
        if panel.text_edit_multiline(&mut ans).changed() {
            let sheet = &mut d.project.sheets.characters[idx];
            if ans.trim().is_empty() {
                sheet.questionnaire.remove(*q);
            } else {
                sheet.questionnaire.insert((*q).to_string(), ans);
            }
            d.project.touch();
            ui.dirty = true;
        }
    }
}

fn sheet_location_editor(
    d: &mut DocState,
    ui: &mut UiState,
    loc: &Locale,
    panel: &mut egui::Ui,
    sel: NodeId,
) {
    let Some(idx) = d
        .project
        .sheets
        .locations
        .iter()
        .position(|s| s.node_id == sel)
    else {
        return;
    };
    panel.separator();
    panel.label(loc.get("sheet_name"));
    let mut name = d.project.sheets.locations[idx].name.clone();
    if panel.text_edit_singleline(&mut name).changed() {
        d.project.sheets.locations[idx].name = name.clone();
        let _ = d.project.tree.rename(sel, name.trim());
        d.project.touch();
        ui.dirty = true;
    }
    for (key, label_key) in [
        ("description", "sheet_description"),
        ("sensory", "sheet_sensory"),
        ("notes", "sheet_notes"),
    ] {
        panel.label(loc.get(label_key));
        let current = match key {
            "description" => d.project.sheets.locations[idx].description.clone(),
            "sensory" => d.project.sheets.locations[idx].sensory.clone(),
            _ => d.project.sheets.locations[idx].notes.clone(),
        };
        let mut value = current;
        if panel.text_edit_multiline(&mut value).changed() {
            let sheet = &mut d.project.sheets.locations[idx];
            match key {
                "description" => sheet.description = value,
                "sensory" => sheet.sensory = value,
                _ => sheet.notes = value,
            }
            d.project.touch();
            ui.dirty = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Editor (singolo + scrivenings)
// ---------------------------------------------------------------------------

fn view_editor(doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale, content: &mut egui::Ui) {
    let Some(d) = doc.as_mut() else {
        content.label(loc.get("status_no_project"));
        return;
    };
    // Modalità scrivenings: se la selezione è un contenitore, mostra tutte le
    // scene dentro in un unico flusso continuo (ognuna resta modificabile).
    let scriv_ids: Option<Vec<NodeId>> = if ui.scrivenings {
        ui.selection.and_then(|sel| {
            d.project.tree.get(sel).ok().and_then(|n| {
                if matches!(
                    n.kind,
                    NodeKind::DraftRoot | NodeKind::Folder | NodeKind::ResearchFolder
                ) {
                    Some(
                        d.project
                            .tree
                            .subtree_ids(sel)
                            .into_iter()
                            .filter(|id| {
                                d.project.tree.get(*id).is_ok_and(|m| {
                                    m.kind.can_have_text() && !d.project.tree.is_in_trash(*id)
                                })
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            })
        })
    } else {
        None
    };
    if let Some(ids) = scriv_ids {
        if ids.is_empty() {
            content.label(loc.get("cork_empty"));
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(content, |scroll| {
                for id in ids {
                    let title = d
                        .project
                        .tree
                        .get(id)
                        .map(|n| n.title.clone())
                        .unwrap_or_default();
                    if scroll.button(format!("\u{25be} {title}")).clicked() {
                        ui.selection = Some(id);
                        ui.open_doc = Some(id);
                    }
                    doc_editor(d, ui, scroll, id);
                    scroll.separator();
                }
            });
        return;
    }
    let Some(open) = ui.open_doc else {
        content.label(loc.get("editor_hint"));
        return;
    };
    if d.project.tree.get(open).is_err() {
        ui.open_doc = None;
        return;
    }
    let title = d
        .project
        .tree
        .get(open)
        .map(|n| n.title.clone())
        .unwrap_or_default();
    content.horizontal(|row| {
        row.heading(&title);
        let words = count_words(d.project.doc_text(open)).words;
        row.small(format!("{} {}", words, loc.get("status_words")));
    });
    content.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(content, |scroll| {
            doc_editor(d, ui, scroll, open);
        });
}

/// Un editor di testo legato a un documento.
fn doc_editor(d: &mut DocState, ui: &mut UiState, egui_ui: &mut egui::Ui, id: NodeId) {
    ensure_view(d, ui, id);
    let changed = {
        let view = ui.views.get_mut(&id).expect("vista appena creata");
        egui_ui
            .add(
                egui::TextEdit::multiline(&mut view.text)
                    .id(egui::Id::new(format!("sc-editor-{id}")))
                    .font(egui::FontId::proportional(17.0))
                    .desired_width(f32::INFINITY)
                    .layouter(&mut |ui, buf, wrap| {
                        markdown::layout_markdown(ui, buf.as_str(), wrap)
                    }),
            )
            .changed()
    };
    if changed {
        let text = ui
            .views
            .get(&id)
            .map(|v| v.text.clone())
            .unwrap_or_default();
        d.project.set_doc_text(id, &text);
        ui.dirty = true;
        ui.last_edit = Some(Instant::now());
    }
}

// ---------------------------------------------------------------------------
// Cartellone
// ---------------------------------------------------------------------------

/// Il genitore mostrato dal cartellone: la selezione se contenitore, altrimenti suo padre.
fn corkboard_parent(d: &DocState, ui: &UiState) -> NodeId {
    ui.selection
        .and_then(|sel| {
            d.project.tree.get(sel).ok().map(|n| {
                if matches!(
                    n.kind,
                    NodeKind::DraftRoot | NodeKind::Folder | NodeKind::ResearchFolder
                ) {
                    sel
                } else {
                    n.parent.unwrap_or(d.project.tree.draft_root)
                }
            })
        })
        .unwrap_or(d.project.tree.draft_root)
}

fn view_corkboard(
    doc: &mut Option<DocState>,
    ui: &mut UiState,
    loc: &Locale,
    content: &mut egui::Ui,
) {
    let Some(d) = doc.as_mut() else {
        content.label(loc.get("status_no_project"));
        return;
    };
    let parent = corkboard_parent(d, ui);
    let title = d
        .project
        .tree
        .get(parent)
        .map(|n| n.title.clone())
        .unwrap_or_default();
    content.heading(format!("{}: {title}", loc.get("tab_corkboard")));
    content.separator();
    let cards: Vec<NodeId> = d
        .project
        .tree
        .children_of(parent)
        .unwrap_or_default()
        .into_iter()
        .filter(|id| {
            d.project.tree.get(*id).is_ok_and(|n| {
                matches!(n.kind, NodeKind::Scene | NodeKind::Folder | NodeKind::Note)
            })
        })
        .collect();
    if cards.is_empty() {
        content.label(loc.get("cork_empty"));
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(content, |scroll| {
            for id in cards {
                let (title, syn, words, status) = d
                    .project
                    .tree
                    .get(id)
                    .map(|n| {
                        (
                            n.title.clone(),
                            n.meta.synopsis.clone(),
                            count_words(d.project.doc_text(id)).words,
                            n.meta.status,
                        )
                    })
                    .unwrap_or_default();
                egui::Frame::group(scroll.style()).show(scroll, |card| {
                    card.horizontal(|row| {
                        let (dot, _) = status_dot(status);
                        row.heading(format!("{dot} {title}"));
                    });
                    if syn.trim().is_empty() {
                        card.small(loc.get("cork_no_synopsis"));
                    } else {
                        card.label(&syn);
                    }
                    card.horizontal(|row| {
                        row.small(format!("{} {}", words, loc.get("cork_words")));
                        if row.small_button(loc.get("cork_open")).clicked() {
                            ui.selection = Some(id);
                            if d.project.tree.get(id).is_ok_and(|n| n.kind.can_have_text()) {
                                ensure_view(d, ui, id);
                                ui.open_doc = Some(id);
                            }
                        }
                        if row.small_button("\u{25b2}").clicked() {
                            let _ = d.project.tree.reorder(id, true);
                            d.project.touch();
                            ui.dirty = true;
                        }
                        if row.small_button("\u{25bc}").clicked() {
                            let _ = d.project.tree.reorder(id, false);
                            d.project.touch();
                            ui.dirty = true;
                        }
                    });
                });
                scroll.add_space(8.0);
            }
        });
}

// ---------------------------------------------------------------------------
// Ricerca e benvenuto
// ---------------------------------------------------------------------------

fn run_search(doc: &DocState, ui: &mut UiState) {
    let index = Index::build(&doc.project);
    match index.search(
        ui.search_query.trim(),
        SearchOptions {
            case_sensitive: ui.search_case,
            regex: ui.search_regex,
        },
    ) {
        Ok(hits) => {
            ui.search_results = hits;
            ui.search_error = None;
        }
        Err(e) => {
            ui.search_results.clear();
            ui.search_error = Some(e.to_string());
        }
    }
    ui.search_done = true;
}

fn view_search(doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale, content: &mut egui::Ui) {
    let Some(d) = doc.as_mut() else {
        content.label(loc.get("status_no_project"));
        return;
    };
    content.heading(loc.get("tab_search"));
    content.horizontal(|row| {
        let resp = row.text_edit_singleline(&mut ui.search_query);
        let go = row.button(loc.get("search_button")).clicked()
            || (resp.lost_focus() && row.input(|i| i.key_pressed(egui::Key::Enter)));
        if go {
            run_search(d, ui);
        }
    });
    content.horizontal(|row| {
        row.checkbox(&mut ui.search_regex, loc.get("search_regex"));
        row.checkbox(&mut ui.search_case, loc.get("search_case"));
    });
    content.separator();
    if let Some(err) = ui.search_error.clone() {
        content.colored_label(egui::Color32::RED, err);
    } else if ui.search_done {
        if ui.search_results.is_empty() {
            content.label(loc.get("search_none"));
        } else {
            content.small(format!(
                "{} {}",
                ui.search_results.len(),
                loc.get("search_results")
            ));
            let hits: Vec<Hit> = ui.search_results.iter().take(200).cloned().collect();
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(content, |scroll| {
                    for hit in hits {
                        let title = d
                            .project
                            .tree
                            .get(hit.doc_id)
                            .map(|n| n.title.clone())
                            .unwrap_or_default();
                        if scroll
                            .button(format!(
                                "{} \u{00b7} {} {}",
                                title,
                                loc.get("scene_label"),
                                hit.line_no
                            ))
                            .clicked()
                        {
                            ui.selection = Some(hit.doc_id);
                            ensure_view(d, ui, hit.doc_id);
                            ui.open_doc = Some(hit.doc_id);
                        }
                        scroll.small(&hit.excerpt);
                        scroll.separator();
                    }
                });
        }
    }
    // Collezioni: ricerche salvate nel progetto.
    content.separator();
    content.heading(loc.get("search_collections"));
    let defs: Vec<CollectionDef> = d.project.meta.collections.clone();
    for def in defs {
        content.horizontal(|row| {
            row.label(&def.name);
            if row.small_button(loc.get("search_button")).clicked() {
                ui.search_query = def.pattern.clone();
                ui.search_regex = def.regex;
                ui.search_case = def.case_sensitive;
                run_search(d, ui);
            }
            if row.small_button(loc.get("search_delete")).clicked() {
                d.project.meta.collections.retain(|c| c.name != def.name);
                d.project.touch();
                ui.dirty = true;
            }
        });
    }
    content.horizontal(|row| {
        row.label(loc.get("search_collection_name"));
        row.text_edit_singleline(&mut ui.new_query_name);
        if row.button(loc.get("search_save_current")).clicked()
            && !ui.new_query_name.trim().is_empty()
            && !ui.search_query.trim().is_empty()
        {
            d.project.meta.collections.push(CollectionDef {
                name: ui.new_query_name.trim().to_string(),
                pattern: ui.search_query.trim().to_string(),
                regex: ui.search_regex,
                case_sensitive: ui.search_case,
            });
            ui.new_query_name.clear();
            d.project.touch();
            ui.dirty = true;
        }
    });
}

fn view_welcome(
    doc: &mut Option<DocState>,
    ui: &mut UiState,
    loc: &Locale,
    content: &mut egui::Ui,
) {
    content.vertical_centered(|center| {
        center.add_space(24.0);
        center.heading(loc.get("welcome_title"));
        center.label(loc.get("welcome_sub"));
        center.add_space(16.0);
        if let Some(d) = doc.as_ref() {
            center.label(format!(
                "{}: {}",
                loc.get("insp_title"),
                d.project.manifest.title
            ));
            center.small(format!(
                "{}: {} {}",
                loc.get("stats_total_words"),
                d.project.total_words(),
                loc.get("common_words")
            ));
            center.add_space(12.0);
        }
        if center.button(loc.get("welcome_new")).clicked() {
            fresh_doc(doc, ui, loc);
        }
        if center.button(loc.get("welcome_open_file")).clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_title(loc.get("open_title_file"))
                .add_filter(loc.get("filter_project"), &["plotvale"])
                .pick_file()
        {
            do_open(path, doc, ui, loc);
        }
        if center.button(loc.get("welcome_open_folder")).clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_title(loc.get("open_title_folder"))
                .pick_folder()
        {
            do_open(path, doc, ui, loc);
        }
    });
}

// ---------------------------------------------------------------------------
// Finestre di dialogo
// ---------------------------------------------------------------------------

fn dialogs(ctx: &egui::Context, doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    if ui.want_quit {
        ui.want_quit = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    // Modifiche non salvate.
    if let Some(action) = ui.pending {
        let mut close = false;
        let mut run = false;
        egui::Window::new(loc.get("dlg_unsaved_title"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |w| {
                w.label(loc.get("dlg_unsaved_msg"));
                w.horizontal(|row| {
                    if row.button(loc.get("dlg_save")).clicked() {
                        // Procedi solo se il salvataggio è andato a buon fine.
                        run = if let Some(d) = doc.as_mut() {
                            if d.path.is_none() {
                                save_as_dialog(d, ui, loc);
                            } else {
                                do_save(d, ui, loc);
                            }
                            !ui.dirty
                        } else {
                            true
                        };
                        close = true;
                    }
                    if row.button(loc.get("dlg_discard")).clicked() {
                        run = true;
                        close = true;
                    }
                    if row.button(loc.get("dlg_cancel")).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            ui.pending = None;
        }
        if run {
            run_pending(doc, ui, loc, action);
        }
    }
    // Conferma eliminazione definitiva.
    if let Some(id) = ui.delete_confirm {
        let mut close = false;
        egui::Window::new(loc.get("dlg_delete_title"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |w| {
                w.label(loc.get("dlg_delete_msg"));
                w.horizontal(|row| {
                    if row.button(loc.get("dlg_delete_trash")).clicked() {
                        if let Some(d) = doc.as_mut() {
                            let _ = d.project.tree.move_to_trash(id);
                            d.project.touch();
                            ui.dirty = true;
                        }
                        close = true;
                    }
                    if row.button(loc.get("dlg_delete_forever_btn")).clicked() {
                        if let Some(d) = doc.as_mut()
                            && let Ok(removed) = d.project.tree.delete_permanently(id)
                        {
                            purge_subtree(d, ui, &removed);
                            d.project.touch();
                        }
                        close = true;
                    }
                    if row.button(loc.get("dlg_cancel")).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            ui.delete_confirm = None;
        }
    }
    // Svuota cestino.
    if ui.confirm_empty_trash {
        let mut close = false;
        egui::Window::new(loc.get("dlg_empty_trash_title"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |w| {
                w.label(loc.get("dlg_empty_trash_msg"));
                w.horizontal(|row| {
                    if row.button(loc.get("ctx_empty_trash")).clicked() {
                        if let Some(d) = doc.as_mut() {
                            let removed = d.project.tree.empty_trash();
                            purge_subtree(d, ui, &removed);
                            d.project.touch();
                            ui.dirty = true;
                        }
                        close = true;
                    }
                    if row.button(loc.get("dlg_cancel")).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            ui.confirm_empty_trash = false;
        }
    }
    if ui.show_goals {
        goals_window(ctx, doc, ui, loc);
    }
    if ui.show_stats {
        stats_window(ctx, doc, ui, loc);
    }
    if ui.show_export {
        export_window(ctx, doc, ui, loc);
    }
    if ui.show_about {
        let mut open = true;
        egui::Window::new(loc.get("about_title"))
            .open(&mut open)
            .show(ctx, |w| {
                w.label(format!("PlotVale {APP_VERSION}"));
                w.label(loc.get("about_text"));
            });
        ui.show_about = open;
    }
    if ui.show_manual {
        let mut open = true;
        egui::Window::new(loc.get("menu_manual"))
            .open(&mut open)
            .show(ctx, |w| {
                w.label(manual_text(loc));
            });
        ui.show_manual = open;
    }
    if ui.show_snapshot.is_some() {
        snapshot_window(ctx, doc, ui, loc);
    }
}

fn goals_window(ctx: &egui::Context, doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    let Some(d) = doc.as_mut() else {
        ui.show_goals = false;
        return;
    };
    let mut open = ui.show_goals;
    egui::Window::new(loc.get("goals_title"))
        .open(&mut open)
        .show(ctx, |w| {
            // Obiettivo progetto.
            let mut pg_on = d.project.targets.project_goal_words.is_some();
            if w.checkbox(&mut pg_on, loc.get("goals_project_enable"))
                .changed()
            {
                d.project.targets.project_goal_words = pg_on.then_some(50_000);
                d.project.touch();
                ui.dirty = true;
            }
            if let Some(goal) = d.project.targets.project_goal_words {
                let mut value = goal;
                w.horizontal(|row| {
                    row.label(loc.get("goals_project_words"));
                    if row
                        .add(
                            egui::DragValue::new(&mut value)
                                .range(100..=10_000_000)
                                .speed(100),
                        )
                        .changed()
                    {
                        d.project.targets.project_goal_words = Some(value);
                        d.project.touch();
                        ui.dirty = true;
                    }
                });
                let total = d.project.total_words() as f32;
                w.add(
                    egui::ProgressBar::new((total / goal as f32).clamp(0.0, 1.0)).show_percentage(),
                );
            }
            // Obiettivo giornaliero.
            let mut dg_on = d.project.targets.daily_goal_words.is_some();
            if w.checkbox(&mut dg_on, loc.get("goals_daily_enable"))
                .changed()
            {
                d.project.targets.daily_goal_words = dg_on.then_some(500);
                d.project.touch();
                ui.dirty = true;
            }
            if let Some(goal) = d.project.targets.daily_goal_words {
                let mut value = goal;
                w.horizontal(|row| {
                    row.label(loc.get("goals_daily_words"));
                    if row
                        .add(
                            egui::DragValue::new(&mut value)
                                .range(10..=100_000)
                                .speed(10),
                        )
                        .changed()
                    {
                        d.project.targets.daily_goal_words = Some(value);
                        d.project.touch();
                        ui.dirty = true;
                    }
                });
                let today = d.project.targets.words_today;
                w.label(format!("{}: {today}", loc.get("goals_today")));
                w.add(
                    egui::ProgressBar::new((today as f32 / goal as f32).clamp(0.0, 1.0))
                        .show_percentage(),
                );
            }
            if d.project.targets.project_goal_words.is_none()
                && d.project.targets.daily_goal_words.is_none()
            {
                w.small(loc.get("goals_no_goal"));
            }
        });
    ui.show_goals = open;
}

fn stats_window(ctx: &egui::Context, doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    let Some(d) = doc.as_ref() else {
        ui.show_stats = false;
        return;
    };
    let mut open = ui.show_stats;
    egui::Window::new(loc.get("stats_title"))
        .open(&mut open)
        .show(ctx, |w| {
            let mut docs = 0;
            let counts = d
                .project
                .tree
                .ids()
                .filter(|id| !d.project.tree.is_in_trash(*id))
                .map(|id| {
                    if !d.project.doc_text(id).trim().is_empty() {
                        docs += 1;
                    }
                    count_words(d.project.doc_text(id))
                })
                .sum::<plotvale_model::WordCounts>();
            w.label(format!(
                "{}: {}",
                loc.get("stats_total_words"),
                counts.words
            ));
            w.label(format!("{}: {}", loc.get("stats_docs"), docs));
            w.label(format!(
                "{}: {}",
                loc.get("stats_chars"),
                counts.characters_no_spaces
            ));
            w.label(format!("{}: {}", loc.get("stats_paras"), counts.paragraphs));
            w.separator();
            w.label(loc.get("stats_by_chapter"));
            for child in d
                .project
                .tree
                .children_of(d.project.tree.draft_root)
                .unwrap_or_default()
            {
                let name = d
                    .project
                    .tree
                    .get(child)
                    .map(|n| n.title.clone())
                    .unwrap_or_default();
                w.small(format!("{}: {}", name, d.project.subtree_words(child)));
            }
        });
    ui.show_stats = open;
}

fn export_window(ctx: &egui::Context, doc: &mut Option<DocState>, ui: &mut UiState, loc: &Locale) {
    let Some(d) = doc.as_mut() else {
        ui.show_export = false;
        return;
    };
    let mut open = ui.show_export;
    egui::Window::new(loc.get("export_title"))
        .open(&mut open)
        .show(ctx, |w| {
            // Profilo.
            let names: Vec<String> = d.profiles.profiles.iter().map(|p| p.name.clone()).collect();
            egui::ComboBox::from_label(loc.get("export_preset"))
                .selected_text(&ui.export_preset)
                .show_ui(w, |combo| {
                    for name in &names {
                        if combo
                            .selectable_value(&mut ui.export_preset, name.clone(), name)
                            .clicked()
                        {
                            // Allinea il flag annotazioni al profilo scelto.
                            if let Some(p) = d.profiles.profiles.iter().find(|p| &p.name == name) {
                                ui.export_notes = p.keep_notes;
                            }
                        }
                    }
                });
            // Allinea una tantum all'apertura.
            w.checkbox(&mut ui.export_notes, loc.get("export_keep_notes"));
            w.separator();
            for format in ExportFormat::all() {
                if w.button(format.display_name()).clicked() {
                    do_export(d, ui, loc, *format);
                }
            }
        });
    ui.show_export = open;
}

fn do_export(d: &mut DocState, ui: &mut UiState, loc: &Locale, format: ExportFormat) {
    let mut profile: CompileProfile = d.profiles.active_profile();
    if let Some(p) = d
        .profiles
        .profiles
        .iter()
        .find(|p| p.name == ui.export_preset)
    {
        profile = p.clone();
    }
    profile.keep_notes = ui.export_notes;
    let selection = ui.selection.map(|s| vec![s]).unwrap_or_default();
    let ids = expand_selection(&d.project, &selection);
    let compiled = match compile(&d.project, &ids, &profile) {
        Ok(c) => c,
        Err(e) => {
            ui.status = format!("{}: {e}", loc.get("err_export"));
            return;
        }
    };
    let file = rfd::FileDialog::new()
        .set_title(loc.get("export_title_dialog"))
        .add_filter(format.display_name(), &[format.extension()])
        .set_file_name(format!(
            "{}.{}",
            d.project.manifest.title,
            format.extension()
        ))
        .save_file();
    if let Some(path) = file {
        match export_to_file(&compiled, format, &path) {
            Ok(()) => ui.status = loc.get("export_done").to_string(),
            Err(e) => ui.status = format!("{}: {e}", loc.get("err_export")),
        }
    }
}

fn snapshot_window(
    ctx: &egui::Context,
    doc: &mut Option<DocState>,
    ui: &mut UiState,
    loc: &Locale,
) {
    let Some((doc_id, idx)) = ui.show_snapshot else {
        return;
    };
    let Some(d) = doc.as_mut() else {
        ui.show_snapshot = None;
        return;
    };
    let (meta, text) = match d.project.snapshots.get(idx) {
        Some(m) if m.doc_id == doc_id => {
            let t = d
                .project
                .snapshot_texts
                .get(&m.id)
                .cloned()
                .unwrap_or_default();
            (m.clone(), t)
        }
        _ => {
            ui.show_snapshot = None;
            return;
        }
    };
    let mut close = false;
    let mut restore = false;
    egui::Window::new(format!("{} — {}", loc.get("insp_snapshots"), meta.label))
        .default_size([560.0, 420.0])
        .show(ctx, |w| {
            w.small(&meta.created_at);
            let current = d.project.doc_text(doc_id).to_string();
            let lines = diff::diff_lines(&text, &current);
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(w, |scroll| {
                    for line in lines.iter().take(600) {
                        match line {
                            diff::DiffLine::Same(t) => {
                                scroll.small(t);
                            }
                            diff::DiffLine::Del(t) => {
                                scroll.colored_label(egui::Color32::RED, format!("\u{2212} {t}"));
                            }
                            diff::DiffLine::Add(t) => {
                                scroll.colored_label(egui::Color32::GREEN, format!("+ {t}"));
                            }
                        }
                    }
                    if lines.len() > 600 {
                        scroll.small("…");
                    }
                });
            w.horizontal(|row| {
                if row.button(loc.get("insp_snapshot_restore")).clicked() {
                    restore = true;
                    close = true;
                }
                if row.button(loc.get("common_close")).clicked() {
                    close = true;
                }
            });
        });
    if restore {
        d.project.set_doc_text(doc_id, &text);
        ui.views.insert(doc_id, DocView { text });
        d.project.touch();
        ui.dirty = true;
    }
    if close {
        ui.show_snapshot = None;
    }
}

fn manual_text(loc: &Locale) -> &'static str {
    if loc.code == "en" {
        "Quick guide\n\n1. Create scenes from the + menu on the left.\n\
         2. Click a scene and write in the center. Markdown works: **bold**, *italic*, # Title.\n\
         3. Right-click an item to move, rename or trash it.\n\
         4. Everything autosaves; File > Export produces DOCX, EPUB and more.\n\
         5. [[note: …]] annotations never leave the app."
    } else {
        "Guida rapida\n\n1. Crea scene dal menu + a sinistra.\n\
         2. Clicca una scena e scrivi al centro. Markdown: **grassetto**, *corsivo*, # Titolo.\n\
         3. Tasto destro per spostare, rinominare, cestinare.\n\
         4. Tutto si salva da solo; File > Esporta produce DOCX, EPUB e altro.\n\
         5. Le annotazioni [[nota: …]] non escono mai dal programma."
    }
}
