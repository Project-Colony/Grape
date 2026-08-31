//! StatusNotifierItem backend, used on Linux.
//!
//! `tray-icon`'s Linux backend builds its menu through `muda`, which calls
//! `gtk::Menu::new()`. That asserts GTK has been initialized, and nothing in
//! this application ever calls `gtk::init` -- iced runs a winit event loop, not
//! a GTK one. The assert is a `panic!`, so it walks straight through the
//! `map_err` in the caller and, with `panic = "abort"` in the release profile,
//! aborts the process.
//!
//! `ksni` speaks the same `org.kde.StatusNotifierItem` protocol libappindicator
//! implements, but over D-Bus in pure Rust, so it needs no GTK main loop and
//! reports failure as an `Err` the caller can log.

use std::sync::mpsc::{Receiver, Sender, channel};

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem, Tray};

use crate::system_integration::{IntegrationError, SystemAction};

const TRAY_ICON_SIZE: i32 = 16;

struct GrapeTray {
    actions: Sender<SystemAction>,
}

impl Tray for GrapeTray {
    /// libappindicator opened the menu on left click; keep that behaviour.
    /// Setting this to `false` would instead route left clicks to `activate`,
    /// which is where raising the window would go if that is ever wanted.
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").to_owned()
    }

    fn title(&self) -> String {
        "Grape".to_owned()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // The asset is straight RGBA; StatusNotifierItem wants ARGB32 in
        // network byte order, so each pixel rotates one byte to the right.
        let mut data = include_bytes!("../../../assets/logo_16.rgba").to_vec();
        for pixel in data.as_chunks_mut::<4>().0 {
            pixel.rotate_right(1);
        }
        vec![Icon {
            width: TRAY_ICON_SIZE,
            height: TRAY_ICON_SIZE,
            data,
        }]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Quit Grape".to_owned(),
                // Runs on the D-Bus service thread. Sending on an unbounded
                // channel does not block, which is what ksni asks of this
                // callback; the action is picked up by `drain_tray_actions`.
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.actions.send(SystemAction::Quit);
                }),
                ..StandardItem::default()
            }
            .into(),
        ]
    }
}

pub struct TrayState {
    handle: Handle<GrapeTray>,
    actions: Receiver<SystemAction>,
}

impl Drop for TrayState {
    fn drop(&mut self) {
        // Required, not tidiness. ksni's service loop selects over
        // `Some(msg) = handle_rx.recv()`; dropping the last handle closes that
        // channel, which only disables the select branch. Nothing but an
        // explicit shutdown request closes the connection and breaks the loop,
        // so without this the icon outlives `set_tray(false)` and cannot be
        // removed. `shutdown` only queues the request -- awaiting it is a
        // separate call -- so this does not block.
        let _ = self.handle.shutdown();
    }
}

/// Note this blocks the calling thread for a few D-Bus round trips (single-digit
/// milliseconds against a healthy bus), and zbus is left with no method-reply
/// timeout, so a wedged tray host would stall the caller.
pub fn build_tray() -> Result<TrayState, IntegrationError> {
    let (sender, actions) = channel();
    let handle = GrapeTray { actions: sender }
        // Grape writes its own XDG autostart entry, so it routinely starts
        // before the panel owns `org.kde.StatusNotifierWatcher`. Without this,
        // that race is a hard error ksni never retries and Grape never retries
        // either, leaving no icon for the whole session while the setting still
        // reads as enabled. With it, the failure is soft: the service loop
        // starts anyway and re-registers when the watcher appears. The cost is
        // that a system with no SNI host at all now yields no icon and no
        // error, which is the better trade for a race we cause ourselves.
        .assume_sni_available(true)
        .spawn()
        .map_err(|err| IntegrationError::new(format!("Tray icon error: {err}")))?;
    Ok(TrayState { handle, actions })
}

pub fn drain_tray_actions(tray: &TrayState) -> Vec<SystemAction> {
    tray.actions.try_iter().collect()
}
