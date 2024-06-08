mod app;
mod config;
mod entry;
mod text_generator;
mod ui;
mod utils;
mod walker;

use crate::ui::App;
use eframe::run_native;
use env_logger::{fmt::Color, Builder};
use log::info;
use std::env;
use std::io::Write;

fn main() {
    env::set_var("RUST_LOG", "info");

    Builder::from_default_env()
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(Color::Green);

            writeln!(
                buf,
                "[{}] {}",
                style.value(record.target()), 
                record.args()
            )
        })
        .init();

    info!("Starting File Tree Viewer...");
    let options = eframe::NativeOptions::default();
    run_native(Box::new(App::new()), options);
}
