// Release builds on Windows are windowed programs. Without this, starting
// Grape from Explorer or Colony also opens a console window that stays up for
// as long as the player runs. Debug builds keep the console, so `cargo run`
// still shows the log.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

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
    #[cfg(windows)]
    if first.as_deref().is_some_and(|arg| arg.starts_with('-')) {
        attach_parent_console();
    }
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

/// Every flag prints and exits, and a windowed release build starts with no
/// console, so what it prints would go nowhere, even when a terminal started
/// it. This borrows that terminal's console. Windows keeps the standard
/// handles a parent passed explicitly, so output redirected to a file or a
/// pipe, as in the release smoke test, still lands there. The call fails
/// harmlessly when there is no parent console, as when Explorer starts Grape.
///
/// Flags only: a process attached to a console is ended with it, so a normal
/// launch from a terminal must not tie the player to that window.
#[cfg(windows)]
fn attach_parent_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    // SAFETY: AttachConsole takes a process ID and touches nothing in this
    // process's memory. Its only failure, no console to attach to, is ignored
    // on purpose.
    let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}
