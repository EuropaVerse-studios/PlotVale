//! PlotVale — l'anti-Scrivener libero (interfaccia grafica).
//!
//! Punto di ingresso: apre la finestra e delega tutto a `plotvale-ui`.
//! Non contiene logica: per automazioni vedi `plotvale-cli`.

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PlotVale",
        plotvale_ui::brand::native_options(),
        Box::new(|cc| Ok(Box::new(plotvale_ui::PlotValeApp::new(cc)) as Box<dyn eframe::App>)),
    )
}
