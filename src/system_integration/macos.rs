use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        let plist_path = self.launch_agent_path()?;
        if enabled {
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent).map_err(|err| {
                    IntegrationError::new(format!("LaunchAgent directory error: {err}"))
                })?;
            }
            let plist = self.launch_agent_plist();
            fs::write(&plist_path, plist)
                .map_err(|err| IntegrationError::new(format!("LaunchAgent write error: {err}")))?;
            let _ = Command::new("launchctl")
                .arg("load")
                .arg(&plist_path)
                .status();
        } else {
            let _ = Command::new("launchctl")
                .arg("unload")
                .arg(&plist_path)
                .status();
            if plist_path.exists() {
                fs::remove_file(&plist_path).map_err(|err| {
                    IntegrationError::new(format!("LaunchAgent removal error: {err}"))
                })?;
            }
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

    fn launch_agent_path(&self) -> Result<PathBuf, IntegrationError> {
        let home = std::env::var("HOME")
            .map_err(|err| IntegrationError::new(format!("HOME not set: {err}")))?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", self.app_info.identifier)))
    }

    fn launch_agent_plist(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{identifier}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe_path}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
            identifier = self.app_info.identifier,
            exe_path = self.app_info.exe_path.display()
        )
    }
}
