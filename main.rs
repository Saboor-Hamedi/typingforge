//! Application binary entry point for TypingForge.
//!
//! Handles command-line arguments (such as the `--publish` subcommand pipeline)
//! and configures the `eframe` / `egui` native desktop window (borderless, transparent,
//! resizable, and min-sized) before launching the main [`VelotypeApp`] event loop.

#![windows_subsystem = "windows"]

use eframe::egui;
use forgetyping::app::VelotypeApp;

/// Main desktop application entry point.
///
/// Sets up the desktop viewport with a modern borderless window design and initializes
/// the egui frame runner.
fn main() -> eframe::Result<()> {
    // If invoked as `cargo run publish` or `velotype publish`
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "publish" {
        println!("[Velotype] Invoking automated publish pipeline...");
        let status = std::process::Command::new("cargo")
            .args(["run", "--bin", "publish"])
            .status();
        match status {
            Ok(st) if st.success() => return Ok(()),
            Ok(st) => {
                eprintln!("[Velotype] Publish failed with exit code: {:?}", st.code());
                std::process::exit(st.code().unwrap_or(1));
            }
            Err(e) => {
                eprintln!("[Velotype] Failed to run cargo publish: {e}");
                std::process::exit(1);
            }
        }
    }

    // Borderless desktop window with default around 920 width and 700 height (fully resizable & maximizable)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Velotype - Kinetic Typing & Real-Time Velocity Graph")
            .with_decorations(false) // Borderless custom window titlebar rendered in UI
            .with_transparent(true)  // Enables window translucency and rounded outer corners
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

