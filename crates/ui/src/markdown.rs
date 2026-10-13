//! Inline Markdown styling for the editor (Level A).
//!
//! Saved text stays pure Markdown; here we render it formatted: large headings,
//! rendered bold and italic, subtly colored markers. Markers stay visible
//! (the custom Level-B editor will hide them, see ROADMAP).

use std::sync::Arc;

use egui::text::{Galley, LayoutJob};
use egui::{FontId, TextFormat};

/// Lays out Markdown text with inline styling, ready for `TextEdit::layouter`.
/// `base` is the body font size (headings scale above it).
pub fn layout_markdown(ui: &egui::Ui, text: &str, wrap_width: f32, base: f32) -> Arc<Galley> {
    let visuals = ui.visuals();
    let ink = visuals.widgets.noninteractive.fg_stroke.color;
    let faint = if visuals.dark_mode {
        egui::Color32::from_gray(150)
    } else {
        egui::Color32::from_gray(120)
    };
    let normal = TextFormat::simple(FontId::proportional(base), ink);
    let weak = TextFormat::simple(FontId::proportional(base), faint);
    let _ = wrap_width;
    let mut job = LayoutJob::default();

    for (n, line) in text.split('\n').enumerate() {
        if n > 0 {
            job.append("\n", 0.0, normal.clone());
        }
        push_line(ui, &mut job, line, base, &normal, &weak);
    }
    if text.is_empty() {
        job.append("", 0.0, normal.clone());
    }
    ui.painter().layout_job(job)
}

fn push_line(
    ui: &egui::Ui,
    job: &mut LayoutJob,
    line: &str,
    base: f32,
    normal: &TextFormat,
    weak: &TextFormat,
) {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("### ") {
        push_heading(ui, job, rest, base + 2.0, normal, weak);
    } else if let Some(rest) = trimmed.strip_prefix("## ") {
        push_heading(ui, job, rest, base + 4.0, normal, weak);
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        push_heading(ui, job, rest, base + 7.0, normal, weak);
    } else if trimmed == "---" || trimmed == "***" {
        job.append(trimmed, 0.0, weak.clone());
    } else if let Some(rest) = trimmed.strip_prefix("> ") {
        let mut quote = normal.clone();
        quote.italics = true;
        quote.color = weak.color;
        push_inline(ui, job, rest, &quote, weak);
    } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        job.append(&line[..line.len() - trimmed.len() + 2], 0.0, weak.clone());
        push_inline(ui, job, &trimmed[2..], normal, weak);
    } else if trimmed.starts_with("[^") && trimmed.contains("]:") {
        let mut small = weak.clone();
        small.font_id = FontId::proportional(14.0);
        push_inline(ui, job, line, &small, weak);
    } else {
        push_inline(ui, job, line, normal, weak);
    }
}

fn push_heading(
    ui: &egui::Ui,
    job: &mut LayoutJob,
    rest: &str,
    size: f32,
    normal: &TextFormat,
    weak: &TextFormat,
) {
    let ink = ui.visuals().widgets.noninteractive.fg_stroke.color;
    let head = TextFormat::simple(FontId::proportional(size), ink);
    let _ = normal;
    push_inline(ui, job, rest, &head, weak);
}

/// Inline formatting with egui's built-in fonts (which have no real bold):
/// "bold" is highlighted like with a marker pen, italics are real,
/// markers stay in a muted color.
fn push_inline(
    ui: &egui::Ui,
    job: &mut LayoutJob,
    text: &str,
    base: &TextFormat,
    marker: &TextFormat,
) {
    let mut emph = base.clone();
    emph.background = ui.visuals().faint_bg_color;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut buf = String::new();
    // Style stack: (highlighted, italic).
    let mut stack: Vec<(bool, bool)> = vec![(false, false)];
    while i < chars.len() {
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            toggle(job, &mut buf, &mut stack, base, &emph, marker, true);
            i += 2;
        } else if chars[i] == '*' {
            toggle(job, &mut buf, &mut stack, base, &emph, marker, false);
            i += 1;
        } else if chars[i] == '`' {
            flush_current(job, &mut buf, &stack, base, &emph);
            if let Some(end) = chars.iter().skip(i + 1).position(|c| *c == '`') {
                let code: String = chars[i..i + 2 + end].iter().collect();
                let mut code_fmt = marker.clone();
                code_fmt.background = ui.visuals().code_bg_color;
                job.append(&code, 0.0, code_fmt);
                i += 2 + end;
            } else {
                buf.push('`');
                i += 1;
            }
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    flush_current(job, &mut buf, &stack, base, &emph);
}

fn current_format(stack: &[(bool, bool)], base: &TextFormat, emph: &TextFormat) -> TextFormat {
    let (bold, italic) = stack.last().copied().unwrap_or((false, false));
    let mut f = if bold { emph.clone() } else { base.clone() };
    // Same size as the current level (no "jumpy" lines).
    f.font_id = base.font_id.clone();
    if italic {
        f.italics = true;
    }
    f
}

fn flush_current(
    job: &mut LayoutJob,
    buf: &mut String,
    stack: &[(bool, bool)],
    base: &TextFormat,
    emph: &TextFormat,
) {
    if !buf.is_empty() {
        let fmt = current_format(stack, base, emph);
        job.append(&std::mem::take(buf), 0.0, fmt);
    }
}

/// Opens or closes a `**` (bold=true) or `*` (bold=false) marker.
#[allow(clippy::too_many_arguments)]
fn toggle(
    job: &mut LayoutJob,
    buf: &mut String,
    stack: &mut Vec<(bool, bool)>,
    base: &TextFormat,
    emph: &TextFormat,
    marker: &TextFormat,
    bold: bool,
) {
    let top = stack.last().copied().unwrap_or((false, false));
    let is_open = if bold { top.0 } else { top.1 };
    flush_current(job, buf, stack, base, emph);
    job.append(if bold { "**" } else { "*" }, 0.0, marker.clone());
    if is_open {
        stack.pop();
    } else if bold {
        stack.push((true, top.1));
    } else {
        stack.push((top.0, true));
    }
}
