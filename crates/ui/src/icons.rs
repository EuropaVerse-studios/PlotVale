//! Real icons for the interface (Phosphor icon font).
//!
//! The font is registered once at startup ([`register`]); widgets then use
//! the [`ic`] constants as text prefixes, e.g. `format!("{} {}", ic::SAVE, label)`.

/// Registers the Phosphor regular font with the context (call once at startup).
pub fn register(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

/// Icon glyphs used across menus, binder, toolbar, and wizard.
pub mod ic {
    pub use egui_phosphor::variants::regular::*;
}
