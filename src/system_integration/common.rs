use std::collections::HashMap;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use super::{IntegrationError, SystemAction};

pub struct ShortcutState {
    _manager: GlobalHotKeyManager,
    actions: HashMap<u32, SystemAction>,
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
