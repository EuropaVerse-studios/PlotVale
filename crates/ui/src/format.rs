//! Word-style formatting toolbar for the editor.
//!
//! The saved text stays Markdown; the toolbar (and shortcuts) inserts or
//! toggles markers around the selection, Google-Docs style. Cursor handling
//! uses egui's `TextEditState`, so the caret lands where writers expect it.
//! All offsets below are char-based (egui cursors are char-based, not byte-based).

use plotvale_model::NodeId;

use crate::i18n::Locale;

/// Stable egui id of a document editor.
pub fn editor_id(id: NodeId) -> egui::Id {
    egui::Id::new(format!("sc-editor-{id}"))
}

/// Byte offset of a char index (always a char boundary).
fn byte_of(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

/// Current selection as char indices, collapsed to the caret if none.
fn selection_chars(ctx: &egui::Context, eid: egui::Id, total_chars: usize) -> (usize, usize) {
    let range = egui::TextEdit::load_state(ctx, eid)
        .and_then(|s| s.cursor.char_range())
        .map(|r| r.as_sorted_char_range());
    match range {
        Some(r) => (r.start.0.min(total_chars), r.end.0.min(total_chars)),
        None => (total_chars, total_chars),
    }
}

/// Moves the caret, keeping editor focus.
fn place_cursor(ctx: &egui::Context, eid: egui::Id, char_idx: usize) {
    if let Some(mut state) = egui::TextEdit::load_state(ctx, eid) {
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(char_idx),
            )));
        egui::TextEdit::store_state(ctx, eid, state);
        ctx.memory_mut(|mem| mem.request_focus(eid));
    }
}

/// Wraps the selection in markers; toggles off when already wrapped.
/// With an empty selection the markers are inserted and the caret lands between them.
pub fn wrap(ctx: &egui::Context, eid: egui::Id, text: &mut String, before: &str, after: &str) {
    let total = text.chars().count();
    let (s, e) = selection_chars(ctx, eid, total);
    let (bs, be) = (byte_of(text, s), byte_of(text, e));
    let selected = text[bs..be].to_owned();
    let before_chars = before.chars().count();
    if !selected.is_empty() && text[..bs].ends_with(before) && text[be..].starts_with(after) {
        let after_len = after.len();
        text.replace_range(be..be + after_len, "");
        text.replace_range(bs - before.len()..bs, "");
        place_cursor(ctx, eid, s.saturating_sub(before_chars));
        return;
    }
    text.replace_range(bs..be, &format!("{before}{selected}{after}"));
    let caret = if selected.is_empty() {
        s + before_chars
    } else {
        s + before_chars + selected.chars().count() + after.chars().count()
    };
    place_cursor(ctx, eid, caret);
}

/// Strips one Markdown heading prefix (`# ` … `###### `), if present.
fn strip_headings(line: &str) -> &str {
    if let Some(hash_end) = line.find(' ') {
        let (hashes, _) = line.split_at(hash_end);
        if !hashes.is_empty() && hashes.chars().all(|c| c == '#') && hashes.len() <= 6 {
            return &line[hash_end + 1..];
        }
    }
    line
}

/// Toggles a line prefix (`# `, `> `, `- `) on the caret's line.
pub fn line_prefix(ctx: &egui::Context, eid: egui::Id, text: &mut String, prefix: &str) {
    let total = text.chars().count();
    let (s, _) = selection_chars(ctx, eid, total);
    let cursor_byte = byte_of(text, s);
    let ls = text[..cursor_byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let le = text[cursor_byte..]
        .find('\n')
        .map(|i| i + cursor_byte)
        .unwrap_or(text.len());
    let line = text[ls..le].to_owned();
    let ls_chars = text[..ls].chars().count();
    let offset_chars = text[ls..cursor_byte].chars().count();
    let same = line.starts_with(prefix);
    let core = if prefix.starts_with('#') {
        strip_headings(&line)
    } else {
        line.as_str()
    };
    let new_line = if same {
        line[prefix.len()..].to_owned()
    } else {
        format!("{prefix}{core}")
    };
    let new_offset = if same {
        offset_chars.saturating_sub(prefix.chars().count())
    } else {
        offset_chars + (new_line.chars().count() - line.chars().count())
    };
    text.replace_range(ls..le, &new_line);
    place_cursor(ctx, eid, ls_chars + new_offset);
}

/// Toolbar button: icon glyph with hover tooltip. Returns clicked.
fn tool_btn(row: &mut egui::Ui, icon: &str, tip: &str) -> bool {
    row.button(egui::RichText::new(icon).size(16.0))
        .on_hover_text(tip)
        .clicked()
}

/// Editing action chosen from the toolbar; applied by the caller so the
/// model sync stays in one place.
pub enum FormatAction {
    Wrap {
        before: &'static str,
        after: &'static str,
    },
    LinePrefix {
        prefix: &'static str,
    },
}

/// Word-style toolbar. Returns the chosen action, if any.
pub fn toolbar(
    bar: &mut egui::Ui,
    loc: &Locale,
    font_size: &mut f32,
    use_icons: bool,
) -> Option<FormatAction> {
    let mut action = None;
    bar.horizontal_wrapped(|row| {
        // `use_icons` is always true once the font is registered; kept as a
        // parameter so tests can render without it.
        let b = if use_icons {
            crate::icons::ic::TEXT_B
        } else {
            "B"
        };
        let i = if use_icons {
            crate::icons::ic::TEXT_ITALIC
        } else {
            "I"
        };
        if tool_btn(row, b, loc.get("fmt_b")) {
            action = Some(FormatAction::Wrap {
                before: "**",
                after: "**",
            });
        }
        if tool_btn(row, i, loc.get("fmt_i")) {
            action = Some(FormatAction::Wrap {
                before: "*",
                after: "*",
            });
        }
        if tool_btn(
            row,
            if use_icons {
                crate::icons::ic::CODE
            } else {
                "<>"
            },
            loc.get("fmt_code"),
        ) {
            action = Some(FormatAction::Wrap {
                before: "`",
                after: "`",
            });
        }
        row.separator();
        for (icon, tip, prefix) in [
            (
                if use_icons {
                    crate::icons::ic::TEXT_H_ONE
                } else {
                    "H1"
                },
                loc.get("fmt_h1"),
                "# ",
            ),
            (
                if use_icons {
                    crate::icons::ic::TEXT_H_TWO
                } else {
                    "H2"
                },
                loc.get("fmt_h2"),
                "## ",
            ),
            (
                if use_icons {
                    crate::icons::ic::TEXT_H_THREE
                } else {
                    "H3"
                },
                loc.get("fmt_h3"),
                "### ",
            ),
        ] {
            if tool_btn(row, icon, tip) {
                action = Some(FormatAction::LinePrefix { prefix });
            }
        }
        row.separator();
        if tool_btn(
            row,
            if use_icons {
                crate::icons::ic::QUOTES
            } else {
                "\u{275d}"
            },
            loc.get("fmt_quote"),
        ) {
            action = Some(FormatAction::LinePrefix { prefix: "> " });
        }
        if tool_btn(
            row,
            if use_icons {
                crate::icons::ic::LIST_BULLETS
            } else {
                "\u{2630}"
            },
            loc.get("fmt_list"),
        ) {
            action = Some(FormatAction::LinePrefix { prefix: "- " });
        }
        if tool_btn(
            row,
            if use_icons {
                crate::icons::ic::NOTE_PENCIL
            } else {
                "+"
            },
            loc.get("fmt_note"),
        ) {
            action = Some(FormatAction::Wrap {
                before: "[[note: ",
                after: "]]",
            });
        }
        row.separator();
        if row
            .button(egui::RichText::new("A+").size(14.0))
            .on_hover_text(loc.get("fmt_size_up"))
            .clicked()
        {
            *font_size = (*font_size + 1.0).min(26.0);
        }
        if row
            .button(egui::RichText::new("A\u{2212}").size(14.0))
            .on_hover_text(loc.get("fmt_size_down"))
            .clicked()
        {
            *font_size = (*font_size - 1.0).max(12.0);
        }
    });
    action
}
