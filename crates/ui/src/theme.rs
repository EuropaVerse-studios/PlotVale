//! PlotVale dark theme: the logo colors applied to egui.
//!
//! Warm black background, cream text, coral for accents, selections, and
//! progress. Generous spacing and soft corners for a modern look.

use egui::{Color32, CornerRadius, Margin, Stroke, Style, Visuals, style::Widgets};

use crate::brand::palette;

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Brand coral accent.
pub fn accent() -> Color32 {
    rgb(palette::ACCENT)
}

/// Text cream.
pub fn cream() -> Color32 {
    rgb(palette::CREAM)
}

/// Brand-bordered card to highlight content (corkboard, panels).
pub fn card_frame(style: &egui::Style) -> egui::Frame {
    let mut frame = egui::Frame::group(style);
    frame.fill = rgb(palette::CARD);
    frame.stroke = Stroke::new(1.0, rgb(palette::ACCENT_DEEP));
    frame.corner_radius = CornerRadius::same(10);
    frame.inner_margin = Margin::same(10);
    frame
}

/// Brand visuals (dark theme).
pub fn brand_visuals() -> Visuals {
    let accent = rgb(palette::ACCENT);
    let accent_light = rgb(palette::ACCENT_LIGHT);
    let accent_deep = rgb(palette::ACCENT_DEEP);
    let ink = rgb(palette::INK);
    let panel = rgb(palette::PANEL);
    let card = rgb(palette::CARD);
    let cream = rgb(palette::CREAM);
    let faint = rgb(palette::FAINT);

    let widget = |bg: Color32, fg: Color32| egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(1.0, faint),
        corner_radius: CornerRadius::same(8),
        fg_stroke: Stroke::new(1.5, fg),
        expansion: 1.0,
    };

    Visuals {
        dark_mode: true,
        override_text_color: Some(cream),
        hyperlink_color: accent_light,
        faint_bg_color: card,
        extreme_bg_color: ink,
        code_bg_color: card,
        warn_fg_color: Color32::from_rgb(0xE8, 0xB4, 0x4B),
        error_fg_color: Color32::from_rgb(0xF0, 0x6A, 0x5A),
        window_fill: panel,
        panel_fill: panel,
        window_stroke: Stroke::new(1.0, accent_deep),
        window_corner_radius: CornerRadius::same(10),
        menu_corner_radius: CornerRadius::same(8),
        widgets: Widgets {
            noninteractive: widget(card, cream),
            inactive: widget(Color32::from_rgb(0x33, 0x2A, 0x23), cream),
            hovered: widget(Color32::from_rgb(0x3D, 0x30, 0x28), cream),
            active: widget(accent_deep, cream),
            open: widget(card, cream),
        },
        selection: egui::style::Selection {
            bg_fill: accent,
            stroke: Stroke::new(1.0, cream),
        },
        window_shadow: egui::epaint::Shadow::NONE,
        popup_shadow: egui::epaint::Shadow::NONE,
        ..Visuals::dark()
    }
}

/// Modern style: brand visuals + spacing, applied once at startup.
pub fn brand_style() -> Style {
    let mut style = Style {
        visuals: brand_visuals(),
        ..Default::default()
    };
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.indent = 20.0;
    style
}

/// Applies the brand theme to the context (forced dark theme, full style).
pub fn apply_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.set_style_of(egui::Theme::Dark, brand_style());
}
