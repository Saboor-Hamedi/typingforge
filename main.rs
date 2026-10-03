#![windows_subsystem = "windows"]


use eframe::egui;
use forgetyping::app::VelotypeApp;

fn main() -> eframe::Result<()> {
    // If invoked as `cargo run publish` or `velotype publish`
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "publish" {
        println!("[Velotype] Invoking automated publish pipeline...");
        let status = std::process::Command::new("cargo")
            .args(["run", "--bin", "publish"])
            .status();
        if let Ok(st) = status {
            if st.success() {
                return Ok(());
            }
        }
    }

    // Borderless desktop window with default around 920 width and 700 height (fully resizable & maximizable)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Velotype - Kinetic Typing & Real-Time Velocity Graph")
            .with_decorations(false) // Borderless window!
            .with_transparent(true)
            .with_inner_size([920.0, 700.0])
            .with_min_inner_size([860.0, 600.0])
            .with_resizable(true)
            .with_maximize_button(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Velotype",
        native_options,
        Box::new(|cc| Ok(Box::new(VelotypeApp::new(cc)))),
    )
}
