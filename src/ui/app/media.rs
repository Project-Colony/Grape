use super::*;
use crate::system_integration::media::Command;

impl GrapeApp {
    pub(super) fn handle_media_command(&mut self, command: &Command) -> Task<UiMessage> {
        match command {
            Command::Play => {
                if let Some(player) = &mut self.player {
                    player.play();
                }
            }
            Command::Pause => {
                if let Some(player) = &mut self.player {
                    player.pause();
                }
            }
            Command::Toggle => self.handle_playback_message(&PlaybackMessage::TogglePlayPause),
            Command::Stop => {
                if let Some(player) = &mut self.player {
                    player.stop();
                }
                self.media.seeked();
            }
            Command::Next => self.handle_playback_message(&PlaybackMessage::NextTrack),
            Command::Previous => self.handle_playback_message(&PlaybackMessage::PreviousTrack),
            Command::Seek(offset) => {
                let position = self
                    .player
                    .as_ref()
                    .map_or(Duration::ZERO, Player::position);
                let amount = Duration::from_micros(offset.unsigned_abs());
                let target = if *offset < 0 {
                    position.saturating_sub(amount)
                } else {
                    position.saturating_add(amount)
                };
                self.seek_playback(target);
            }
            Command::SetPosition { track_id, position } => {
                // Recheck on the UI thread: the song may have changed while the
                // command waited in the queue.
                if self.media.accepts_position(track_id)
                    && self
                        .playing_track
                        .as_ref()
                        .is_some_and(|track| *position <= track.duration)
                {
                    self.seek_playback(*position);
                }
            }
            Command::Volume(volume) if volume.is_finite() => {
                return self.update(UiMessage::SetDefaultVolume(
                    (volume.clamp(0.0, 1.0) * 100.0).round() as u8,
                ));
            }
            Command::Rate(rate) if rate.is_finite() && (0.5..=2.0).contains(rate) => {
                return self.update(UiMessage::SetPlaybackSpeed((rate * 10.0).round() as u8));
            }
            Command::Raise => {
                return window::oldest().then(|id| {
                    id.map_or_else(Task::none, |id| {
                        Task::batch([window::minimize(id, false), window::gain_focus(id)])
                    })
                })
            }
            Command::Quit => {
                self.save_session_state();
                return iced::exit();
            }
            _ => {}
        }
        Task::none()
    }

    pub(super) fn seek_playback(&mut self, target: Duration) {
        let Some(track) = &self.playing_track else {
            return;
        };
        if track.duration.is_zero() {
            return;
        }
        let duration = track.duration;
        let target = target.min(duration);
        let Some(player) = &mut self.player else {
            return;
        };
        if let Err(error) = player.seek(target) {
            error!(%error, "Failed to seek");
            self.ui.error_message = Some("Seek not supported for this format".to_string());
            return;
        }
        self.media.seeked();
        self.ui.playback.position = target;
        self.ui.playback.animated_progress = progress_ratio(target, duration);
    }

    pub(super) fn publish_media_state(&self) {
        let status = self
            .player
            .as_ref()
            .map_or(PlayerPlaybackState::Stopped, Player::state);
        let position = self
            .player
            .as_ref()
            .map_or(Duration::ZERO, Player::position);
        self.media.publish(
            status,
            position,
            self.player.as_ref().map_or(1.0, Player::playback_rate),
            self.player.as_ref().map_or(0.0, Player::volume),
            self.ui.play_from_queue && self.playback_queue.peek_next().is_some(),
            self.ui.play_from_queue
                && !self.playback_queue.is_empty()
                && self.playback_queue.index() > 0,
        );
    }
}
