mod app;
mod config;
mod entry;
mod filesystem;
mod text_generator;
mod ui;
mod utils;
mod walker;

use std::env;

use eframe::run_native;
use crate::ui::App;
use log::info;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Starting File Tree Viewer...");
    let options = eframe::NativeOptions::default();
    run_native(Box::new(App::new()), options);
}
