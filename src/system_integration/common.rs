use std::collections::HashMap;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::{IntegrationError, SystemAction};

const TRAY_ICON_SIZE: u32 = 16;

pub struct TrayState {
    _tray: TrayIcon,
    quit_id: MenuId,
}

pub struct ShortcutState {
    _manager: GlobalHotKeyManager,
    actions: HashMap<u32, SystemAction>,
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

pub fn build_shortcuts() -> Result<ShortcutState, IntegrationError> {
    let manager = GlobalHotKeyManager::new()
        .map_err(|err| IntegrationError::new(format!("Hotkey manager error: {err}")))?;
    let mut actions = HashMap::new();
    let toggle = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);
    manager
        .register(toggle)
        .map_err(|err| IntegrationError::new(format!("Hotkey register error: {err}")))?;
    actions.insert(toggle.id(), SystemAction::TogglePlayPause);
    let next = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowRight);
    manager
        .register(next)
        .map_err(|err| IntegrationError::new(format!("Hotkey register error: {err}")))?;
    actions.insert(next.id(), SystemAction::NextTrack);
    let previous = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft);
    manager
        .register(previous)
        .map_err(|err| IntegrationError::new(format!("Hotkey register error: {err}")))?;
    actions.insert(previous.id(), SystemAction::PreviousTrack);
    Ok(ShortcutState {
        _manager: manager,
        actions,
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

pub fn drain_shortcut_actions(shortcuts: &ShortcutState) -> Vec<SystemAction> {
    let receiver = GlobalHotKeyEvent::receiver();
    let mut actions = Vec::new();
    while let Ok(event) = receiver.try_recv() {
        if event.state == HotKeyState::Pressed {
            if let Some(action) = shortcuts.actions.get(&event.id) {
                actions.push(*action);
            }
        }
    }
    actions
}

fn default_tray_icon() -> Result<Icon, IntegrationError> {
    let rgba = include_bytes!("../../assets/logo_16.rgba").to_vec();
    Icon::from_rgba(rgba, TRAY_ICON_SIZE, TRAY_ICON_SIZE)
        .map_err(|err| IntegrationError::new(format!("Tray icon error: {err}")))
}
