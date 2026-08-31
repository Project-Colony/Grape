mod config;
mod eq;
mod library;
mod notifications;
mod player;
mod playlist;
mod system_integration;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

use crate::library::Catalog;

const USAGE: &str = "\
Grape - a desktop music player for a local library.

Usage:
  grape [LIBRARY_FOLDER]

Arguments:
  LIBRARY_FOLDER    Scan this folder instead of the one in preferences.

Options:
  -h, --help        Print this message and exit.
  -V, --version     Print the version and exit.
";

fn main() -> ExitCode {
    // Answered before anything opens a window. The release workflow smoke-tests
    // each built artifact with `--version`, on runners with no display: a build
    // that treated the flag as a library folder would hang or abort there,
    // after the asset had already been signed.
    let mut arguments = std::env::args().skip(1);
    let first = arguments.next();
    match first.as_deref() {
        Some("-V" | "--version") => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Some("-h" | "--help") => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        // A folder named like a flag is not worth supporting, and silently
        // scanning `--verbose` as a directory would be worse than refusing.
        Some(unknown) if unknown.starts_with('-') => {
            eprintln!("grape: unrecognised option '{unknown}'\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
        _ => {}
    }

    tracing_subscriber::fmt::init();

    let library_root_override = first.map(PathBuf::from);
    let catalog = Catalog::empty();

    if let Err(err) = ui::run(catalog, library_root_override) {
        eprintln!("UI error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
