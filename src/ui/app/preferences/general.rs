use super::super::*;
use super::helpers::*;
use iced::widget::column;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InterfaceLanguageChoice {
    value: InterfaceLanguage,
    label: &'static str,
}

impl std::fmt::Display for InterfaceLanguageChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl GrapeApp {
    pub(super) fn general_preferences_panel(&self) -> Element<'_, UiMessage> {
        let language = self.language();
        let theme = self.theme_tokens();
        let strings = self.strings();

        let reindex_action = DeclarativeAction::ReindexLibrary;
        let clear_cache_action = DeclarativeAction::ClearCache;

        let language_choices = InterfaceLanguage::all()
            .iter()
            .map(|value| InterfaceLanguageChoice {
                value: *value,
                label: value.label(language),
            })
            .collect::<Vec<_>>();
        let selected_language = language_choices
            .iter()
            .find(|choice| choice.value == self.ui.settings.interface_language)
            .copied();

        let library_input =
            text_input(strings.library_folder_placeholder, &self.ui.settings.library_folder)
                .style(move |_, status| style::text_input_style(theme, status))
                .on_input(UiMessage::LibraryFolderChanged);
        let cache_input = text_input(strings.cache_path_placeholder, &self.ui.settings.cache_path)
            .style(move |_, status| style::text_input_style(theme, status))
            .on_input(UiMessage::CachePathChanged);

        let startup_content = || {
            column![
                section_hint(theme, strings.startup_hint),
                row![
                    setting_label(
                        theme,
                        strings.launch_at_startup_title,
                        strings.launch_at_startup_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.launch_at_startup,
                            UiMessage::SetLaunchAtStartup(true),
                            UiMessage::SetLaunchAtStartup(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.restore_last_session_title,
                        strings.restore_last_session_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.restore_last_session,
                            UiMessage::SetRestoreLastSession(true),
                            UiMessage::SetRestoreLastSession(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.open_on_title, strings.open_on_subtitle),
                    controls(
                        column![
                            row![
                                option_button(
                                    theme,
                                    self.ui.settings.open_on == StartupScreen::Home,
                                    StartupScreen::Home.label(language),
                                    UiMessage::SetOpenOn(StartupScreen::Home),
                                ),
                                option_button(
                                    theme,
                                    self.ui.settings.open_on == StartupScreen::Library,
                                    StartupScreen::Library.label(language),
                                    UiMessage::SetOpenOn(StartupScreen::Library),
                                ),
                            ]
                            .spacing(spacing::LG),
                            row![
                                option_button(
                                    theme,
                                    self.ui.settings.open_on == StartupScreen::Playlists,
                                    StartupScreen::Playlists.label(language),
                                    UiMessage::SetOpenOn(StartupScreen::Playlists),
                                ),
                                option_button(
                                    theme,
                                    self.ui.settings.open_on == StartupScreen::LastScreen,
                                    StartupScreen::LastScreen.label(language),
                                    UiMessage::SetOpenOn(StartupScreen::LastScreen),
                                ),
                            ]
                            .spacing(spacing::LG),
                        ]
                        .spacing(spacing::MD)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.close_behavior_title,
                        strings.close_behavior_subtitle
                    ),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.close_behavior == CloseBehavior::Quit,
                                CloseBehavior::Quit.label(language),
                                UiMessage::SetCloseBehavior(CloseBehavior::Quit),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.close_behavior == CloseBehavior::MinimizeToTray,
                                CloseBehavior::MinimizeToTray.label(language),
                                UiMessage::SetCloseBehavior(CloseBehavior::MinimizeToTray),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let language_content = || {
            column![
                section_hint(theme, strings.language_hint),
                row![
                    setting_label(
                        theme,
                        strings.interface_language_title,
                        strings.interface_language_subtitle
                    ),
                    controls(
                        pick_list(language_choices.clone(), selected_language, |choice| {
                            UiMessage::SetInterfaceLanguage(choice.value)
                        },)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.time_format_title, strings.time_format_subtitle),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.time_format == TimeFormat::H24,
                                TimeFormat::H24.label(language),
                                UiMessage::SetTimeFormat(TimeFormat::H24),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.time_format == TimeFormat::H12,
                                TimeFormat::H12.label(language),
                                UiMessage::SetTimeFormat(TimeFormat::H12),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let updates_content = || {
            column![
                section_hint(theme, strings.updates_hint),
                row![
                    setting_label(
                        theme,
                        strings.auto_check_updates_title,
                        strings.auto_check_updates_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.auto_check_updates,
                            UiMessage::SetAutoCheckUpdates(true),
                            UiMessage::SetAutoCheckUpdates(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.channel_title, strings.channel_subtitle),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.update_channel == UpdateChannel::Stable,
                                UpdateChannel::Stable.label(language),
                                UiMessage::SetUpdateChannel(UpdateChannel::Stable),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.update_channel == UpdateChannel::Beta,
                                UpdateChannel::Beta.label(language),
                                UiMessage::SetUpdateChannel(UpdateChannel::Beta),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.auto_install_updates_title,
                        strings.auto_install_updates_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.auto_install_updates,
                            UiMessage::SetAutoInstallUpdates(true),
                            UiMessage::SetAutoInstallUpdates(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let privacy_content = || {
            column![
                section_hint(theme, strings.privacy_hint),
                row![
                    setting_label(
                        theme,
                        strings.clear_history_title,
                        strings.clear_history_subtitle
                    ),
                    controls(
                        action_button(theme, strings.clear_history_button, UiMessage::ClearHistory)
                            .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let storage_content = || {
            column![
                section_hint(theme, strings.storage_hint),
                row![
                    setting_label(
                        theme,
                        strings.library_folder_title,
                        strings.library_folder_subtitle
                    ),
                    controls(
                        row![
                            library_input.width(Length::Fill),
                            action_button(
                                theme,
                                strings.add_folder_button,
                                UiMessage::PickLibraryFolder
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.auto_scan_title, strings.auto_scan_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.auto_scan_on_launch,
                            UiMessage::SetAutoScanOnLaunch(true),
                            UiMessage::SetAutoScanOnLaunch(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        clear_cache_action.title(language),
                        clear_cache_action.description(language)
                    ),
                    controls(
                        row![
                            cache_input.width(Length::Fill),
                            action_controls(
                                theme,
                                strings,
                                language,
                                self.ui.pending_action,
                                clear_cache_action
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let system_availability = self
            .system_integration
            .as_ref()
            .map(SystemIntegration::availability)
            .unwrap_or_else(SystemIntegrationAvailability::detect);
        let system_integration_content = || {
            column![
                section_hint(theme, strings.system_integration_hint),
                row![
                    setting_label(
                        theme,
                        strings.notifications_title,
                        strings.notifications_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.notifications_enabled,
                            UiMessage::SetNotificationsEnabled(true),
                            UiMessage::SetNotificationsEnabled(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.now_playing_title, strings.now_playing_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.now_playing_notifications,
                            UiMessage::SetNowPlayingNotifications(true),
                            UiMessage::SetNowPlayingNotifications(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.system_tray_title, strings.system_tray_subtitle),
                    controls(
                        toggle_row_with_support(
                            theme,
                            strings,
                            self.ui.settings.system_tray_enabled,
                            system_availability.tray,
                            UiMessage::SetSystemTrayEnabled(true),
                            UiMessage::SetSystemTrayEnabled(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.global_shortcuts_title,
                        strings.global_shortcuts_subtitle
                    ),
                    controls(
                        toggle_row_with_support(
                            theme,
                            strings,
                            self.ui.settings.enable_advanced_shortcuts,
                            system_availability.global_shortcuts,
                            UiMessage::SetAdvancedShortcuts(true),
                            UiMessage::SetAdvancedShortcuts(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let performance_content = || {
            column![
                section_hint(theme, strings.performance_hint),
                row![
                    setting_label(theme, strings.limit_cpu_title, strings.limit_cpu_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.limit_cpu_during_playback,
                            UiMessage::SetLimitCpuDuringPlayback(true),
                            UiMessage::SetLimitCpuDuringPlayback(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.hardware_accel_title,
                        strings.hardware_accel_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.hardware_acceleration,
                            UiMessage::SetHardwareAcceleration(true),
                            UiMessage::SetHardwareAcceleration(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let advanced_content = || {
            column![
                section_hint(theme, strings.advanced_hint),
                row![
                    setting_label(theme, strings.open_logs_title, strings.open_logs_subtitle),
                    controls(
                        action_button(theme, strings.open_button, UiMessage::OpenLogsFolder).into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        reindex_action.title(language),
                        reindex_action.description(language)
                    ),
                    controls(action_controls(
                        theme,
                        strings,
                        language,
                        self.ui.pending_action,
                        reindex_action,
                    )),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.reset_preferences_title,
                        strings.reset_preferences_subtitle
                    ),
                    controls(
                        action_button(theme, strings.reset_button, UiMessage::ResetPreferences)
                            .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let general_panel = scrollable(
            column![
                column![
                    text(strings.general_settings_title)
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.preferences_saved)
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(spacing::MD),
                section_header(
                    theme,
                    strings.section_startup,
                    self.ui.preferences_sections.startup,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Startup),
                ),
                if self.ui.preferences_sections.startup {
                    startup_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_language,
                    self.ui.preferences_sections.language,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Language),
                ),
                if self.ui.preferences_sections.language {
                    language_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_updates,
                    self.ui.preferences_sections.updates,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Updates),
                ),
                if self.ui.preferences_sections.updates {
                    updates_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_privacy,
                    self.ui.preferences_sections.privacy,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Privacy),
                ),
                if self.ui.preferences_sections.privacy {
                    privacy_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_storage,
                    self.ui.preferences_sections.storage,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Storage),
                ),
                if self.ui.preferences_sections.storage {
                    storage_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_system_integration,
                    self.ui.preferences_sections.system_integration,
                    UiMessage::TogglePreferencesSection(PreferencesSection::SystemIntegration),
                ),
                if self.ui.preferences_sections.system_integration {
                    system_integration_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_performance,
                    self.ui.preferences_sections.performance,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Performance),
                ),
                if self.ui.preferences_sections.performance {
                    performance_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.section_advanced,
                    self.ui.preferences_sections.advanced,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Advanced),
                ),
                if self.ui.preferences_sections.advanced {
                    advanced_content()
                } else {
                    column![]
                },
            ]
            .spacing(spacing::SECTION),
        )
        .on_scroll(|viewport| UiMessage::PreferencesScrolled {
            tab: PreferencesTab::General,
            offset_y: viewport.absolute_offset().y,
        })
        .id(Id::new(PREFERENCES_GENERAL_SCROLL_ID))
        .height(Length::Fill);

        general_panel.into()
    }
}
