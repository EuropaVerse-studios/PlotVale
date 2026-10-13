//! `plotvale-export` — export renderers.
//!
//! Every format renders the same [`Compiled`] produced by the composition
//! engine: content never changes between DOCX, EPUB, and the rest.

use std::fmt::Write as _;
use std::io::Cursor;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

use plotvale_compile::{Block, Compiled, Inline, SectionKind};

/// Export errors.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Output file write.
    #[error("file write: {0}")]
    Io(#[from] std::io::Error),
    /// DOCX generator error.
    #[error("DOCX generation: {0}")]
    Docx(String),
    /// EPUB generator error.
    #[error("EPUB generation: {0}")]
    Epub(String),
}

/// Supported export formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// Markdown.
    Md,
    /// Plain text.
    Txt,
    /// Standalone HTML with index.
    Html,
    /// Word (.docx).
    Docx,
    /// Ebook (.epub).
    Epub,
}

impl ExportFormat {
    /// File extension (no dot).
    pub fn extension(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Txt => "txt",
            Self::Html => "html",
            Self::Docx => "docx",
            Self::Epub => "epub",
        }
    }

    /// Name shown in menus.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Md => "Markdown (.md)",
            Self::Txt => "Testo (.txt)",
            Self::Html => "Pagina web (.html)",
            Self::Docx => "Word (.docx)",
            Self::Epub => "Ebook (.epub)",
        }
    }

    /// All formats, in menu order.
    pub fn all() -> &'static [Self] {
        &[Self::Md, Self::Txt, Self::Html, Self::Docx, Self::Epub]
    }
}

impl FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Md),
            "txt" | "text" => Ok(Self::Txt),
            "html" | "htm" => Ok(Self::Html),
            "docx" => Ok(Self::Docx),
            "epub" => Ok(Self::Epub),
            other => Err(format!("unknown format: {other}")),
        }
    }
}

/// Renders a compiled manuscript in the requested format.
pub fn render(compiled: &Compiled, format: ExportFormat) -> Result<Vec<u8>, ExportError> {
    match format {
        ExportFormat::Md => Ok(render_md(compiled).into_bytes()),
        ExportFormat::Txt => Ok(render_txt(compiled).into_bytes()),
        ExportFormat::Html => Ok(render_html(compiled).into_bytes()),
        ExportFormat::Docx => render_docx(compiled),
        ExportFormat::Epub => render_epub(compiled),
    }
}

/// Renders and writes the output file.
pub fn export_to_file(
    compiled: &Compiled,
    format: ExportFormat,
    path: &Path,
) -> Result<(), ExportError> {
    std::fs::write(path, render(compiled, format)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Per-format inlines
// ---------------------------------------------------------------------------

fn inline_md(items: &[Inline]) -> String {
    let mut out = String::new();
    for item in items {
        match item {
            Inline::Text(t) => out.push_str(t),
            Inline::Bold(inner) => {
                out.push_str("**");
                out.push_str(&inline_md(inner));
                out.push_str("**");
            }
            Inline::Italic(inner) => {
                out.push('*');
                out.push_str(&inline_md(inner));
                out.push('*');
            }
            Inline::Code(c) => {
                out.push('`');
                out.push_str(c);
                out.push('`');
            }
            Inline::FootnoteRef(id) => {
                let _ = write!(out, "[^{id}]");
            }
            Inline::Note(_) => {}
        }
    }
    out
}

fn inline_txt(items: &[Inline]) -> String {
    Inline::plain_text(items)
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn inline_html(items: &[Inline]) -> String {
    let mut out = String::new();
    for item in items {
        match item {
            Inline::Text(t) => out.push_str(&escape_html(t)),
            Inline::Bold(inner) => {
                out.push_str("<strong>");
                out.push_str(&inline_html(inner));
                out.push_str("</strong>");
            }
            Inline::Italic(inner) => {
                out.push_str("<em>");
                out.push_str(&inline_html(inner));
                out.push_str("</em>");
            }
            Inline::Code(c) => {
                out.push_str("<code>");
                out.push_str(&escape_html(c));
                out.push_str("</code>");
            }
            Inline::FootnoteRef(id) => {
                let _ = write!(out, "<sup>[{}]</sup>", escape_html(id));
            }
            Inline::Note(_) => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Markdown / TXT / HTML
// ---------------------------------------------------------------------------

fn render_md(compiled: &Compiled) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", compiled.title);
    if !compiled.author.trim().is_empty() {
        let _ = writeln!(out, "\n**{}**\n", compiled.author);
    }
    let mut prev_scene = false;
    for section in &compiled.sections {
        match section.kind {
            SectionKind::FrontMatter => {
                for block in &section.blocks {
                    if let Block::Para(items) = block {
                        let _ = writeln!(out, "\n_{}_\n", inline_md(items));
                    }
                }
            }
            SectionKind::Chapter => {
                prev_scene = false;
                let _ = writeln!(out, "\n# {}\n", section.title);
                for block in &section.blocks {
                    if let Block::Quote(items) = block {
                        let _ = writeln!(out, "> {}\n", inline_md(items));
                    }
                }
            }
            SectionKind::Scene => {
                if prev_scene {
                    let _ = writeln!(out, "\n{}\n", compiled.scene_separator);
                }
                prev_scene = true;
                for block in &section.blocks {
                    match block {
                        Block::Heading { level, text } => {
                            let _ = writeln!(
                                out,
                                "\n{} {}\n",
                                "#".repeat((*level).min(6) as usize),
                                inline_md(text)
                            );
                        }
                        Block::Para(items) => {
                            let _ = writeln!(out, "{}\n", inline_md(items));
                        }
                        Block::Quote(items) => {
                            let _ = writeln!(out, "> {}\n", inline_md(items));
                        }
                        Block::ListItem(items) => {
                            let _ = writeln!(out, "- {}", inline_md(items));
                        }
                        Block::SceneBreak => {
                            let _ = writeln!(out, "\n{}\n", compiled.scene_separator);
                            prev_scene = false;
                        }
                        Block::PageBreak => {}
                        Block::FootnoteDef { id, text } => {
                            let _ = writeln!(out, "[^{id}]: {}", inline_md(text));
                        }
                    }
                }
            }
        }
    }
    out
}

fn render_txt(compiled: &Compiled) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", compiled.title.to_uppercase());
    if !compiled.author.trim().is_empty() {
        let _ = writeln!(out, "{}\n", compiled.author);
    }
    let mut prev_scene = false;
    for section in &compiled.sections {
        match section.kind {
            SectionKind::FrontMatter => {
                for block in &section.blocks {
                    if let Block::Para(items) = block {
                        let _ = writeln!(out, "{}\n", inline_txt(items));
                    }
                }
            }
            SectionKind::Chapter => {
                prev_scene = false;
                let _ = writeln!(out, "\n{}\n", section.title.to_uppercase());
            }
            SectionKind::Scene => {
                if prev_scene {
                    let _ = writeln!(out, "\n{}\n", compiled.scene_separator);
                }
                prev_scene = true;
                for block in &section.blocks {
                    match block {
                        Block::Heading { text, .. } => {
                            let _ = writeln!(out, "\n{}\n", inline_txt(text));
                        }
                        Block::Para(items) | Block::Quote(items) | Block::ListItem(items) => {
                            let _ = writeln!(out, "{}\n", inline_txt(items));
                        }
                        Block::SceneBreak => {
                            let _ = writeln!(out, "\n{}\n", compiled.scene_separator);
                            prev_scene = false;
                        }
                        Block::PageBreak => {}
                        Block::FootnoteDef { id, text } => {
                            let _ = writeln!(out, "[{id}] {}", inline_txt(text));
                        }
                    }
                }
            }
        }
    }
    out
}

fn render_html(compiled: &Compiled) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"it\">\n<head>\n<meta charset=\"utf-8\">\n");
    let _ = writeln!(out, "<title>{}</title>", escape_html(&compiled.title));
    out.push_str("<style>body{max-width:38em;margin:2em auto;font-family:Georgia,serif;line-height:1.6;padding:0 1em}h1{text-align:center}h2{margin-top:2em}.sep{text-align:center;margin:2em 0}.author{text-align:center}.ded{text-align:center;font-style:italic;margin:3em 0}nav{margin:2em 0}.footnotes{margin-top:3em;border-top:1px solid #999;font-size:.9em}</style>\n</head>\n<body>\n");
    let _ = writeln!(out, "<h1>{}</h1>", escape_html(&compiled.title));
    if !compiled.author.trim().is_empty() {
        let _ = writeln!(
            out,
            "<p class=\"author\">{}</p>",
            escape_html(&compiled.author)
        );
    }
    // Index.
    out.push_str("<nav><ul>\n");
    for (i, section) in compiled.sections.iter().enumerate() {
        if section.kind == SectionKind::Chapter {
            let _ = writeln!(
                out,
                "<li><a href=\"#s{i}\">{}</a></li>",
                escape_html(&section.title)
            );
        }
    }
    out.push_str("</ul></nav>\n");
    let mut footnotes: Vec<(String, String)> = Vec::new();
    let mut prev_scene = false;
    for (i, section) in compiled.sections.iter().enumerate() {
        match section.kind {
            SectionKind::FrontMatter => {
                for block in &section.blocks {
                    if let Block::Para(items) = block {
                        let _ = writeln!(out, "<p class=\"ded\">{}</p>", inline_html(items));
                    }
                }
            }
            SectionKind::Chapter => {
                prev_scene = false;
                let _ = writeln!(out, "<h1 id=\"s{i}\">{}</h1>", escape_html(&section.title));
                for block in &section.blocks {
                    if let Block::Quote(items) = block {
                        let _ = writeln!(out, "<blockquote>{}</blockquote>", inline_html(items));
                    }
                }
            }
            SectionKind::Scene => {
                if prev_scene {
                    let _ = writeln!(
                        out,
                        "<p class=\"sep\">{}</p>",
                        escape_html(&compiled.scene_separator)
                    );
                }
                prev_scene = true;
                for block in &section.blocks {
                    match block {
                        Block::Heading { level, text } => {
                            let level = (*level).clamp(2, 6);
                            let _ = writeln!(out, "<h{level}>{}</h{level}>", inline_html(text));
                        }
                        Block::Para(items) => {
                            let _ = writeln!(out, "<p>{}</p>", inline_html(items));
                        }
                        Block::Quote(items) => {
                            let _ =
                                writeln!(out, "<blockquote>{}</blockquote>", inline_html(items));
                        }
                        Block::ListItem(items) => {
                            let _ = writeln!(out, "<ul><li>{}</li></ul>", inline_html(items));
                        }
                        Block::SceneBreak => {
                            let _ = writeln!(
                                out,
                                "<p class=\"sep\">{}</p>",
                                escape_html(&compiled.scene_separator)
                            );
                            prev_scene = false;
                        }
                        Block::PageBreak => {}
                        Block::FootnoteDef { id, text } => {
                            footnotes.push((id.clone(), inline_html(text)));
                        }
                    }
                }
            }
        }
    }
    if !footnotes.is_empty() {
        out.push_str("<div class=\"footnotes\"><h2>Note</h2>\n");
        for (id, text) in &footnotes {
            let _ = writeln!(out, "<p>[{}] {text}</p>", escape_html(id));
        }
        out.push_str("</div>\n");
    }
    out.push_str("</body>\n</html>\n");
    out
}

// ---------------------------------------------------------------------------
// DOCX (docx-rs)
// ---------------------------------------------------------------------------

fn render_docx(compiled: &Compiled) -> Result<Vec<u8>, ExportError> {
    use docx_rs::{AlignmentType, BreakType, Docx, Paragraph, Run};

    /// Half typographic points (docx-rs unit): 24 = 12 pt.
    fn run(items: &[Inline], size: usize, bold: bool, italic: bool) -> Vec<Run> {
        let mut runs = Vec::new();
        fn push(runs: &mut Vec<Run>, text: &str, size: usize, bold: bool, italic: bool) {
            if text.is_empty() {
                return;
            }
            let mut run = Run::new().add_text(text).size(size);
            if bold {
                run = run.bold();
            }
            if italic {
                run = run.italic();
            }
            runs.push(run);
        }
        fn walk(runs: &mut Vec<Run>, items: &[Inline], size: usize, bold: bool, italic: bool) {
            for item in items {
                match item {
                    Inline::Text(t) => push(runs, t, size, bold, italic),
                    Inline::Bold(inner) => walk(runs, inner, size, true, italic),
                    Inline::Italic(inner) => walk(runs, inner, size, bold, true),
                    Inline::Code(c) => push(runs, c, size, bold, italic),
                    Inline::FootnoteRef(id) => push(runs, &format!("[{id}]"), size, bold, italic),
                    Inline::Note(_) => {}
                }
            }
        }
        walk(&mut runs, items, size, bold, italic);
        if runs.is_empty() {
            runs.push(Run::new().add_text("").size(size));
        }
        runs
    }

    fn para(runs: Vec<Run>, centered: bool) -> Paragraph {
        let mut p = Paragraph::new();
        for r in runs {
            p = p.add_run(r);
        }
        if centered {
            p = p.align(AlignmentType::Center);
        }
        p
    }

    let mut docx = Docx::new();
    docx = docx.add_paragraph(para(
        run(&[Inline::Text(compiled.title.clone())], 56, true, false),
        true,
    ));
    if !compiled.author.trim().is_empty() {
        docx = docx.add_paragraph(para(
            run(&[Inline::Text(compiled.author.clone())], 24, false, false),
            true,
        ));
    }
    let mut footnotes: Vec<(String, Vec<Inline>)> = Vec::new();
    let mut prev_scene = false;
    let mut first_chapter = true;
    for section in &compiled.sections {
        match section.kind {
            SectionKind::FrontMatter => {
                for block in &section.blocks {
                    if let Block::Para(items) = block {
                        docx = docx.add_paragraph(para(run(items, 24, false, true), true));
                    }
                }
            }
            SectionKind::Chapter => {
                prev_scene = false;
                if compiled.page_break_before_chapters && !first_chapter {
                    docx = docx.add_paragraph(
                        Paragraph::new().add_run(Run::new().add_break(BreakType::Page)),
                    );
                }
                first_chapter = false;
                docx = docx.add_paragraph(para(
                    run(&[Inline::Text(section.title.clone())], 36, true, false),
                    false,
                ));
                for block in &section.blocks {
                    if let Block::Quote(items) = block {
                        docx = docx.add_paragraph(para(run(items, 22, false, true), false));
                    }
                }
            }
            SectionKind::Scene => {
                if prev_scene {
                    docx = docx.add_paragraph(para(
                        run(
                            &[Inline::Text(compiled.scene_separator.clone())],
                            24,
                            false,
                            false,
                        ),
                        true,
                    ));
                }
                prev_scene = true;
                for block in &section.blocks {
                    match block {
                        Block::Heading { text, .. } => {
                            docx = docx.add_paragraph(para(run(text, 30, true, false), false));
                        }
                        Block::Para(items) | Block::Quote(items) | Block::ListItem(items) => {
                            docx = docx.add_paragraph(para(run(items, 24, false, false), false));
                        }
                        Block::SceneBreak => {
                            docx = docx.add_paragraph(para(
                                run(
                                    &[Inline::Text(compiled.scene_separator.clone())],
                                    24,
                                    false,
                                    false,
                                ),
                                true,
                            ));
                            prev_scene = false;
                        }
                        Block::PageBreak => {
                            docx = docx.add_paragraph(
                                Paragraph::new().add_run(Run::new().add_break(BreakType::Page)),
                            );
                        }
                        Block::FootnoteDef { id, text } => {
                            footnotes.push((id.clone(), text.clone()));
                        }
                    }
                }
            }
        }
    }
    if !footnotes.is_empty() {
        docx = docx.add_paragraph(para(
            run(&[Inline::Text("Note".to_string())], 30, true, false),
            false,
        ));
        for (id, text) in &footnotes {
            let mut items = vec![Inline::Text(format!("[{id}] "))];
            items.extend(text.clone());
            docx = docx.add_paragraph(para(run(&items, 20, false, false), false));
        }
    }
    let mut buf = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buf)
        .map_err(|e| ExportError::Docx(e.to_string()))?;
    Ok(buf.into_inner())
}

// ---------------------------------------------------------------------------
// EPUB (epub-builder)
// ---------------------------------------------------------------------------

fn render_epub(compiled: &Compiled) -> Result<Vec<u8>, ExportError> {
    use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};

    fn xhtml(title: &str, body: &str) -> Vec<u8> {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xml:lang=\"it\" lang=\"it\">\n<head><title>{}</title></head>\n<body>\n{}\n</body>\n</html>\n",
            escape_html(title),
            body
        )
        .into_bytes()
    }

    let mut builder =
        EpubBuilder::new(ZipLibrary::new().map_err(|e| ExportError::Epub(e.to_string()))?)
            .map_err(|e| ExportError::Epub(e.to_string()))?;
    builder
        .metadata("title", &compiled.title)
        .map_err(|e| ExportError::Epub(e.to_string()))?;
    if !compiled.author.trim().is_empty() {
        builder
            .metadata("author", &compiled.author)
            .map_err(|e| ExportError::Epub(e.to_string()))?;
    }
    builder
        .metadata("lang", "it")
        .map_err(|e| ExportError::Epub(e.to_string()))?;

    // Title page.
    let mut cover = format!("<h1>{}</h1>", escape_html(&compiled.title));
    if !compiled.author.trim().is_empty() {
        let _ = write!(cover, "<p>{}</p>", escape_html(&compiled.author));
    }
    builder
        .add_content(
            EpubContent::new(
                "copertina.xhtml",
                Cursor::new(xhtml(&compiled.title, &cover)),
            )
            .title("Copertina"),
        )
        .map_err(|e| ExportError::Epub(e.to_string()))?;

    // Groups sections into files: dedication and chapters open a new file,
    // scenes append to the current file (each file added only once).
    let mut footnotes: Vec<(String, String)> = Vec::new();
    let mut files: Vec<(String, String)> = Vec::new();
    let mut cur_title = String::new();
    let mut cur_body = String::new();
    let flush = |files: &mut Vec<(String, String)>, title: &mut String, body: &mut String| {
        if !body.trim().is_empty() {
            let t = if title.trim().is_empty() {
                format!("Sezione {}", files.len() + 1)
            } else {
                std::mem::take(title)
            };
            files.push((t, std::mem::take(body)));
        } else {
            title.clear();
        }
    };
    for section in &compiled.sections {
        match section.kind {
            SectionKind::FrontMatter => {
                flush(&mut files, &mut cur_title, &mut cur_body);
                cur_title = String::from("Dedica");
                for block in &section.blocks {
                    if let Block::Para(items) = block {
                        let _ = write!(cur_body, "<p><em>{}</em></p>", inline_html(items));
                    }
                }
            }
            SectionKind::Chapter => {
                flush(&mut files, &mut cur_title, &mut cur_body);
                cur_title = section.title.clone();
                let _ = write!(cur_body, "<h1>{}</h1>", escape_html(&section.title));
                for block in &section.blocks {
                    if let Block::Quote(items) = block {
                        let _ = write!(
                            cur_body,
                            "<blockquote><em>{}</em></blockquote>",
                            inline_html(items)
                        );
                    }
                }
            }
            SectionKind::Scene => {
                if cur_title.trim().is_empty() && cur_body.trim().is_empty() {
                    cur_title = section.title.clone();
                }
                for block in &section.blocks {
                    match block {
                        Block::Heading { level, text } => {
                            let level = (*level).clamp(2, 6);
                            let _ = write!(cur_body, "<h{level}>{}</h{level}>", inline_html(text));
                        }
                        Block::Para(items) => {
                            let _ = write!(cur_body, "<p>{}</p>", inline_html(items));
                        }
                        Block::Quote(items) => {
                            let _ =
                                write!(cur_body, "<blockquote>{}</blockquote>", inline_html(items));
                        }
                        Block::ListItem(items) => {
                            let _ = write!(cur_body, "<ul><li>{}</li></ul>", inline_html(items));
                        }
                        Block::SceneBreak => {
                            let _ = write!(
                                cur_body,
                                "<p style=\"text-align:center\">{}</p>",
                                escape_html(&compiled.scene_separator)
                            );
                        }
                        Block::PageBreak => {}
                        Block::FootnoteDef { id, text } => {
                            footnotes.push((id.clone(), inline_html(text)));
                        }
                    }
                }
            }
        }
    }
    flush(&mut files, &mut cur_title, &mut cur_body);
    for (i, (title, body)) in files.iter().enumerate() {
        let name = format!("sezione-{}.xhtml", i + 1);
        builder
            .add_content(
                EpubContent::new(name, Cursor::new(xhtml(title, body))).title(title.clone()),
            )
            .map_err(|e| ExportError::Epub(e.to_string()))?;
    }
    if !footnotes.is_empty() {
        let mut body = String::from("<h1>Note</h1>");
        for (id, text) in &footnotes {
            let _ = write!(body, "<p>[{}] {text}</p>", escape_html(id));
        }
        builder
            .add_content(
                EpubContent::new("note.xhtml", Cursor::new(xhtml("Note", &body))).title("Note"),
            )
            .map_err(|e| ExportError::Epub(e.to_string()))?;
    }
    let mut out = Vec::new();
    builder
        .generate(&mut out)
        .map_err(|e| ExportError::Epub(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotvale_compile::{
        CompileProfile, Compiled, Section, SectionKind, compile, expand_selection,
    };
    use plotvale_model::Project;

    fn sample() -> Compiled {
        let mut project = Project::new("Il Romanzo", "Elena Autrice", "0.1.0");
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        let scene = project.tree.children_of(cap).unwrap()[0];
        project.set_doc_text(
            scene,
            "Un **inizio** e un *sospetto*.[^1]\n\n---\n\nSeconda parte.\n\n[^1]: La nota.",
        );
        let ids = expand_selection(&project, &[]);
        compile(&project, &ids, &CompileProfile::manuscript()).unwrap()
    }

    #[test]
    fn golden_md() {
        let md = render_md(&sample());
        let expected = "# Il Romanzo\n\n**Elena Autrice**\n\n\n# Chapter 1\n\nUn **inizio** e un *sospetto*.[^1]\n\n\n***\n\nSeconda parte.\n\n[^1]: La nota.\n";
        assert_eq!(md, expected);
    }

    #[test]
    fn golden_txt() {
        let txt = render_txt(&sample());
        assert!(txt.starts_with("IL ROMANZO\nElena Autrice\n"));
        assert!(txt.contains("CHAPTER 1"));
        assert!(txt.contains("Un inizio e un sospetto."));
        assert!(txt.contains("[1] La nota."));
    }

    #[test]
    fn golden_html() {
        let html = render_html(&sample());
        assert!(html.contains("<title>Il Romanzo</title>"));
        assert!(html.contains("<h1>Il Romanzo</h1>"));
        assert!(html.contains("<strong>inizio</strong>"));
        assert!(html.contains("<em>sospetto</em>"));
        assert!(html.contains("<div class=\"footnotes\">"));
    }

    #[test]
    fn docx_is_a_zip() {
        let bytes = render(&sample(), ExportFormat::Docx).unwrap();
        assert!(bytes.len() > 1000, "docx troppo piccolo: {}", bytes.len());
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }

    #[test]
    fn epub_is_a_zip() {
        let bytes = render(&sample(), ExportFormat::Epub).unwrap();
        assert!(bytes.len() > 1000, "epub troppo piccolo: {}", bytes.len());
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }

    #[test]
    fn formats_parse() {
        assert_eq!("docx".parse(), Ok(ExportFormat::Docx));
        assert_eq!("EPUB".parse(), Ok(ExportFormat::Epub));
        assert!("pdf".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn unused_section_ctor() {
        let _ = Section {
            title: String::new(),
            level: 0,
            kind: SectionKind::FrontMatter,
            blocks: Vec::new(),
        };
    }
}
