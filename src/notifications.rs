use crate::config::UserSettings;
use crate::player::NowPlaying;
use tracing::warn;

#[cfg(not(target_arch = "wasm32"))]
use notify_rust::Notification;

pub fn notify_now_playing(settings: &UserSettings, now_playing: &NowPlaying) -> bool {
    if !settings.notifications_enabled || !settings.now_playing_notifications {
        return false;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let body = format!("{} — {}", now_playing.artist, now_playing.album);
        let result = Notification::new()
            .appname("Grape")
            .summary(&now_playing.title)
            .body(&body)
            .show();

        if let Err(error) = result {
            warn!(error = %error, "Failed to send now playing notification");
            return false;
        }
        true
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = now_playing;
        false
    }
}
