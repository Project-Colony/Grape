use std::process::Command;

use crate::system_integration::common::{
    ShortcutState, TrayState, build_shortcuts, build_tray, drain_shortcut_actions,
    drain_tray_actions,
};

use super::{AppInfo, IntegrationError, SystemAction, SystemIntegrationAvailability};

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

pub struct PlatformIntegration {
    app_info: AppInfo,
    tray: Option<TrayState>,
    shortcuts: Option<ShortcutState>,
}

impl PlatformIntegration {
    pub fn new(app_info: AppInfo) -> Self {
        Self {
            app_info,
            tray: None,
            shortcuts: None,
        }
    }

    pub fn availability() -> SystemIntegrationAvailability {
        SystemIntegrationAvailability {
            notifications: true,
            tray: true,
            global_shortcuts: true,
            hardware_acceleration: true,
        }
    }

    pub fn apply_launch_at_startup(&self, enabled: bool) -> Result<(), IntegrationError> {
        let value = format!("\"{}\"", self.app_info.exe_path.display());
        let status = if enabled {
            Command::new("reg")
                .args([
                    "add",
                    RUN_KEY,
                    "/v",
                    &self.app_info.name,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &value,
                    "/f",
                ])
                .status()
        } else {
            Command::new("reg")
                .args(["delete", RUN_KEY, "/v", &self.app_info.name, "/f"])
                .status()
        }
        .map_err(|err| IntegrationError::new(format!("Registry update failed: {err}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(IntegrationError::new(format!(
                "Registry update failed with status {status}"
            )))
        }
    }

    pub fn set_tray(&mut self, enabled: bool) -> Result<(), IntegrationError> {
        if enabled {
            if self.tray.is_none() {
                self.tray = Some(build_tray()?);
            }
        } else {
            self.tray = None;
        }
        Ok(())
    }

    pub fn set_global_shortcuts(&mut self, enabled: bool) -> Result<(), IntegrationError> {
        if enabled {
            if self.shortcuts.is_none() {
                self.shortcuts = Some(build_shortcuts()?);
            }
        } else {
            self.shortcuts = None;
        }
        Ok(())
    }

    pub fn drain_actions(&mut self) -> Vec<SystemAction> {
        let mut actions = Vec::new();
        if let Some(tray) = &self.tray {
            actions.extend(drain_tray_actions(tray));
        }
        if let Some(shortcuts) = &self.shortcuts {
            actions.extend(drain_shortcut_actions(shortcuts));
        }
        actions
    }
}
