//! System tray backend, selected per platform.
//!
//! Windows and macOS use `tray-icon`, whose native backends are win32 and
//! AppKit respectively. Linux uses `ksni`: `tray-icon`'s Linux backend goes
//! through libappindicator and GTK 3, which requires a GTK main loop this
//! application (iced/winit) does not have, and aborts the process without one.
//! Both speak `org.kde.StatusNotifierItem` on Linux, so this is a swap of
//! implementation, not of protocol.

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
mod native;
#[cfg(target_os = "linux")]
mod sni;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
pub use native::{TrayState, build_tray, drain_tray_actions};
#[cfg(target_os = "linux")]
pub use sni::{TrayState, build_tray, drain_tray_actions};
