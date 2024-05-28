mod app;
mod config;
mod entry;
mod filesystem;
mod text_generator;
mod ui;
mod utils;
mod walker;

use eframe::run_native;
use crate::ui::App;
use log::info;
use std::env;
use env_logger::{Builder, fmt::Color};
use std::io::Write;

fn main() {
    // Set the RUST_LOG environment variable programmatically
    env::set_var("RUST_LOG", "info");

    // Initialize env_logger with a custom format
    Builder::from_default_env().format(|buf, record| {
        let mut style = buf.style();
        style.set_color(Color::Green);

        writeln!(
            buf,
            "[{}] {}",
            style.value(record.target()),  // Only the module name is green
            record.args()
        )
    }).init();

    info!("Starting File Tree Viewer...");
    let options = eframe::NativeOptions::default();
    run_native(Box::new(App::new()), options);
}
