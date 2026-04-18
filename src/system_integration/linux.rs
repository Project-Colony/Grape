use std::fs;
use std::path::PathBuf;

use crate::system_integration::common::{
    ShortcutState, TrayState, build_shortcuts, build_tray, drain_shortcut_actions,
    drain_tray_actions,
};

use super::{AppInfo, IntegrationError, SystemAction, SystemIntegrationAvailability};

pub struct PlatformIntegration {
    app_info: AppInfo,
    tray: Option<TrayState>,
    shortcuts: Option<ShortcutState>,
}

impl PlatformIntegration {
    pub fn new(app_info: AppInfo) -> Self {
        Self { app_info, tray: None, shortcuts: None }
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
        let autostart_path = self.autostart_path()?;
        if enabled {
            if let Some(parent) = autostart_path.parent() {
                fs::create_dir_all(parent).map_err(|err| {
                    IntegrationError::new(format!("Autostart directory error: {err}"))
                })?;
            }
            fs::write(&autostart_path, self.autostart_desktop_entry())
                .map_err(|err| IntegrationError::new(format!("Autostart write error: {err}")))?;
        } else if autostart_path.exists() {
            fs::remove_file(&autostart_path)
                .map_err(|err| IntegrationError::new(format!("Autostart removal error: {err}")))?;
        }
        Ok(())
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

    fn autostart_path(&self) -> Result<PathBuf, IntegrationError> {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.config")))
            .map_err(|err| IntegrationError::new(format!("Config path error: {err}")))?;
        Ok(PathBuf::from(config_home).join("autostart").join("grape.desktop"))
    }

    fn autostart_desktop_entry(&self) -> String {
        // Desktop Entry spec requires special characters in Exec values to be
        // quoted.  Wrapping the path in double-quotes handles spaces, while
        // inner double-quotes, backticks, and dollar signs are escaped.
        let raw = self.app_info.exe_path.display().to_string();
        let escaped = raw
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`");
        format!(
            r#"[Desktop Entry]
Type=Application
Name={}
Exec="{escaped}"
X-GNOME-Autostart-enabled=true
"#,
            self.app_info.name,
        )
    }
}
