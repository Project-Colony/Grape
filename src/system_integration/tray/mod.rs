//! System tray backend, selected per platform.
//!
//! Windows and macOS use `tray-icon`, whose native backends are win32 and
//! AppKit respectively. Linux is served by a separate StatusNotifierItem
//! implementation: `tray-icon`'s Linux backend goes through libappindicator
//! and GTK 3, which requires a GTK main loop this application (iced/winit)
//! does not have.

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{TrayState, build_tray, drain_tray_actions};
