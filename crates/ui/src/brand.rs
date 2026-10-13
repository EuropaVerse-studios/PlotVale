//! Brand PlotVale: logo, icona della finestra e palette.
//!
//! I colori sono misurati dal logo (`assets/logo.png`): corallo su nero caldo,
//! testi crema. Il tema scuro dell'app (`theme::brand_visuals`) li applica.

/// PNG del logo (1024×1024) incorporato nel binario.
pub const LOGO_PNG: &[u8] = include_bytes!("../../assets/logo.png");

/// Palette del brand (RGB, misurata dal logo).
pub mod palette {
    /// Corallo del marchio — accento primario.
    pub const ACCENT: [u8; 3] = [0xF0, 0x70, 0x50];
    /// Corallo chiaro per hover e stati attivi.
    pub const ACCENT_LIGHT: [u8; 3] = [0xF4, 0x83, 0x6A];
    /// Corallo profondo per bordi e dettagli.
    pub const ACCENT_DEEP: [u8; 3] = [0xC9, 0x50, 0x2F];
    /// Nero caldo dello sfondo.
    pub const INK: [u8; 3] = [0x17, 0x13, 0x10];
    /// Nero caldo dei pannelli.
    pub const PANEL: [u8; 3] = [0x20, 0x19, 0x15];
    /// Nero caldo delle card.
    pub const CARD: [u8; 3] = [0x28, 0x20, 0x1A];
    /// Crema dei testi.
    pub const CREAM: [u8; 3] = [0xF5, 0xED, 0xDF];
    /// Crema attenuata per testi secondari.
    pub const FAINT: [u8; 3] = [0xA8, 0x9C, 0x8D];
}

/// Decodifica il logo ridimensionato a `max_side` px (lato lungo).
pub fn decode_logo(max_side: u32) -> Option<egui::ColorImage> {
    let img = image::load_from_memory(LOGO_PNG).ok()?;
    let img = img.resize(
        max_side,
        max_side,
        image::imageops::FilterType::Lanczos3,
    );
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    Some(egui::ColorImage::from_rgba_unmultiplied([w, h], rgba.as_raw()))
}

/// Icona della finestra (128px) a partire dal logo.
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

/// Opzioni native con titolo, id e icona PlotVale.
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
