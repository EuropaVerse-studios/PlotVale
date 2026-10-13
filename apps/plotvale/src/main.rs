//! PlotVale — the free anti-Scrivener (graphical interface).
//!
//! Entry point: opens the window and delegates everything to `plotvale-ui`.
//! No logic here: for automation see `plotvale-cli`.

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PlotVale",
        plotvale_ui::brand::native_options(),
        Box::new(|cc| Ok(Box::new(plotvale_ui::PlotValeApp::new(cc)) as Box<dyn eframe::App>)),
    )
}
