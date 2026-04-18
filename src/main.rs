mod config;
mod eq;
mod library;
mod notifications;
mod player;
mod playlist;
mod system_integration;
mod ui;

use std::path::PathBuf;

use crate::library::Catalog;

fn main() {
    tracing_subscriber::fmt::init();

    let library_root_override = std::env::args().nth(1).map(PathBuf::from);
    let catalog = Catalog::empty();

    if let Err(err) = ui::run(catalog, library_root_override) {
        eprintln!("Erreur UI: {err}");
    }
}
