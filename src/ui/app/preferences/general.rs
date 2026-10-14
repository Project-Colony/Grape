use super::super::*;
use super::helpers::*;
use colony_ui::widgets as kit;
use iced::widget::Column;

impl GrapeApp {
    /// Colony's General heading, then Grape's own sections.
    pub(super) fn general_preferences(&self) -> Column<'_, UiMessage> {
        let typo = self.typography();
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();
        let settings = &self.ui.settings;
        let open = &self.ui.preferences_sections;
        let reindex = DeclarativeAction::ReindexLibrary;
        let clear_cache = DeclarativeAction::ClearCache;

        let startup = section(
            &typo,
            open,
            "startup",
            strings.preferences_section_startup,
            strings.preferences_startup_desc,
            [
                kit::functional_toggle(
                    &typo,
                    strings.preferences_launch_at_startup,
                    strings.preferences_launch_at_startup_desc,
                    settings.launch_at_startup,
                    UiMessage::SetLaunchAtStartup(!settings.launch_at_startup),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_restore_last_session,
                    strings.preferences_restore_last_session_desc,
                    settings.restore_last_session,
                    UiMessage::SetRestoreLastSession(!settings.restore_last_session),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_open_on,
                    strings.preferences_open_on_desc,
                    choice(
                        &typo,
                        [
                            StartupScreen::Home,
                            StartupScreen::Library,
                            StartupScreen::Playlists,
                            StartupScreen::LastScreen,
                        ],
                        |screen| screen.label(language),
                        &settings.open_on,
                        UiMessage::SetOpenOn,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_close_behavior,
                    strings.preferences_close_behavior_desc,
                    choice(
                        &typo,
                        [CloseBehavior::Quit, CloseBehavior::MinimizeToTray],
                        |behavior| behavior.label(language),
                        &settings.close_behavior,
                        UiMessage::SetCloseBehavior,
                    ),
                ),
            ],
        );

        let language_section = section(
            &typo,
            open,
            "language",
            strings.preferences_section_language,
            strings.preferences_language_desc,
            [
                kit::setting_row(
                    &typo,
                    strings.preferences_interface_language,
                    strings.preferences_interface_language_desc,
                    choice(
                        &typo,
                        *InterfaceLanguage::all(),
                        |value| value.label(language),
                        &settings.interface_language,
                        UiMessage::SetInterfaceLanguage,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_time_format,
                    strings.preferences_time_format_desc,
                    choice(
                        &typo,
                        [TimeFormat::H24, TimeFormat::H12],
                        |format| format.label(language),
                        &settings.time_format,
                        UiMessage::SetTimeFormat,
                    ),
                ),
            ],
        );

        let privacy = section(
            &typo,
            open,
            "privacy",
            strings.preferences_section_privacy,
            strings.preferences_privacy_desc,
            [kit::setting_row(
                &typo,
                strings.preferences_clear_history,
                strings.preferences_clear_history_desc,
                action_button(
                    theme,
                    strings.preferences_clear_history_button,
                    UiMessage::ClearHistory,
                ),
            )],
        );

        let library_input =
            text_input(strings.preferences_library_folder_placeholder, &settings.library_folder)
                .style(move |_, status| style::text_input_style(theme, status))
                .font(colony_ui::ui_font())
                .size(sz(13))
                .on_input(UiMessage::LibraryFolderChanged);
        let cache_input =
            text_input(strings.preferences_cache_path_placeholder, &settings.cache_path)
                .style(move |_, status| style::text_input_style(theme, status))
                .font(colony_ui::ui_font())
                .size(sz(13))
                .on_input(UiMessage::CachePathChanged);
        let storage = section(
            &typo,
            open,
            "storage",
            strings.preferences_section_storage,
            strings.preferences_storage_desc,
            [
                kit::setting_row(
                    &typo,
                    strings.preferences_library_folder,
                    strings.preferences_library_folder_desc,
                    row![
                        library_input.width(Length::Fill),
                        action_button(
                            theme,
                            strings.preferences_add_folder_button,
                            UiMessage::PickLibraryFolder
                        ),
                    ]
                    .spacing(spacing::LG)
                    .align_y(Alignment::Center),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_auto_scan,
                    strings.preferences_auto_scan_desc,
                    settings.auto_scan_on_launch,
                    UiMessage::SetAutoScanOnLaunch(!settings.auto_scan_on_launch),
                ),
                kit::setting_row(
                    &typo,
                    clear_cache.title(language),
                    clear_cache.description(language),
                    row![
                        cache_input.width(Length::Fill),
                        action_controls(
                            theme,
                            strings,
                            language,
                            self.ui.pending_action,
                            clear_cache
                        ),
                    ]
                    .spacing(spacing::LG)
                    .align_y(Alignment::Center),
                ),
            ],
        );

        let availability = self
            .system_integration
            .as_ref()
            .map(SystemIntegration::availability)
            .unwrap_or_else(SystemIntegrationAvailability::detect);
        // A toggle the platform cannot honour is replaced by saying so, rather
        // than drawn and then switched back off behind the user's back.
        let supported_toggle = |supported: bool,
                                title: &'static str,
                                description: &'static str,
                                on: bool,
                                message: UiMessage| {
            if supported {
                kit::functional_toggle(&typo, title, description, on, message)
            } else {
                kit::setting_row(
                    &typo,
                    title,
                    description,
                    text(strings.preferences_unavailable)
                        .size(sz(12))
                        .font(colony_ui::ui_font())
                        .style(move |_| style::text_style_muted(theme)),
                )
            }
        };
        let system_integration = section(
            &typo,
            open,
            "system_integration",
            strings.preferences_section_system_integration,
            strings.preferences_system_integration_desc,
            [
                kit::functional_toggle(
                    &typo,
                    strings.preferences_notifications,
                    strings.preferences_notifications_desc,
                    settings.notifications_enabled,
                    UiMessage::SetNotificationsEnabled(!settings.notifications_enabled),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_now_playing,
                    strings.preferences_now_playing_desc,
                    settings.now_playing_notifications,
                    UiMessage::SetNowPlayingNotifications(!settings.now_playing_notifications),
                ),
                supported_toggle(
                    availability.tray,
                    strings.preferences_system_tray,
                    strings.preferences_system_tray_desc,
                    settings.system_tray_enabled,
                    UiMessage::SetSystemTrayEnabled(!settings.system_tray_enabled),
                ),
                supported_toggle(
                    availability.global_shortcuts,
                    strings.preferences_global_shortcuts,
                    strings.preferences_global_shortcuts_desc,
                    settings.enable_advanced_shortcuts,
                    UiMessage::SetAdvancedShortcuts(!settings.enable_advanced_shortcuts),
                ),
            ],
        );

        let performance = section(
            &typo,
            open,
            "performance",
            strings.preferences_section_performance,
            strings.preferences_performance_desc,
            [
                kit::functional_toggle(
                    &typo,
                    strings.preferences_limit_cpu,
                    strings.preferences_limit_cpu_desc,
                    settings.limit_cpu_during_playback,
                    UiMessage::SetLimitCpuDuringPlayback(!settings.limit_cpu_during_playback),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_hardware_accel,
                    strings.preferences_hardware_accel_desc,
                    settings.hardware_acceleration,
                    UiMessage::SetHardwareAcceleration(!settings.hardware_acceleration),
                ),
            ],
        );

        let advanced = section(
            &typo,
            open,
            "advanced",
            strings.preferences_section_advanced,
            strings.preferences_advanced_desc,
            [
                kit::setting_row(
                    &typo,
                    strings.preferences_open_logs,
                    strings.preferences_open_logs_desc,
                    action_button(
                        theme,
                        strings.preferences_open_button,
                        UiMessage::OpenLogsFolder,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    reindex.title(language),
                    reindex.description(language),
                    action_controls(theme, strings, language, self.ui.pending_action, reindex),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_reset,
                    strings.preferences_reset_desc,
                    action_button(
                        theme,
                        strings.preferences_reset_button,
                        UiMessage::ResetPreferences,
                    ),
                ),
            ],
        );

        self.kit_preferences()
            .general()
            .push(startup)
            .push(language_section)
            .push(privacy)
            .push(storage)
            .push(system_integration)
            .push(performance)
            .push(advanced)
    }
}
