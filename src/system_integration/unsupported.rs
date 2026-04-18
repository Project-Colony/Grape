use super::{AppInfo, IntegrationError, SystemAction, SystemIntegrationAvailability};

pub struct PlatformIntegration {
    _app_info: AppInfo,
}

impl PlatformIntegration {
    pub fn new(app_info: AppInfo) -> Self {
        Self { _app_info: app_info }
    }

    pub fn availability() -> SystemIntegrationAvailability {
        SystemIntegrationAvailability {
            notifications: false,
            tray: false,
            global_shortcuts: false,
            hardware_acceleration: false,
        }
    }

    pub fn apply_launch_at_startup(&self, _enabled: bool) -> Result<(), IntegrationError> {
        Ok(())
    }

    pub fn set_tray(&mut self, _enabled: bool) -> Result<(), IntegrationError> {
        Ok(())
    }

    pub fn set_global_shortcuts(&mut self, _enabled: bool) -> Result<(), IntegrationError> {
        Ok(())
    }

    pub fn drain_actions(&mut self) -> Vec<SystemAction> {
        Vec::new()
    }
}
