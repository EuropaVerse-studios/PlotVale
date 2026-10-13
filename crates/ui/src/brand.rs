//! PlotVale brand: logo, window icon, and palette.
//!
//! Colors are sampled from the logo (`assets/logo.png`): coral on warm black,
//! cream text. The app dark theme (`theme::brand_visuals`) applies them.

/// Logo PNG (1024×1024) embedded in the binary.
pub const LOGO_PNG: &[u8] = include_bytes!("../../../assets/logo.png");

/// Brand palette (RGB, sampled from the logo).
pub mod palette {
    /// Brand coral — primary accent.
    pub const ACCENT: [u8; 3] = [0xF0, 0x70, 0x50];
    /// Light coral for hover and active states.
    pub const ACCENT_LIGHT: [u8; 3] = [0xF4, 0x83, 0x6A];
    /// Deep coral for borders and details.
    pub const ACCENT_DEEP: [u8; 3] = [0xC9, 0x50, 0x2F];
    /// Warm black background.
    pub const INK: [u8; 3] = [0x17, 0x13, 0x10];
    /// Warm black panels.
    pub const PANEL: [u8; 3] = [0x20, 0x19, 0x15];
    /// Warm black cards.
    pub const CARD: [u8; 3] = [0x28, 0x20, 0x1A];
    /// Cream text.
    pub const CREAM: [u8; 3] = [0xF5, 0xED, 0xDF];
    /// Muted cream for secondary text.
    pub const FAINT: [u8; 3] = [0xA8, 0x9C, 0x8D];
}

/// Decodes the logo resized to `max_side` px (long side).
pub fn decode_logo(max_side: u32) -> Option<egui::ColorImage> {
    let img = image::load_from_memory(LOGO_PNG).ok()?;
    let img = img.resize(max_side, max_side, image::imageops::FilterType::Lanczos3);
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [w, h],
        rgba.as_raw(),
    ))
}

/// Window icon (128px) derived from the logo.
pub fn window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(LOGO_PNG).ok()?;
    let img = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
    let rgba = img.into_rgba8();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width: 128,
        height: 128,
    })
}

/// Native options with PlotVale title, id, and icon.
pub fn native_options() -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("PlotVale")
        .with_app_id("plotvale");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    eframe::NativeOptions {
        viewport,
        ..Default::default()
    }
}
