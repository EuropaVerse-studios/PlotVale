//! Guided new-project wizard: template → details → structure → features → location.
//!
//! Modern apps do not drop users into a blank window: the wizard collects the
//! basics first, then builds the project in one go. Pure state + UI here; the
//! caller installs the finished project via [`build_project`].

use std::path::PathBuf;

use plotvale_model::{NodeKind, Project};

use crate::i18n::Locale;

/// Project template: preset defaults the user can still adjust.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Template {
    /// Multi-chapter novel with example content and a daily goal.
    #[default]
    Novel,
    /// Single-chapter short story.
    Story,
    /// Empty skeleton, no example text, no goals.
    Blank,
}

/// Wizard state, kept in [`crate::UiState`] while the dialog is open.
pub struct WizardState {
    pub step: usize,
    pub template: Template,
    pub title: String,
    pub author: String,
    pub language: String,
    pub chapters: usize,
    pub scenes_per_chapter: usize,
    pub with_example: bool,
    pub daily_goal: Option<u64>,
    pub project_goal: Option<u64>,
    pub expert: bool,
    pub folder_mode: bool,
    pub location: Option<PathBuf>,
}

impl WizardState {
    pub fn new() -> Self {
        let mut wiz = Self {
            step: 0,
            template: Template::Novel,
            title: String::new(),
            author: String::new(),
            language: "en".to_string(),
            chapters: 3,
            scenes_per_chapter: 2,
            with_example: true,
            daily_goal: Some(500),
            project_goal: None,
            expert: false,
            folder_mode: false,
            location: None,
        };
        wiz.apply_template();
        wiz
    }

    /// Applies the template presets (user tweaks afterwards are kept).
    pub fn apply_template(&mut self) {
        match self.template {
            Template::Novel => {
                self.chapters = 3;
                self.scenes_per_chapter = 2;
                self.with_example = true;
                self.daily_goal = Some(500);
            }
            Template::Story => {
                self.chapters = 1;
                self.scenes_per_chapter = 1;
                self.with_example = true;
                self.daily_goal = Some(500);
            }
            Template::Blank => {
                self.chapters = 1;
                self.scenes_per_chapter = 1;
                self.with_example = false;
                self.daily_goal = None;
            }
        }
    }

    /// Suggested file name derived from the title.
    pub fn suggested_name(&self, loc: &Locale) -> String {
        let base = self.title.trim();
        let base = if base.is_empty() {
            loc.get("untitled").to_string()
        } else {
            base.to_string()
        };
        // Keep it filesystem-friendly.
        let safe: String = base
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        safe.trim().to_string()
    }
}

impl Default for WizardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the project described by the wizard.
///
/// `version` is the app version for the manifest; labels come from the UI locale.
pub fn build_project(
    wiz: &WizardState,
    loc: &Locale,
    version: &str,
    chapter_label: &str,
    scene_label: &str,
) -> Project {
    let title = wiz.title.trim();
    let title = if title.is_empty() {
        loc.get("untitled").to_string()
    } else {
        title.to_string()
    };
    let mut project = Project::new(&title, wiz.author.trim(), version);
    project.manifest.language = wiz.language.clone();

    // Shape the default Chapter 1 / Scene 1, then grow the rest.
    let draft = project.tree.draft_root;
    let first_chapter = project
        .tree
        .children_of(draft)
        .unwrap_or_default()
        .into_iter()
        .next();
    let chapters = wiz.chapters.clamp(1, 30);
    let scenes = wiz.scenes_per_chapter.clamp(1, 20);

    // Rename the pre-made chapter/scene and set (or clear) the example text.
    let mut first_scene = None;
    if let Some(cap) = first_chapter {
        let _ = project.tree.rename(cap, &format!("{chapter_label} 1"));
        if let Some(scene) = project
            .tree
            .children_of(cap)
            .unwrap_or_default()
            .into_iter()
            .next()
        {
            let _ = project.tree.rename(scene, &format!("{scene_label} 1"));
            first_scene = Some(scene);
        }
    }
    if let Some(scene) = first_scene {
        if wiz.with_example {
            // Project::new already inserted the example text: keep it.
        } else {
            project.set_doc_text(scene, "");
        }
    }
    // Extra scenes in chapter 1.
    if let Some(cap) = first_chapter {
        for i in 2..=scenes {
            if let Ok(scene) =
                project
                    .tree
                    .create(cap, NodeKind::Scene, &format!("{scene_label} {i}"))
            {
                project.set_doc_text(scene, "");
            }
        }
    }
    // Extra chapters.
    for c in 2..=chapters {
        if let Ok(cap) =
            project
                .tree
                .create(draft, NodeKind::Folder, &format!("{chapter_label} {c}"))
        {
            for i in 1..=scenes {
                if let Ok(scene) =
                    project
                        .tree
                        .create(cap, NodeKind::Scene, &format!("{scene_label} {i}"))
                {
                    project.set_doc_text(scene, "");
                }
            }
        }
    }

    project.targets.daily_goal_words = wiz.daily_goal;
    project.targets.project_goal_words = wiz.project_goal;
    project.touch();
    project
}
