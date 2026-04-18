use std::path::PathBuf;

use crate::config::UserSettings;
use tracing::{info, warn};

#[cfg(not(target_arch = "wasm32"))]
mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unsupported;

#[cfg(target_os = "windows")]
use windows::PlatformIntegration;
#[cfg(target_os = "macos")]
use macos::PlatformIntegration;
#[cfg(target_os = "linux")]
use linux::PlatformIntegration;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
use unsupported::PlatformIntegration;

const APP_NAME: &str = "Grape";
const APP_IDENTIFIER: &str = "com.colony.grape";

#[derive(Debug, Clone, Copy)]
pub struct SystemIntegrationAvailability {
    pub notifications: bool,
    pub tray: bool,
    pub global_shortcuts: bool,
    pub hardware_acceleration: bool,
}

impl SystemIntegrationAvailability {
    pub fn detect() -> Self {
        PlatformIntegration::availability()
    }
}

#[derive(Debug, Clone)]
struct AppInfo {
    name: String,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    identifier: String,
    exe_path: PathBuf,
}

impl AppInfo {
    fn detect() -> Self {
        let exe_path = std::env::current_exe().unwrap_or_else(|err| {
            warn!(error = %err, "Failed to resolve current executable path");
            PathBuf::from(APP_NAME)
        });
        Self {
            name: APP_NAME.to_string(),
            identifier: APP_IDENTIFIER.to_string(),
            exe_path,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    Quit,
    TogglePlayPause,
    NextTrack,
    PreviousTrack,
}

#[derive(Debug)]
pub struct IntegrationError {
    message: String,
}

impl IntegrationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for IntegrationError {}

pub struct SystemIntegration {
    availability: SystemIntegrationAvailability,
    platform: PlatformIntegration,
}

impl SystemIntegration {
    pub fn availability(&self) -> SystemIntegrationAvailability {
        self.availability
    }

    pub fn sync(mut existing: Option<Self>, settings: &mut UserSettings) -> (Option<Self>, bool) {
        let availability = existing
            .as_ref()
            .map(|integration| integration.availability)
            .unwrap_or_else(SystemIntegrationAvailability::detect);
        let changed = apply_availability(settings, availability);
        let wants_integration = settings.wants_system_integration() || settings.launch_at_startup;
        let mut integration = existing.take().unwrap_or_else(|| Self {
            availability,
            platform: PlatformIntegration::new(AppInfo::detect()),
        });
        integration.apply_settings(settings);
        let next = if wants_integration {
            Some(integration)
        } else {
            None
        };
        (next, changed)
    }

    pub fn drain_actions(&mut self) -> Vec<SystemAction> {
        self.platform.drain_actions()
    }

    fn apply_settings(&mut self, settings: &UserSettings) {
        if let Err(err) = self
            .platform
            .apply_launch_at_startup(settings.launch_at_startup)
        {
            warn!(error = %err, "Failed to apply launch at startup integration");
        }
        if let Err(err) = self.platform.set_tray(settings.system_tray_enabled) {
            warn!(error = %err, "Failed to apply system tray integration");
        }
        if let Err(err) = self
            .platform
            .set_global_shortcuts(settings.enable_advanced_shortcuts)
        {
            warn!(error = %err, "Failed to apply global shortcuts integration");
        }
        if settings.launch_at_startup {
            info!("Launch at startup integration enabled.");
        }
    }
}

fn apply_availability(
    settings: &mut UserSettings,
    availability: SystemIntegrationAvailability,
) -> bool {
    let mut changed = false;
    if !availability.notifications {
        if settings.notifications_enabled || settings.now_playing_notifications {
            settings.notifications_enabled = false;
            settings.now_playing_notifications = false;
            changed = true;
            warn!("System notifications unavailable; disabling notification settings.");
        }
    }
    if !availability.tray && settings.system_tray_enabled {
        settings.system_tray_enabled = false;
        changed = true;
        warn!("System tray unavailable; disabling tray integration.");
    }
    if !availability.global_shortcuts && settings.enable_advanced_shortcuts {
        settings.enable_advanced_shortcuts = false;
        changed = true;
        warn!("Global shortcuts unavailable; disabling shortcut integration.");
    }
    if !availability.hardware_acceleration && settings.hardware_acceleration {
        settings.hardware_acceleration = false;
        changed = true;
        warn!("Hardware acceleration unavailable; disabling GPU acceleration.");
    }
    if settings.now_playing_notifications && !settings.notifications_enabled {
        settings.now_playing_notifications = false;
        changed = true;
    }
    changed
}
