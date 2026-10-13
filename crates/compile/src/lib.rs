//! `plotvale-compile` — the composition engine.
//!
//! Turns an ordered node selection into [`Compiled`]: title, author, typed
//! sections, and layout hints. It is the **single source of truth** for all
//! exports: renderers never invent content.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use plotvale_model::{NodeId, Project};

/// Composition errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompileError {
    /// Requested node missing.
    #[error("item not found: {0}")]
    NotFound(NodeId),
}

// ---------------------------------------------------------------------------
// Formatted text
// ---------------------------------------------------------------------------

/// Text span with formatting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(String),
    /// **bold** (may nest italic).
    Bold(Vec<Inline>),
    /// *italic* (may nest bold).
    Italic(Vec<Inline>),
    /// `code`.
    Code(String),
    /// Note reference `[^id]`.
    FootnoteRef(String),
    /// Annotation `[[note: …]]` (excluded from export unless profile says so).
    Note(String),
}

impl Inline {
    /// Plain text without formatting (for statistics and previews).
    pub fn plain_text(items: &[Inline]) -> String {
        let mut out = String::new();
        for item in items {
            match item {
                Inline::Text(t) | Inline::Code(t) => out.push_str(t),
                Inline::Bold(inner) | Inline::Italic(inner) => {
                    out.push_str(&Self::plain_text(inner));
                }
                Inline::FootnoteRef(id) => {
                    out.push_str(&format!("[{id}]"));
                }
                Inline::Note(_) => {}
            }
        }
        out
    }
}

/// Parses the inline formatting of a Markdown line.
pub fn parse_inline(s: &str) -> Vec<Inline> {
    let chars: Vec<char> = s.chars().collect();
    let (items, _) = parse_inline_until(&chars, 0, None);
    items
}

/// Parses up to a terminator (`**`, `*`, …) or the end.
fn parse_inline_until(
    chars: &[char],
    mut i: usize,
    terminator: Option<&str>,
) -> (Vec<Inline>, usize) {
    let mut out = Vec::new();
    let mut buf = String::new();
    let flush = |out: &mut Vec<Inline>, buf: &mut String| {
        if !buf.is_empty() {
            out.push(Inline::Text(std::mem::take(buf)));
        }
    };
    while i < chars.len() {
        // Top-level terminator?
        if let Some(term) = terminator
            && matches_term(chars, i, term)
        {
            flush(&mut out, &mut buf);
            return (out, i + term.chars().count());
        }
        if matches_term(chars, i, "**") {
            // Look for the closing marker; if missing, it is literal text.
            let (inner, next) = parse_inline_until(chars, i + 2, Some("**"));
            if next > i + 2 && closed_before(chars, next, "**") {
                flush(&mut out, &mut buf);
                out.push(Inline::Bold(inner));
                i = next;
            } else {
                buf.push_str("**");
                i += 2;
            }
        } else if chars[i] == '*' {
            let (inner, next) = parse_inline_until(chars, i + 1, Some("*"));
            if next > i + 1 && closed_before(chars, next, "*") {
                flush(&mut out, &mut buf);
                out.push(Inline::Italic(inner));
                i = next;
            } else {
                buf.push('*');
                i += 1;
            }
        } else if chars[i] == '`' {
            if let Some(end) = find_char(chars, i + 1, '`') {
                flush(&mut out, &mut buf);
                out.push(Inline::Code(chars[i + 1..end].iter().collect()));
                i = end + 1;
            } else {
                buf.push('`');
                i += 1;
            }
        } else if matches_term(chars, i, "[[") {
            if let Some(end) = find_str(chars, i + 2, "]]") {
                flush(&mut out, &mut buf);
                let raw: String = chars[i + 2..end].iter().collect();
                let note = raw
                    .strip_prefix("nota:")
                    .or_else(|| raw.strip_prefix("note:"))
                    .unwrap_or(&raw)
                    .trim()
                    .to_string();
                out.push(Inline::Note(note));
                i = end + 2;
            } else {
                buf.push_str("[[");
                i += 2;
            }
        } else if matches_term(chars, i, "[^") {
            if let Some(end) = find_char(chars, i + 2, ']') {
                flush(&mut out, &mut buf);
                out.push(Inline::FootnoteRef(chars[i + 2..end].iter().collect()));
                i = end + 1;
            } else {
                buf.push_str("[^");
                i += 2;
            }
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    flush(&mut out, &mut buf);
    (out, i)
}

fn matches_term(chars: &[char], i: usize, term: &str) -> bool {
    let t: Vec<char> = term.chars().collect();
    chars.len() >= i + t.len() && chars[i..i + t.len()] == t[..]
}

fn closed_before(chars: &[char], next: usize, term: &str) -> bool {
    let t: Vec<char> = term.chars().collect();
    next >= t.len() && chars[next - t.len()..next] == t[..]
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    chars
        .iter()
        .skip(from)
        .position(|c| *c == target)
        .map(|p| p + from)
}

fn find_str(chars: &[char], from: usize, needle: &str) -> Option<usize> {
    let n: Vec<char> = needle.chars().collect();
    (from..=chars.len().saturating_sub(n.len())).find(|i| chars[*i..*i + n.len()] == n[..])
}

// ---------------------------------------------------------------------------
// Blocks and sections
// ---------------------------------------------------------------------------

/// Content block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// Level 1–6 heading.
    Heading {
        /// Level (1–6).
        level: u8,
        /// Formatted text.
        text: Vec<Inline>,
    },
    /// Paragraph.
    Para(Vec<Inline>),
    /// Quote (`> …`).
    Quote(Vec<Inline>),
    /// List item (`- …`).
    ListItem(Vec<Inline>),
    /// Scene separator (`---` in the text).
    SceneBreak,
    /// Page break (paginated formats only).
    PageBreak,
    /// Note definition `[^id]: text`.
    FootnoteDef {
        /// Note id.
        id: String,
        /// Formatted text.
        text: Vec<Inline>,
    },
}

/// Parses a document's Markdown text into blocks.
pub fn parse_blocks(text: &str, keep_notes: bool) -> Vec<Block> {
    let mut blocks = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" || trimmed == "***" {
            blocks.push(Block::SceneBreak);
        } else if let Some(rest) = trimmed.strip_prefix("###### ") {
            blocks.push(block_heading(6, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("##### ") {
            blocks.push(block_heading(5, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("#### ") {
            blocks.push(block_heading(4, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            blocks.push(block_heading(3, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            blocks.push(block_heading(2, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            blocks.push(block_heading(1, rest, keep_notes));
        } else if let Some(rest) = trimmed.strip_prefix("> ") {
            blocks.push(Block::Quote(filter_notes(parse_inline(rest), keep_notes)));
        } else if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            blocks.push(Block::ListItem(filter_notes(
                parse_inline(rest),
                keep_notes,
            )));
        } else if let Some((id, def)) = parse_footnote_def(trimmed) {
            blocks.push(Block::FootnoteDef {
                id,
                text: filter_notes(parse_inline(def), keep_notes),
            });
        } else {
            blocks.push(Block::Para(filter_notes(parse_inline(trimmed), keep_notes)));
        }
    }
    blocks
}

fn block_heading(level: u8, rest: &str, keep_notes: bool) -> Block {
    Block::Heading {
        level,
        text: filter_notes(parse_inline(rest), keep_notes),
    }
}

fn parse_footnote_def(line: &str) -> Option<(String, &str)> {
    let rest = line.strip_prefix("[^")?;
    let end = rest.find("]:")?;
    let id = rest[..end].to_string();
    let def = rest[end + 2..].trim();
    if id.is_empty() || def.is_empty() {
        return None;
    }
    Some((id, def))
}

/// Strips annotations from inlines unless the profile wants them.
fn filter_notes(items: Vec<Inline>, keep: bool) -> Vec<Inline> {
    if keep {
        return items;
    }
    items
        .into_iter()
        .filter_map(|item| match item {
            Inline::Note(_) => None,
            Inline::Bold(inner) => Some(Inline::Bold(filter_notes(inner, false))),
            Inline::Italic(inner) => Some(Inline::Italic(filter_notes(inner, false))),
            other => Some(other),
        })
        .collect()
}

/// Section kind of the compiled manuscript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    /// Title page and front matter.
    FrontMatter,
    /// Chapter or folder (has a shown title).
    Chapter,
    /// Scene (title shown only if the profile asks).
    Scene,
}

/// One section of the compiled manuscript.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// Section title.
    pub title: String,
    /// Depth level (1 = chapter).
    pub level: u8,
    /// Section kind.
    pub kind: SectionKind,
    /// Content blocks.
    pub blocks: Vec<Block>,
}

// ---------------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------------

/// Composition profile: how to assemble and export.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileProfile {
    /// Profile name.
    pub name: String,
    /// Shows scene titles (by default chapters only).
    #[serde(default)]
    pub show_scene_titles: bool,
    /// Separator between scenes (e.g. `***`).
    #[serde(default = "default_separator")]
    pub scene_separator: String,
    /// Page break before chapters (paginated formats).
    #[serde(default = "default_true")]
    pub page_break_before_chapters: bool,
    /// Keeps `[[note: …]]` annotations in the output.
    #[serde(default)]
    pub keep_notes: bool,
    /// Opening dedication or epigraph.
    #[serde(default)]
    pub dedication: String,
}

fn default_separator() -> String {
    "***".to_string()
}

fn default_true() -> bool {
    true
}

impl CompileProfile {
    /// Standard manuscript: titled chapters, scenes separated by `***`.
    pub fn manuscript() -> Self {
        Self {
            name: "manoscritto".to_string(),
            show_scene_titles: false,
            scene_separator: "***".to_string(),
            page_break_before_chapters: true,
            keep_notes: false,
            dedication: String::new(),
        }
    }

    /// For beta readers: visible scene titles, no annotations.
    pub fn beta_reader() -> Self {
        Self {
            name: "beta-reader".to_string(),
            show_scene_titles: true,
            ..Self::manuscript()
        }
    }

    /// Ebook: no forced page breaks between short chapters.
    pub fn ebook() -> Self {
        Self {
            name: "ebook".to_string(),
            page_break_before_chapters: true,
            ..Self::manuscript()
        }
    }

    /// Faithful: everything in the project, annotations included.
    pub fn faithful() -> Self {
        Self {
            name: "fedelta".to_string(),
            show_scene_titles: true,
            keep_notes: true,
            ..Self::manuscript()
        }
    }
}

/// Profiles saved in the project (`composizione/profili.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilesFile {
    /// Available profiles.
    pub profiles: Vec<CompileProfile>,
    /// Active profile name.
    pub active: String,
}

impl Default for ProfilesFile {
    fn default() -> Self {
        Self {
            profiles: vec![
                CompileProfile::manuscript(),
                CompileProfile::beta_reader(),
                CompileProfile::ebook(),
                CompileProfile::faithful(),
            ],
            active: "manoscritto".to_string(),
        }
    }
}

impl ProfilesFile {
    /// Active profile (or the first if the name no longer exists).
    pub fn active_profile(&self) -> CompileProfile {
        self.profiles
            .iter()
            .find(|p| p.name == self.active)
            .or_else(|| self.profiles.first())
            .cloned()
            .unwrap_or_else(CompileProfile::manuscript)
    }
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

/// Compiled manuscript, ready for renderers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compiled {
    /// Title of the work.
    pub title: String,
    /// Author.
    pub author: String,
    /// Sections in order.
    pub sections: Vec<Section>,
    /// Separator between scenes (from the profile).
    pub scene_separator: String,
    /// Page break before chapters (from the profile).
    pub page_break_before_chapters: bool,
}

/// Compiles the given documents (in order) under the profile.
///
/// Folders become titled chapters; scenes become text sections; excluded
/// documents (`include_in_export = false`), notes, and trash are skipped.
pub fn compile(
    project: &Project,
    doc_ids: &[NodeId],
    profile: &CompileProfile,
) -> Result<Compiled, CompileError> {
    let mut sections = Vec::new();
    if !profile.dedication.trim().is_empty() {
        sections.push(Section {
            title: String::new(),
            level: 0,
            kind: SectionKind::FrontMatter,
            blocks: vec![Block::Para(parse_inline(profile.dedication.trim()))],
        });
    }
    for id in doc_ids {
        let node = project
            .tree
            .get(*id)
            .map_err(|_| CompileError::NotFound(*id))?;
        if project.tree.is_in_trash(*id) || !node.meta.include_in_export {
            continue;
        }
        match node.kind {
            plotvale_model::NodeKind::Folder | plotvale_model::NodeKind::DraftRoot => {
                let mut blocks = Vec::new();
                if !node.meta.synopsis.trim().is_empty() {
                    blocks.push(Block::Quote(parse_inline(node.meta.synopsis.trim())));
                }
                sections.push(Section {
                    title: node.title.clone(),
                    level: 1,
                    kind: SectionKind::Chapter,
                    blocks,
                });
            }
            plotvale_model::NodeKind::Scene => {
                let text = project.doc_text(*id);
                let mut blocks = parse_blocks(text, profile.keep_notes);
                if profile.show_scene_titles {
                    blocks.insert(
                        0,
                        Block::Heading {
                            level: 2,
                            text: parse_inline(&node.title),
                        },
                    );
                }
                sections.push(Section {
                    title: node.title.clone(),
                    level: 2,
                    kind: SectionKind::Scene,
                    blocks,
                });
            }
            // Notes, research, and sheets stay out of the manuscript.
            _ => {}
        }
    }
    Ok(Compiled {
        title: project.manifest.title.clone(),
        author: project.manifest.author.clone(),
        sections,
        scene_separator: profile.scene_separator.clone(),
        page_break_before_chapters: profile.page_break_before_chapters,
    })
}

/// Expands a selection into an ordered id list, descending into folders
/// in tree order. If empty, uses the whole manuscript.
pub fn expand_selection(project: &Project, selection: &[NodeId]) -> Vec<NodeId> {
    let roots: Vec<NodeId> = if selection.is_empty() {
        vec![project.tree.draft_root]
    } else {
        selection.to_vec()
    };
    let mut out = Vec::new();
    for root in roots {
        if project.tree.is_in_trash(root) {
            continue;
        }
        for id in project.tree.subtree_ids(root) {
            if id == project.tree.draft_root {
                continue;
            }
            out.push(id);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotvale_model::Project;

    fn sample_project() -> Project {
        let mut project = Project::new("Romanzo", "Autrice", "0.1.0");
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        let scene = project.tree.children_of(cap).unwrap()[0];
        project.set_doc_text(
            scene,
            "# Prologo\n\nUn **inizio** e un *sospetto*.[^1]\n\n[[nota: rivedere]]\n\n[^1]: La prima nota.",
        );
        project
    }

    #[test]
    fn inline_parsing() {
        let items = parse_inline("Un **inizio *doppio*** e `code`[^1]");
        assert!(matches!(items[1], Inline::Bold(_)));
        if let Inline::Bold(inner) = &items[1] {
            assert!(matches!(inner[1], Inline::Italic(_)));
        } else {
            panic!("atteso grassetto");
        }
        assert!(matches!(items[3], Inline::Code(_)));
        assert!(matches!(items[4], Inline::FootnoteRef(_)));
    }

    #[test]
    fn notes_are_stripped_by_default() {
        let blocks = parse_blocks("Testo [[nota: segreta]] fine.", false);
        assert_eq!(
            Inline::plain_text(&[blocks_into_para(&blocks)]),
            "Testo  fine."
        );
    }

    fn blocks_into_para(blocks: &[Block]) -> Inline {
        match &blocks[0] {
            Block::Para(items) => Inline::Bold(items.clone()),
            other => panic!("atteso paragrafo, trovato {other:?}"),
        }
    }

    #[test]
    fn compile_manuscript() {
        let project = sample_project();
        let ids = expand_selection(&project, &[]);
        let compiled = compile(&project, &ids, &CompileProfile::manuscript()).unwrap();
        assert_eq!(compiled.title, "Romanzo");
        // Chapter + scene.
        assert_eq!(compiled.sections.len(), 2);
        assert_eq!(compiled.sections[0].kind, SectionKind::Chapter);
        assert_eq!(compiled.sections[1].kind, SectionKind::Scene);
        // The annotation is gone, the note stays.
        let text = format!("{:?}", compiled.sections[1].blocks);
        assert!(!text.contains("rivedere"));
        assert!(text.contains("FootnoteRef"));
    }

    #[test]
    fn excluded_docs_are_skipped() {
        let mut project = sample_project();
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        let scene = project.tree.children_of(cap).unwrap()[0];
        project.tree.get_mut(scene).unwrap().meta.include_in_export = false;
        let ids = expand_selection(&project, &[]);
        let compiled = compile(&project, &ids, &CompileProfile::manuscript()).unwrap();
        assert_eq!(compiled.sections.len(), 1);
    }
}
