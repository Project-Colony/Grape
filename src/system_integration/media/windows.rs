use super::*;
use ::windows::{
    core::{Result, HSTRING},
    Foundation::{TimeSpan, TypedEventHandler},
    Media::{
        MediaPlaybackStatus, MediaPlaybackType, PlaybackPositionChangeRequestedEventArgs,
        PlaybackRateChangeRequestedEventArgs, SystemMediaTransportControls,
        SystemMediaTransportControlsButton, SystemMediaTransportControlsButtonPressedEventArgs,
        SystemMediaTransportControlsTimelineProperties,
    },
    Storage::{StorageFile, Streams::RandomAccessStreamReference},
    Win32::{Foundation::HWND, System::WinRT::ISystemMediaTransportControlsInterop},
};

struct Controls {
    controls: SystemMediaTransportControls,
    button: Option<i64>,
    position: Option<i64>,
    rate: Option<i64>,
}

impl Drop for Controls {
    fn drop(&mut self) {
        if let Some(token) = self.button {
            let _ = self.controls.RemoveButtonPressed(token);
        }
        if let Some(token) = self.position {
            let _ = self.controls.RemovePlaybackPositionChangeRequested(token);
        }
        if let Some(token) = self.rate {
            let _ = self.controls.RemovePlaybackRateChangeRequested(token);
        }
        let _ = self.controls.SetIsEnabled(false);
    }
}

fn timespan(position: Duration) -> TimeSpan {
    TimeSpan {
        Duration: (position.as_nanos() / 100).min(i64::MAX as u128) as i64,
    }
}

impl Controls {
    fn new(hwnd: usize, state: watch::Receiver<State>, commands: CommandSender) -> Result<Self> {
        let factory: ISystemMediaTransportControlsInterop = ::windows::core::factory::<
            SystemMediaTransportControls,
            ISystemMediaTransportControlsInterop,
        >()?;
        // The handle comes from iced's live native window. The subscription owns
        // the SMTC registrations and removes them on cancellation/window shutdown.
        let controls = unsafe { factory.GetForWindow(HWND(hwnd as *mut _)) }?;
        let mut this = Self {
            controls,
            button: None,
            position: None,
            rate: None,
        };
        let buttons = commands.clone();
        this.button = Some(this.controls.ButtonPressed(&TypedEventHandler::<
            SystemMediaTransportControls,
            SystemMediaTransportControlsButtonPressedEventArgs,
        >::new(move |_, args| {
            let Some(args) = args.as_ref() else {
                return Ok(());
            };
            let command = match args.Button()? {
                SystemMediaTransportControlsButton::Play => Command::Play,
                SystemMediaTransportControlsButton::Pause => Command::Pause,
                SystemMediaTransportControlsButton::Stop => Command::Stop,
                SystemMediaTransportControlsButton::Next => Command::Next,
                SystemMediaTransportControlsButton::Previous => Command::Previous,
                SystemMediaTransportControlsButton::FastForward => Command::Seek(10_000_000),
                SystemMediaTransportControlsButton::Rewind => Command::Seek(-10_000_000),
                _ => return Ok(()),
            };
            send(&buttons, command);
            Ok(())
        }))?);
        let positions = commands.clone();
        this.position = Some(this.controls.PlaybackPositionChangeRequested(
            &TypedEventHandler::<
                SystemMediaTransportControls,
                PlaybackPositionChangeRequestedEventArgs,
            >::new(move |_, args| {
                let Some(args) = args.as_ref() else {
                    return Ok(());
                };
                let ticks = args.RequestedPlaybackPosition()?.Duration;
                let state = state.borrow();
                if ticks >= 0 && state.can_seek() {
                    if let Some(track) = &state.track {
                        send(
                            &positions,
                            Command::SetPosition {
                                track_id: track.id.clone(),
                                position: Duration::from_nanos((ticks as u64).saturating_mul(100)),
                            },
                        );
                    }
                }
                Ok(())
            }),
        )?);
        this.rate = Some(
            this.controls
                .PlaybackRateChangeRequested(&TypedEventHandler::<
                    SystemMediaTransportControls,
                    PlaybackRateChangeRequestedEventArgs,
                >::new(move |_, args| {
                    if let Some(args) = args.as_ref() {
                        send(&commands, Command::Rate(args.RequestedPlaybackRate()?));
                    }
                    Ok(())
                }))?,
        );
        Ok(this)
    }

    async fn publish(&self, old: Option<&State>, state: &State) -> Result<()> {
        let controls = &self.controls;
        if old.is_none_or(|old| old.track.is_some() != state.track.is_some()) {
            controls.SetIsEnabled(state.track.is_some())?;
            controls.SetIsPlayEnabled(state.track.is_some())?;
            controls.SetIsPauseEnabled(state.track.is_some())?;
            controls.SetIsStopEnabled(state.track.is_some())?;
        }
        if old.is_none_or(|old| old.status != state.status) {
            controls.SetPlaybackStatus(match state.status {
                PlaybackState::Playing => MediaPlaybackStatus::Playing,
                PlaybackState::Paused => MediaPlaybackStatus::Paused,
                PlaybackState::Stopped => MediaPlaybackStatus::Stopped,
            })?;
        }
        if old.is_none_or(|old| old.rate != state.rate) {
            controls.SetPlaybackRate(state.rate)?;
        }
        if old.is_none_or(|old| old.can_next != state.can_next) {
            controls.SetIsNextEnabled(state.can_next)?;
        }
        if old.is_none_or(|old| old.can_previous != state.can_previous) {
            controls.SetIsPreviousEnabled(state.can_previous)?;
        }
        if old.is_none_or(|old| old.can_seek() != state.can_seek()) {
            controls.SetIsFastForwardEnabled(state.can_seek())?;
            controls.SetIsRewindEnabled(state.can_seek())?;
        }
        let track_changed = old.is_none_or(|old| old.track != state.track);
        if track_changed {
            let updater = controls.DisplayUpdater()?;
            updater.ClearAll()?;
            updater.SetType(MediaPlaybackType::Music)?;
            if let Some(track) = &state.track {
                let music = updater.MusicProperties()?;
                music.SetTitle(&HSTRING::from(&track.title))?;
                music.SetArtist(&HSTRING::from(&track.artist))?;
                music.SetAlbumTitle(&HSTRING::from(&track.album))?;
                // File access happens only on a track change, off the UI thread.
                // A missing cover must not prevent metadata and controls working.
                if let Some(path) = &track.cover_path {
                    if let Ok(path) = std::path::absolute(path) {
                        if let Ok(operation) =
                            StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))
                        {
                            if let Ok(file) = operation.await {
                                if let Ok(image) =
                                    RandomAccessStreamReference::CreateFromFile(&file)
                                {
                                    let _ = updater.SetThumbnail(&image);
                                }
                            }
                        }
                    }
                }
            }
            updater.Update()?;
        }
        // Windows interpolates between anchors. Refresh at most once per second
        // during normal playback, immediately after seek/status/rate/track changes.
        if old.is_none_or(|old| {
            track_changed
                || old.status != state.status
                || old.rate != state.rate
                || old.seek_serial != state.seek_serial
                || old.position.as_secs() != state.position.as_secs()
        }) {
            let timeline = SystemMediaTransportControlsTimelineProperties::new()?;
            let end = timespan(
                state
                    .track
                    .as_ref()
                    .map_or(Duration::ZERO, |track| track.duration),
            );
            timeline.SetStartTime(TimeSpan::default())?;
            timeline.SetMinSeekTime(TimeSpan::default())?;
            timeline.SetEndTime(end)?;
            timeline.SetMaxSeekTime(end)?;
            timeline.SetPosition(timespan(state.position()))?;
            controls.UpdateTimelineProperties(&timeline)?;
        }
        Ok(())
    }
}

pub(super) async fn run(
    hwnd: usize,
    mut state: watch::Receiver<State>,
    commands: CommandSender,
) -> Result<()> {
    let controls = Controls::new(hwnd, state.clone(), commands)?;
    let mut previous = None;
    loop {
        let current = state.borrow_and_update().clone();
        controls.publish(previous.as_ref(), &current).await?;
        previous = Some(current);
        if state.changed().await.is_err() {
            break;
        }
    }
    Ok(())
}
