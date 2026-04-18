#![allow(dead_code)]
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::config_root;
use crate::player::NowPlaying;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlaybackQueue {
    items: Vec<NowPlaying>,
    index: usize,
}

impl PlaybackQueue {
    pub fn set_queue(&mut self, items: Vec<NowPlaying>) {
        self.items = items;
        if self.items.is_empty() {
            self.index = 0;
        } else if self.index >= self.items.len() {
            self.index = self.items.len() - 1;
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if self.items.is_empty() {
            self.index = 0;
        } else {
            self.index = index.min(self.items.len() - 1);
        }
    }

    pub fn items(&self) -> &[NowPlaying] {
        &self.items
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.index = 0;
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() {
            return false;
        }
        if from == to {
            return true;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        if self.index == from {
            self.index = to;
        } else if from < self.index && to >= self.index {
            self.index = self.index.saturating_sub(1);
        } else if from > self.index && to <= self.index {
            self.index = (self.index + 1).min(self.items.len().saturating_sub(1));
        }
        true
    }

    pub fn current(&self) -> Option<NowPlaying> {
        self.items.get(self.index).cloned()
    }

    pub fn peek_next(&self) -> Option<&NowPlaying> {
        self.items.get(self.index + 1)
    }

    pub fn next(&mut self) -> Option<NowPlaying> {
        if self.index + 1 < self.items.len() {
            self.index += 1;
            self.current()
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<NowPlaying> {
        if self.index > 0 {
            self.index -= 1;
            self.current()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<NowPlaying>,
}

impl Playlist {
    pub fn empty(name: impl Into<String>) -> Self {
        Self { name: name.into(), items: Vec::new() }
    }

    pub fn add(&mut self, item: NowPlaying) {
        self.items.push(item);
    }

    pub fn delete_item(&mut self, index: usize) -> Option<NowPlaying> {
        self.remove(index)
    }

    pub fn remove(&mut self, index: usize) -> Option<NowPlaying> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        self.reorder(from, to)
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() {
            return false;
        }
        if from == to {
            return true;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn set_items(&mut self, items: Vec<NowPlaying>) {
        self.items = items;
    }

    pub fn to_exchange(&self) -> PlaylistExchange {
        PlaylistExchange {
            version: PlaylistExchange::VERSION,
            name: self.name.clone(),
            items: self.items.iter().map(PlaylistItem::from).collect(),
        }
    }

    pub fn from_exchange(exchange: PlaylistExchange) -> Self {
        Self {
            name: exchange.name,
            items: exchange.items.into_iter().map(NowPlaying::from).collect(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_exchange())
    }

    pub fn from_json(payload: &str) -> Result<Self, serde_json::Error> {
        let exchange: PlaylistExchange = serde_json::from_str(payload)?;
        Ok(Self::from_exchange(exchange))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistManager {
    pub playlists: Vec<Playlist>,
    pub active_index: usize,
}

impl PlaylistManager {
    pub fn new_default() -> Self {
        Self {
            playlists: vec![Playlist::empty("Queue")],
            active_index: 0,
        }
    }

    pub fn load_or_default() -> Self {
        let path = playlist_path();
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Self::new_default();
            }
            Err(err) => {
                warn!(error = %err, path = %path.display(), "Failed to read playlist");
                return Self::new_default();
            }
        };

        match Self::from_json(&contents) {
            Ok(manager) => manager,
            Err(err) => {
                warn!(error = %err, path = %path.display(), "Failed to parse playlist");
                Self::new_default()
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = playlist_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = self.to_json().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        atomic_write(&path, payload.as_bytes())
    }

    pub fn active(&self) -> Option<&Playlist> {
        self.playlists.get(self.active_index)
    }

    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.playlists.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    pub fn create_playlist(
        &mut self,
        name: impl Into<String>,
        default_label: &str,
    ) -> (usize, String) {
        let name = name.into();
        let base = name.trim();
        let base = if base.is_empty() { default_label } else { base };
        let unique = self.unique_name(base, None, default_label);
        self.playlists.push(Playlist::empty(unique.clone()));
        self.active_index = self.playlists.len().saturating_sub(1);
        (self.active_index, unique)
    }

    pub fn rename_playlist(
        &mut self,
        index: usize,
        name: impl Into<String>,
        default_label: &str,
    ) -> Option<String> {
        let name = name.into();
        let base = name.trim();
        if base.is_empty() {
            return None;
        }
        let unique = self.unique_name(base, Some(index), default_label);
        self.playlists.get_mut(index).map(|playlist| {
            playlist.name = unique.clone();
            unique
        })
    }

    pub fn remove_playlist(&mut self, index: usize) -> bool {
        if self.playlists.len() <= 1 || index >= self.playlists.len() {
            return false;
        }
        self.playlists.remove(index);
        if self.active_index >= self.playlists.len() {
            self.active_index = self.playlists.len().saturating_sub(1);
        }
        true
    }

    pub fn add(&mut self, item: NowPlaying) {
        if let Some(playlist) = self.playlists.get_mut(self.active_index) {
            playlist.add(item);
        }
    }

    pub fn delete_item(&mut self, index: usize) -> Option<NowPlaying> {
        self.playlists
            .get_mut(self.active_index)
            .and_then(|playlist| playlist.delete_item(index))
    }

    pub fn remove(&mut self, index: usize) -> Option<NowPlaying> {
        self.playlists
            .get_mut(self.active_index)
            .and_then(|playlist| playlist.remove(index))
    }

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        self.playlists
            .get_mut(self.active_index)
            .map_or(false, |playlist| playlist.move_item(from, to))
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        self.playlists
            .get_mut(self.active_index)
            .map_or(false, |playlist| playlist.reorder(from, to))
    }

    pub fn clear(&mut self) {
        if let Some(playlist) = self.playlists.get_mut(self.active_index) {
            playlist.clear();
        }
    }

    pub fn set_items(&mut self, items: Vec<NowPlaying>) {
        if let Some(playlist) = self.playlists.get_mut(self.active_index) {
            playlist.set_items(items);
        }
    }

    fn unique_name(&self, base: &str, skip_index: Option<usize>, default_label: &str) -> String {
        let base = base.trim();
        if base.is_empty() {
            return default_label.trim().to_string();
        }
        let matches_name = |(index, playlist): (usize, &Playlist)| {
            if skip_index == Some(index) {
                return false;
            }
            playlist.name.eq_ignore_ascii_case(base)
        };
        if !self.playlists.iter().enumerate().any(matches_name) {
            return base.to_string();
        }
        for suffix in 2..=999 {
            let candidate = format!("{base} ({suffix})");
            let exists = self.playlists.iter().enumerate().any(|(index, playlist)| {
                if skip_index == Some(index) {
                    return false;
                }
                playlist.name.eq_ignore_ascii_case(&candidate)
            });
            if !exists {
                return candidate;
            }
        }
        format!("{base} (1000)")
    }

    pub fn to_exchange(&self) -> PlaylistManagerExchange {
        PlaylistManagerExchange {
            version: PlaylistManagerExchange::VERSION,
            playlists: self.playlists.iter().map(Playlist::to_exchange).collect(),
            active_index: self.active_index,
        }
    }

    pub fn from_exchange(exchange: PlaylistManagerExchange) -> Self {
        let mut playlists: Vec<Playlist> =
            exchange.playlists.into_iter().map(Playlist::from_exchange).collect();
        if playlists.is_empty() {
            playlists.push(Playlist::empty("Queue"));
        }
        let active_index = exchange.active_index.min(playlists.len().saturating_sub(1));
        Self { playlists, active_index }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_exchange())
    }

    pub fn from_json(payload: &str) -> Result<Self, serde_json::Error> {
        let exchange: PlaylistManagerExchange = serde_json::from_str(payload)?;
        Ok(Self::from_exchange(exchange))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistExchange {
    pub version: u32,
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

impl PlaylistExchange {
    pub const VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistManagerExchange {
    pub version: u32,
    pub playlists: Vec<PlaylistExchange>,
    pub active_index: usize,
}

impl PlaylistManagerExchange {
    pub const VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub duration_secs: u32,
    pub path: String,
}

impl From<&NowPlaying> for PlaylistItem {
    fn from(item: &NowPlaying) -> Self {
        Self {
            artist: item.artist.clone(),
            album: item.album.clone(),
            title: item.title.clone(),
            duration_secs: item.duration_secs,
            path: item.path.to_string_lossy().into_owned(),
        }
    }
}

impl From<PlaylistItem> for NowPlaying {
    fn from(item: PlaylistItem) -> Self {
        Self {
            artist: item.artist,
            album: item.album,
            title: item.title,
            duration_secs: item.duration_secs,
            path: PathBuf::from(item.path),
        }
    }
}

fn playlist_path() -> PathBuf {
    config_root().join("playlist.json")
}

fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp_path =
        parent.join(format!(".{}.tmp", path.file_name().unwrap_or_default().to_string_lossy()));
    fs::write(&tmp_path, data)?;
    fs::rename(&tmp_path, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track(index: u32) -> NowPlaying {
        NowPlaying {
            artist: format!("Artist {index}"),
            album: format!("Album {index}"),
            title: format!("Track {index}"),
            duration_secs: 180 + index,
            path: PathBuf::from(format!("/music/track_{index}.mp3")),
        }
    }

    #[test]
    fn add_remove_items() {
        let mut playlist = Playlist::empty("Favorites");
        let first = sample_track(1);
        let second = sample_track(2);

        playlist.add(first.clone());
        playlist.add(second.clone());

        assert_eq!(playlist.items.len(), 2);
        assert_eq!(playlist.remove(0), Some(first));
        assert_eq!(playlist.items.len(), 1);
        assert_eq!(playlist.remove(10), None);
        assert_eq!(playlist.items, vec![second]);
    }

    #[test]
    fn reorder_items() {
        let mut playlist = Playlist::empty("Mix");
        let first = sample_track(1);
        let second = sample_track(2);
        let third = sample_track(3);

        playlist.add(first.clone());
        playlist.add(second.clone());
        playlist.add(third.clone());

        assert!(playlist.reorder(0, 2));
        assert_eq!(playlist.items, vec![second.clone(), third.clone(), first.clone()]);
        assert!(playlist.reorder(1, 1));
        assert!(!playlist.reorder(10, 0));
    }

    #[test]
    fn exchange_and_json_roundtrip() {
        let mut playlist = Playlist::empty("Roadtrip");
        playlist.add(sample_track(1));
        playlist.add(sample_track(2));

        let exchange = playlist.to_exchange();
        assert_eq!(exchange.version, PlaylistExchange::VERSION);

        let json = playlist.to_json().expect("serialize playlist");
        let parsed = Playlist::from_json(&json).expect("deserialize playlist");

        assert_eq!(playlist, parsed);
    }

    #[test]
    fn manager_json_roundtrip() {
        let mut manager = PlaylistManager::new_default();
        manager.add(sample_track(1));
        manager.add(sample_track(2));

        let json = manager.to_json().expect("serialize manager");
        let parsed = PlaylistManager::from_json(&json).expect("deserialize manager");

        assert_eq!(manager, parsed);
    }
}
