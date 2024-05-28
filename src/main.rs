mod app;
mod config;
mod entry;
mod text_generator;
mod ui;
mod walker;

use eframe::run_native;
use crate::ui::App;

fn main() {
    let options = eframe::NativeOptions::default();
    run_native(Box::new(App::new()), options);
}
