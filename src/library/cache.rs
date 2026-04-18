use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::library::{Album, EmbeddedCover, Track};

const CACHE_DIRNAME: &str = ".grape_cache";
const INDEX_FILENAME: &str = "index.json";
const FOLDERS_DIRNAME: &str = "folders";
const TRACKS_DIRNAME: &str = "tracks";
const COVER_DIRNAME: &str = "covers";
const METADATA_DIRNAME: &str = "metadata";
const CACHE_VERSION: u32 = 5;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheIndex {
    version: u32,
    #[serde(default)]
    tracks: HashMap<String, TrackEntry>,
    #[serde(skip)]
    legacy_version: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderEntry {
    #[serde(default)]
    tracks: HashMap<String, TrackEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderCacheFile {
    album: Album,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TrackEntry {
    #[serde(default)]
    pub id: String,
    pub modified_secs: u64,
    pub hash: u64,
    #[serde(default)]
    pub file_len: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TrackSignature {
    pub modified_secs: u64,
    pub hash: u64,
    #[serde(default)]
    pub file_len: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackMetadata {
    number: u8,
    title: String,
    duration_secs: u32,
    #[serde(default)]
    duration_millis: Option<u64>,
    #[serde(default)]
    bitrate_kbps: Option<u32>,
    #[serde(default)]
    codec: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    album_artist: Option<String>,
    #[serde(default)]
    compilation: bool,
    #[serde(default)]
    year: Option<u16>,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    embedded_cover: Option<EmbeddedCover>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackCacheFile {
    id: String,
    signature: TrackSignature,
    metadata: TrackMetadata,
}

pub struct CachedAlbum {
    pub album: Album,
}

impl CacheIndex {
    pub fn track_entries(&self) -> &HashMap<String, TrackEntry> {
        &self.tracks
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyCacheIndex {
    version: u32,
    #[serde(default)]
    entries: HashMap<String, FolderEntry>,
}

pub fn load_index(root: &Path) -> io::Result<CacheIndex> {
    if !root.exists() {
        return Ok(CacheIndex::default());
    }

    let index_path = index_path(root);
    if !index_path.exists() {
        return Ok(CacheIndex::default());
    }

    let contents = fs::read_to_string(&index_path)?;
    let value: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if value.get("entries").is_some() {
        let legacy: LegacyCacheIndex = serde_json::from_value(value)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let mut tracks = HashMap::new();
        for entry in legacy.entries.values() {
            for (key, track_entry) in &entry.tracks {
                let id = hash_key(key);
                tracks.insert(
                    key.clone(),
                    TrackEntry {
                        id,
                        modified_secs: track_entry.modified_secs,
                        hash: track_entry.hash,
                        file_len: None,
                    },
                );
            }
        }
        return Ok(CacheIndex {
            version: CACHE_VERSION,
            tracks,
            legacy_version: Some(legacy.version),
        });
    }

    let mut index: CacheIndex = serde_json::from_value(value)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if index.version != CACHE_VERSION {
        index.legacy_version = Some(index.version);
        index.version = CACHE_VERSION;
    }

    for (key, entry) in &mut index.tracks {
        if entry.id.is_empty() {
            entry.id = hash_key(key);
        }
    }

    Ok(index)
}

pub fn load_album(root: &Path, album_path: &Path) -> io::Result<Option<CachedAlbum>> {
    let key = album_key(root, album_path)?;
    let cache_path = folder_cache_path(root, &key);
    if !cache_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&cache_path)?;
    let mut cache_file: FolderCacheFile = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    cache_file.album.path = album_path.to_path_buf();
    Ok(Some(CachedAlbum {
        album: cache_file.album,
    }))
}

pub fn store_album(
    root: &Path,
    index: &mut CacheIndex,
    album_path: &Path,
    album: &Album,
) -> io::Result<String> {
    let key = album_key(root, album_path)?;
    let cache_dir = root.join(CACHE_DIRNAME).join(FOLDERS_DIRNAME);
    fs::create_dir_all(&cache_dir)?;

    let cache_file = FolderCacheFile {
        album: album.clone(),
    };

    let contents = serde_json::to_string_pretty(&cache_file)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(folder_cache_path(root, &key), contents)?;

    let track_entries = build_track_entries(root, album)?;
    for (track_key, track_entry) in track_entries {
        index.tracks.insert(track_key, track_entry);
    }

    Ok(key)
}

pub fn finalize(
    root: &Path,
    index: &mut CacheIndex,
    used_keys: &HashSet<String>,
    used_track_keys: &HashSet<String>,
    used_track_ids: &HashSet<String>,
    used_metadata_keys: &HashSet<String>,
    used_cover_filenames: &HashSet<String>,
) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    index.version = CACHE_VERSION;
    let folders_dir = root.join(CACHE_DIRNAME).join(FOLDERS_DIRNAME);
    if folders_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&folders_dir) {
            for entry_result in read_dir {
                let Ok(entry) = entry_result else {
                    continue;
                };
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !used_keys.contains(stem) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let removed_tracks: Vec<String> = index
        .tracks
        .keys()
        .filter(|key| !used_track_keys.contains(*key))
        .cloned()
        .collect();

    for key in removed_tracks {
        index.tracks.remove(&key);
    }

    let tracks_dir = root.join(CACHE_DIRNAME).join(TRACKS_DIRNAME);
    if tracks_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&tracks_dir) {
            for entry_result in read_dir {
                let Ok(entry) = entry_result else {
                    continue;
                };
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !used_track_ids.contains(stem) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let metadata_dir = root.join(CACHE_DIRNAME).join(METADATA_DIRNAME);
    if metadata_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&metadata_dir) {
            for entry_result in read_dir {
                let Ok(entry) = entry_result else {
                    continue;
                };
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !used_metadata_keys.contains(stem) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let covers_dir = root.join(CACHE_DIRNAME).join(COVER_DIRNAME);
    if covers_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&covers_dir) {
            for entry_result in read_dir {
                let Ok(entry) = entry_result else {
                    continue;
                };
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !used_cover_filenames.contains(file_name) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let contents = serde_json::to_string_pretty(index)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::create_dir_all(root.join(CACHE_DIRNAME))?;
    fs::write(index_path(root), contents)
}

pub fn ensure_cover_cache_dir(root: &Path) -> io::Result<PathBuf> {
    let dir = root.join(CACHE_DIRNAME).join(COVER_DIRNAME);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn ensure_metadata_cache_dir(root: &Path) -> io::Result<PathBuf> {
    let dir = root.join(CACHE_DIRNAME).join(METADATA_DIRNAME);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn index_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIRNAME).join(INDEX_FILENAME)
}

fn folder_cache_path(root: &Path, key: &str) -> PathBuf {
    root.join(CACHE_DIRNAME)
        .join(FOLDERS_DIRNAME)
        .join(format!("{key}.json"))
}

fn album_key(root: &Path, album_path: &Path) -> io::Result<String> {
    let relative = relative_path(root, album_path);
    Ok(hash_key(&relative))
}

fn hash_key(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn track_key(root: &Path, track_path: &Path) -> String {
    relative_path(root, track_path)
}

pub fn track_id(root: &Path, track_path: &Path) -> String {
    hash_key(&track_key(root, track_path))
}

pub fn track_signature(path: &Path) -> io::Result<TrackSignature> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    let modified_secs = duration.as_secs();
    let file_len = metadata.len();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    file_len.hash(&mut hasher);
    modified_secs.hash(&mut hasher);
    Ok(TrackSignature {
        modified_secs,
        hash: hasher.finish(),
        file_len: Some(file_len),
    })
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn build_track_entries(root: &Path, album: &Album) -> io::Result<HashMap<String, TrackEntry>> {
    let mut entries = HashMap::new();
    for track in &album.tracks {
        if let Ok(signature) = track_signature(&track.path) {
            let key = track_key(root, &track.path);
            let id = track_id(root, &track.path);
            store_track_metadata(root, &id, &signature, track)?;
            entries.insert(
                key,
                TrackEntry {
                    id,
                    modified_secs: signature.modified_secs,
                    hash: signature.hash,
                    file_len: signature.file_len,
                },
            );
        }
    }
    Ok(entries)
}

fn track_cache_path(root: &Path, id: &str) -> PathBuf {
    root.join(CACHE_DIRNAME)
        .join(TRACKS_DIRNAME)
        .join(format!("{id}.json"))
}

pub fn load_track_metadata(root: &Path, id: &str) -> io::Result<Option<TrackCacheFile>> {
    let path = track_cache_path(root, id);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)?;
    let cache_file: TrackCacheFile = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(Some(cache_file))
}

pub fn store_track_metadata(
    root: &Path,
    id: &str,
    signature: &TrackSignature,
    track: &Track,
) -> io::Result<()> {
    let cache_dir = root.join(CACHE_DIRNAME).join(TRACKS_DIRNAME);
    fs::create_dir_all(&cache_dir)?;
    let cache_file = TrackCacheFile {
        id: id.to_string(),
        signature: signature.clone(),
        metadata: TrackMetadata::from_track(track),
    };
    let contents = serde_json::to_string_pretty(&cache_file)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(track_cache_path(root, id), contents)
}

impl TrackEntry {
    pub fn matches_signature(&self, signature: &TrackSignature) -> bool {
        if self.modified_secs != signature.modified_secs || self.hash != signature.hash {
            return false;
        }
        match (self.file_len, signature.file_len) {
            (Some(left), Some(right)) => left == right,
            _ => true,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl TrackSignature {
    pub fn matches_cache(&self, cache: &TrackCacheFile) -> bool {
        if self.modified_secs != cache.signature.modified_secs || self.hash != cache.signature.hash
        {
            return false;
        }
        match (self.file_len, cache.signature.file_len) {
            (Some(left), Some(right)) => left == right,
            _ => true,
        }
    }
}

impl TrackCacheFile {
    pub fn metadata(&self) -> &TrackMetadata {
        &self.metadata
    }
}

impl TrackMetadata {
    pub fn from_track(track: &Track) -> Self {
        Self {
            number: track.number,
            title: track.title.clone(),
            duration_secs: track.duration_secs,
            duration_millis: track.duration_millis,
            bitrate_kbps: track.bitrate_kbps,
            codec: track.codec.clone(),
            artist: track.artist.clone(),
            album_artist: track.album_artist.clone(),
            compilation: track.compilation,
            year: track.year,
            genre: track.genre.clone(),
            embedded_cover: track.embedded_cover.clone(),
        }
    }

    pub fn into_track(self, path: PathBuf) -> Track {
        Track {
            number: self.number,
            title: self.title,
            duration_secs: self.duration_secs,
            duration_millis: self.duration_millis,
            bitrate_kbps: self.bitrate_kbps,
            codec: self.codec,
            path,
            artist: self.artist,
            album_artist: self.album_artist,
            compilation: self.compilation,
            year: self.year,
            genre: self.genre,
            embedded_cover: self.embedded_cover,
        }
    }
}
