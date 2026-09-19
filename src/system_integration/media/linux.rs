use std::collections::HashMap;

use super::*;
use zbus::{
    fdo, interface,
    zvariant::{ObjectPath, OwnedValue, Value},
    Connection,
};

const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";

struct Root {
    commands: CommandSender,
}

#[interface(name = "org.mpris.MediaPlayer2")]
impl Root {
    fn raise(&self) {
        send(&self.commands, Command::Raise);
    }
    fn quit(&self) {
        send(&self.commands, Command::Quit);
    }
    #[zbus(property)]
    fn can_quit(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn identity(&self) -> &str {
        "Grape"
    }
    #[zbus(property)]
    fn desktop_entry(&self) -> &str {
        "grape"
    }
    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        Vec::new()
    }
    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        Vec::new()
    }
}

struct Player {
    state: watch::Receiver<State>,
    commands: CommandSender,
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
impl Player {
    fn next(&self) {
        send(&self.commands, Command::Next);
    }
    fn previous(&self) {
        send(&self.commands, Command::Previous);
    }
    fn pause(&self) {
        send(&self.commands, Command::Pause);
    }
    fn play_pause(&self) {
        send(&self.commands, Command::Toggle);
    }
    fn stop(&self) {
        send(&self.commands, Command::Stop);
    }
    fn play(&self) {
        send(&self.commands, Command::Play);
    }
    fn seek(&self, offset: i64) {
        send(&self.commands, Command::Seek(offset));
    }
    fn set_position(&self, track_id: ObjectPath<'_>, position: i64) {
        let state = self.state.borrow();
        if position < 0 || !state.can_seek() {
            return;
        }
        let Some(track) = &state.track else {
            return;
        };
        if track.id != track_id.as_str() || Duration::from_micros(position as u64) > track.duration
        {
            return;
        }
        send(
            &self.commands,
            Command::SetPosition {
                track_id: track.id.clone(),
                position: Duration::from_micros(position as u64),
            },
        );
    }
    fn open_uri(&self, _uri: &str) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Open tracks from Grape's library".into(),
        ))
    }
    #[zbus(property)]
    fn playback_status(&self) -> &str {
        status(self.state.borrow().status)
    }
    #[zbus(property)]
    fn rate(&self) -> f64 {
        self.state.borrow().rate
    }
    #[zbus(property)]
    fn set_rate(&self, rate: f64) -> fdo::Result<()> {
        if !rate.is_finite() || !(0.5..=2.0).contains(&rate) {
            return Err(fdo::Error::InvalidArgs(
                "Rate must be between 0.5 and 2.0".into(),
            ));
        }
        send(&self.commands, Command::Rate(rate));
        Ok(())
    }
    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        0.5
    }
    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        2.0
    }
    #[zbus(property)]
    fn volume(&self) -> f64 {
        self.state.borrow().volume
    }
    #[zbus(property)]
    fn set_volume(&self, volume: f64) -> fdo::Result<()> {
        if !volume.is_finite() {
            return Err(fdo::Error::InvalidArgs("Volume must be finite".into()));
        }
        send(&self.commands, Command::Volume(volume.clamp(0.0, 1.0)));
        Ok(())
    }
    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, OwnedValue> {
        metadata(&self.state.borrow())
    }
    #[zbus(property(emits_changed_signal = "false"))]
    fn position(&self) -> i64 {
        micros(self.state.borrow().position())
    }
    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        self.state.borrow().can_next
    }
    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        self.state.borrow().can_previous
    }
    #[zbus(property)]
    fn can_play(&self) -> bool {
        self.state.borrow().track.is_some()
    }
    #[zbus(property)]
    fn can_pause(&self) -> bool {
        self.state.borrow().track.is_some()
    }
    #[zbus(property)]
    fn can_seek(&self) -> bool {
        self.state.borrow().can_seek()
    }
    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }
}

fn micros(time: Duration) -> i64 {
    time.as_micros().min(i64::MAX as u128) as i64
}
fn status(status: PlaybackState) -> &'static str {
    match status {
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
        PlaybackState::Stopped => "Stopped",
    }
}

fn metadata(state: &State) -> HashMap<String, OwnedValue> {
    let mut values = HashMap::new();
    let Some(track) = &state.track else {
        return values;
    };
    let mut insert = |key: &str, value: Value<'_>| {
        // All values here are owned scalars/arrays, never file descriptors.
        values.insert(
            key.to_string(),
            value.try_to_owned().expect("owned metadata"),
        );
    };
    insert(
        "mpris:trackid",
        ObjectPath::try_from(track.id.as_str()).unwrap().into(),
    );
    insert("xesam:title", track.title.as_str().into());
    insert("xesam:artist", vec![track.artist.as_str()].into());
    insert("xesam:album", track.album.as_str().into());
    insert("xesam:url", track.url.as_str().into());
    if !track.duration.is_zero() {
        insert("mpris:length", micros(track.duration).into());
    }
    if !track.cover_url.is_empty() {
        insert("mpris:artUrl", track.cover_url.as_str().into());
    }
    values
}

fn changes<'a>(old: &State, new: &'a State) -> HashMap<&'static str, Value<'a>> {
    let mut changes = HashMap::new();
    if old.track != new.track {
        changes.insert("Metadata", metadata(new).into());
    }
    if old.status != new.status {
        changes.insert("PlaybackStatus", status(new.status).into());
    }
    if old.rate != new.rate {
        changes.insert("Rate", new.rate.into());
    }
    if old.volume != new.volume {
        changes.insert("Volume", new.volume.into());
    }
    if old.can_next != new.can_next {
        changes.insert("CanGoNext", new.can_next.into());
    }
    if old.can_previous != new.can_previous {
        changes.insert("CanGoPrevious", new.can_previous.into());
    }
    if old.track.is_some() != new.track.is_some() {
        changes.insert("CanPlay", new.track.is_some().into());
        changes.insert("CanPause", new.track.is_some().into());
    }
    if old.can_seek() != new.can_seek() {
        changes.insert("CanSeek", new.can_seek().into());
    }
    changes
}

pub(super) async fn run(
    mut state: watch::Receiver<State>,
    commands: CommandSender,
) -> zbus::Result<()> {
    let mut previous = state.borrow_and_update().clone();
    let connection = zbus::connection::Builder::session()?
        .name(format!(
            "org.mpris.MediaPlayer2.grape.instance{}",
            std::process::id()
        ))?
        .serve_at(
            PATH,
            Root {
                commands: commands.clone(),
            },
        )?
        .serve_at(
            PATH,
            Player {
                state: state.clone(),
                commands,
            },
        )?
        .build()
        .await?;
    while state.changed().await.is_ok() {
        let current = state.borrow_and_update().clone();
        publish(&connection, &previous, &current).await?;
        previous = current;
    }
    Ok(())
}

async fn publish(connection: &Connection, old: &State, new: &State) -> zbus::Result<()> {
    let changed = changes(old, new);
    if !changed.is_empty() {
        connection
            .emit_signal(
                None::<&str>,
                PATH,
                "org.freedesktop.DBus.Properties",
                "PropertiesChanged",
                &(PLAYER, changed, Vec::<String>::new()),
            )
            .await?;
    }
    if old.seek_serial != new.seek_serial {
        connection
            .emit_signal(
                None::<&str>,
                PATH,
                PLAYER,
                "Seeked",
                &(micros(new.position()),),
            )
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::futures::StreamExt;

    #[test]
    fn progress_does_not_emit_metadata_or_position_properties() {
        let old = State {
            track: Some(Arc::new(Track::new(&super::super::tests::track(), 1))),
            ..State::default()
        };
        let new = State {
            position: Duration::from_secs(1),
            ..old.clone()
        };
        assert!(changes(&old, &new).is_empty());
        let mut without_cover = (*old.track.clone().unwrap()).clone();
        without_cover.cover_url.clear();
        let changed = State {
            track: Some(Arc::new(without_cover)),
            ..new
        };
        assert!(changes(&old, &changed).contains_key("Metadata"));
        assert!(!metadata(&changed).contains_key("mpris:artUrl"));
        assert_eq!(
            i64::try_from(metadata(&old).remove("mpris:length").unwrap()).unwrap(),
            120_000_000
        );
    }

    #[test]
    #[ignore = "run under dbus-run-session; uses an isolated session bus"]
    fn mpris_round_trip() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut session = Session::default();
            session.track_changed(Some(&super::super::tests::track()));
            session.publish(
                PlaybackState::Paused,
                Duration::from_secs(5),
                1.0,
                0.5,
                true,
                false,
            );
            let (commands, mut received) = mpsc::channel(32);
            let server = tokio::spawn(run(
                session.state.subscribe(),
                Arc::new(Mutex::new(commands)),
            ));
            let connection = Connection::session().await.unwrap();
            let name = format!(
                "org.mpris.MediaPlayer2.grape.instance{}",
                std::process::id()
            );
            let proxy = zbus::Proxy::new(&connection, name.as_str(), PATH, PLAYER)
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    if proxy.get_property::<String>("PlaybackStatus").await.is_ok() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .unwrap();
            assert_eq!(
                proxy
                    .get_property::<String>("PlaybackStatus")
                    .await
                    .unwrap(),
                "Paused"
            );
            assert_eq!(
                proxy.get_property::<i64>("Position").await.unwrap(),
                5_000_000
            );
            assert!(!proxy.get_property::<bool>("CanGoPrevious").await.unwrap());
            let metadata: HashMap<String, OwnedValue> =
                proxy.get_property("Metadata").await.unwrap();
            assert_eq!(
                String::try_from(metadata["xesam:title"].try_clone().unwrap()).unwrap(),
                "Track"
            );
            assert_eq!(
                String::try_from(metadata["mpris:artUrl"].try_clone().unwrap()).unwrap(),
                "file:///art/cover%20%231.png"
            );
            // Commands must arrive while paused, with no UI timer running.
            proxy.call::<_, _, ()>("Play", &()).await.unwrap();
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(2), received.next())
                    .await
                    .unwrap(),
                Some(Command::Play)
            );
            let id = session.track.as_ref().unwrap().id.clone();
            proxy
                .call::<_, _, ()>(
                    "SetPosition",
                    &(ObjectPath::try_from(id.as_str()).unwrap(), 9_000_000i64),
                )
                .await
                .unwrap();
            assert_eq!(
                received.next().await.unwrap(),
                Command::SetPosition {
                    track_id: id,
                    position: Duration::from_secs(9)
                }
            );
            proxy
                .call::<_, _, ()>(
                    "SetPosition",
                    &(ObjectPath::try_from("/stale").unwrap(), 10i64),
                )
                .await
                .unwrap();
            assert!(received.try_recv().is_err());
            assert!(proxy.set_property("Rate", f64::NAN).await.is_err());
            proxy.set_property("Volume", 0.25f64).await.unwrap();
            assert_eq!(received.next().await.unwrap(), Command::Volume(0.25));
            let mut seeks = proxy.receive_signal("Seeked").await.unwrap();
            session.seeked();
            session.publish(
                PlaybackState::Paused,
                Duration::from_secs(9),
                1.0,
                0.25,
                true,
                false,
            );
            let signal = tokio::time::timeout(Duration::from_secs(2), seeks.next())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(signal.body().deserialize::<(i64,)>().unwrap(), (9_000_000,));
            server.abort();
            let _ = server.await;
        });
    }
}
