//! `tray-icon` backend.

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::system_integration::{IntegrationError, SystemAction};

const TRAY_ICON_SIZE: u32 = 16;

pub struct TrayState {
    _tray: TrayIcon,
    quit_id: MenuId,
}

pub fn build_tray() -> Result<TrayState, IntegrationError> {
    let menu = Menu::new();
    let quit = MenuItem::new("Quit Grape", true, None);
    menu.append(&quit)
        .map_err(|err| IntegrationError::new(format!("Tray menu error: {err}")))?;
    let icon = default_tray_icon()?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Grape")
        .with_icon(icon)
        .build()
        .map_err(|err| IntegrationError::new(format!("Tray icon error: {err}")))?;
    Ok(TrayState {
        _tray: tray,
        quit_id: quit.id().clone(),
    })
}

pub fn drain_tray_actions(tray: &TrayState) -> Vec<SystemAction> {
    let receiver = MenuEvent::receiver();
    let mut actions = Vec::new();
    while let Ok(event) = receiver.try_recv() {
        if event.id == tray.quit_id {
            actions.push(SystemAction::Quit);
        }
    }
    actions
}

fn default_tray_icon() -> Result<Icon, IntegrationError> {
    let rgba = include_bytes!("../../../assets/logo_16.rgba").to_vec();
    Icon::from_rgba(rgba, TRAY_ICON_SIZE, TRAY_ICON_SIZE)
        .map_err(|err| IntegrationError::new(format!("Tray icon error: {err}")))
}
