mod app;
mod file_tree;
mod viewer;

use app::MdxApp;

fn main() -> anyhow::Result<()> {
    let initial_folder = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Markdown Explorer")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Markdown Explorer",
        options,
        Box::new(|cc| Ok(Box::new(MdxApp::new(cc, initial_folder)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
