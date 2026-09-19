//! Native media sessions. One latest-state slot, bounded commands, no idle polling.
// Other platforms retain the shared state API until a native backend exists.
#![cfg_attr(not(any(target_os = "linux", target_os = "windows")), allow(dead_code))]
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::{futures::channel::mpsc, stream, Subscription};
use tokio::sync::watch;

use crate::player::PlaybackState;
use crate::ui::state::Track as UiTrack;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Play,
    Pause,
    Toggle,
    Stop,
    Next,
    Previous,
    Seek(i64),
    SetPosition {
        track_id: String,
        position: Duration,
    },
    Volume(f64),
    Rate(f64),
    Raise,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Track {
    id: String,
    path: PathBuf,
    title: String,
    artist: String,
    album: String,
    cover_path: Option<PathBuf>,
    cover_url: String,
    url: String,
    duration: Duration,
}

impl Track {
    fn new(track: &UiTrack, generation: u64) -> Self {
        Self {
            // A new playback instance gets a new id, including replays of the same file.
            id: format!("/org/mpris/MediaPlayer2/track/{generation}"),
            path: track.path.clone(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            cover_path: track.cover_path.clone(),
            cover_url: track
                .cover_path
                .as_deref()
                .map(file_url)
                .unwrap_or_default(),
            url: file_url(&track.path),
            duration: track.duration,
        }
    }
}

fn file_url(path: &Path) -> String {
    // URL handles spaces, Unicode, #, % and Windows drive letters correctly.
    std::path::absolute(path)
        .ok()
        .and_then(|path| reqwest::Url::from_file_path(path).ok())
        .map(|url| url.to_string())
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    track: Option<Arc<Track>>,
    status: PlaybackState,
    position: Duration,
    updated_at: Instant,
    rate: f64,
    volume: f64,
    can_next: bool,
    can_previous: bool,
    seek_serial: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            track: None,
            status: PlaybackState::Stopped,
            position: Duration::ZERO,
            updated_at: Instant::now(),
            rate: 1.0,
            volume: 1.0,
            can_next: false,
            can_previous: false,
            seek_serial: 0,
        }
    }
}

impl State {
    fn position(&self) -> Duration {
        let elapsed = if self.status == PlaybackState::Playing {
            self.updated_at.elapsed().mul_f64(self.rate)
        } else {
            Duration::ZERO
        };
        let position = self.position.saturating_add(elapsed);
        self.track
            .as_ref()
            .filter(|track| !track.duration.is_zero())
            .map_or(position, |track| position.min(track.duration))
    }

    fn can_seek(&self) -> bool {
        self.track
            .as_ref()
            .is_some_and(|track| !track.duration.is_zero())
    }
}

pub(crate) struct Session {
    state: watch::Sender<State>,
    generation: u64,
    track: Option<Arc<Track>>,
    seek_serial: u64,
    pub window_handle: Option<usize>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            state: watch::channel(State::default()).0,
            generation: 0,
            track: None,
            seek_serial: 0,
            window_handle: if cfg!(target_os = "windows") {
                None
            } else {
                Some(0)
            },
        }
    }
}

impl Session {
    pub fn track_changed(&mut self, track: Option<&UiTrack>) {
        self.generation = self.generation.wrapping_add(1);
        self.track = track.map(|track| Arc::new(Track::new(track, self.generation)));
    }

    pub fn seeked(&mut self) {
        self.seek_serial = self.seek_serial.wrapping_add(1);
    }

    pub fn accepts_position(&self, id: &str) -> bool {
        self.track.as_ref().is_some_and(|track| track.id == id)
    }

    pub fn publish(
        &self,
        status: PlaybackState,
        position: Duration,
        rate: f64,
        volume: f64,
        can_next: bool,
        can_previous: bool,
    ) {
        self.state.send_if_modified(|state| {
            if state.track == self.track
                && state.seek_serial == self.seek_serial
                && state.status == status
                && state.position == position
                && state.rate == rate
                && state.volume == volume
                && state.can_next == can_next
                && state.can_previous == can_previous
            {
                return false;
            }
            state.track.clone_from(&self.track);
            state.seek_serial = self.seek_serial;
            state.status = status;
            state.position = position;
            state.updated_at = Instant::now();
            state.rate = rate;
            state.volume = volume;
            state.can_next = can_next;
            state.can_previous = can_previous;
            true
        });
    }

    pub fn subscription(&self) -> Subscription<Command> {
        if !cfg!(any(target_os = "linux", target_os = "windows")) {
            return Subscription::none();
        }
        let Some(hwnd) = self.window_handle else {
            return Subscription::none();
        };
        Subscription::run_with(
            Input {
                hwnd,
                state: self.state.subscribe(),
            },
            |input| {
                let state = input.state.clone();
                let hwnd = input.hwnd;
                stream::channel(32, move |commands| async move {
                    let commands = Arc::new(Mutex::new(commands));
                    #[cfg(target_os = "linux")]
                    let result = linux::run(state, commands).await;
                    #[cfg(target_os = "windows")]
                    let result = windows::run(hwnd, state, commands).await;
                    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                    let result: Result<(), String> = {
                        let _ = (state, commands);
                        Ok(())
                    };
                    let _ = hwnd;
                    if let Err(error) = result {
                        tracing::warn!(%error, "System media controls unavailable");
                    }
                    // Keep a failed subscription alive; avoid a busy restart/log loop.
                    std::future::pending::<()>().await;
                })
            },
        )
    }
}

struct Input {
    hwnd: usize,
    state: watch::Receiver<State>,
}
impl Hash for Input {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.hwnd.hash(hasher);
    }
}

type CommandSender = Arc<Mutex<mpsc::Sender<Command>>>;

fn send(commands: &CommandSender, command: Command) {
    // Never block an OS callback. The bounded channel limits a misbehaving client's memory use.
    let _ = commands.lock().unwrap().try_send(command);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn track() -> UiTrack {
        UiTrack {
            id: 0,
            title: "Track".into(),
            artist: "Artist".into(),
            album: "Album".into(),
            path: PathBuf::from("/music/é #100%.flac"),
            cover_path: Some(PathBuf::from("/art/cover #1.png")),
            duration: Duration::from_secs(120),
            track_number: Some(1),
            normalized_title: String::new(),
            normalized_artist: String::new(),
            normalized_album: String::new(),
        }
    }

    #[test]
    fn media_updates_share_metadata_and_reject_old_track_ids() {
        let mut session = Session::default();
        session.track_changed(Some(&track()));
        session.publish(
            PlaybackState::Playing,
            Duration::ZERO,
            1.0,
            0.5,
            true,
            false,
        );
        let original = session.state.borrow().track.clone().unwrap();
        session.publish(
            PlaybackState::Playing,
            Duration::from_secs(1),
            1.0,
            0.5,
            true,
            false,
        );
        assert!(Arc::ptr_eq(
            &original,
            session.state.borrow().track.as_ref().unwrap()
        ));
        assert!(session.accepts_position(&original.id));
        session.track_changed(Some(&track()));
        assert!(!session.accepts_position(&original.id));
        session.track_changed(None);
        session.publish(
            PlaybackState::Stopped,
            Duration::ZERO,
            1.0,
            0.5,
            false,
            false,
        );
        assert!(session.state.borrow().track.is_none());
    }

    #[test]
    fn paused_session_does_not_wake_worker_and_cover_urls_are_encoded() {
        let session = Session::default();
        let mut receiver = session.state.subscribe();
        session.publish(
            PlaybackState::Paused,
            Duration::from_secs(4),
            1.0,
            0.5,
            false,
            false,
        );
        receiver.borrow_and_update();
        session.publish(
            PlaybackState::Paused,
            Duration::from_secs(4),
            1.0,
            0.5,
            false,
            false,
        );
        assert!(!receiver.has_changed().unwrap());
        #[cfg(unix)]
        assert_eq!(
            file_url(Path::new("/art/été #100%.png")),
            "file:///art/%C3%A9t%C3%A9%20%23100%25.png"
        );
    }

    #[test]
    fn os_command_backlog_is_bounded() {
        let (tx, mut rx) = mpsc::channel(32);
        let tx = Arc::new(Mutex::new(tx));
        for _ in 0..10_000 {
            send(&tx, Command::Play);
        }
        let mut received = 0;
        while rx.try_recv().is_ok() {
            received += 1;
        }
        assert!(received > 0 && received <= 33, "{received}");
    }

    #[test]
    fn position_uses_rate_and_clamps_at_end() {
        let state = State {
            track: Some(Arc::new(Track::new(&track(), 1))),
            status: PlaybackState::Playing,
            position: Duration::from_secs(10),
            updated_at: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
            rate: 2.0,
            ..State::default()
        };
        assert!((14.0..14.5).contains(&state.position().as_secs_f64()));
        let paused = State {
            status: PlaybackState::Paused,
            ..state.clone()
        };
        assert_eq!(paused.position(), Duration::from_secs(10));
        let ended = State {
            position: Duration::from_secs(119),
            ..state
        };
        assert_eq!(ended.position(), Duration::from_secs(120));
    }
}
