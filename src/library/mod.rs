use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::UserSettings;

/// Display name used for the "Various Artists" virtual grouping when a
/// compilation flag is detected on one or more tracks.
const VARIOUS_ARTISTS_LABEL: &str = "Various Artists";

/// Infer the album-level artist from a set of tracks, following the standard
/// precedence used by foobar2000 / MusicBee / iTunes:
/// 1. If any track has the `COMPILATION` / `TCMP` flag set, the album is a
///    compilation → "Various Artists".
/// 2. Else, if every track with an `ALBUMARTIST` tag agrees on a single
///    value → that value.
/// 3. Else, if every track with an `ARTIST` tag agrees on a single value →
///    that value (treats mono-artist albums that omit ALBUMARTIST).
/// 4. Else → `None`; caller should fall back to folder-derived name or the
///    generic "unknown artist" label.
pub(crate) fn infer_album_artist(tracks: &[Track]) -> Option<String> {
    if tracks.iter().any(|t| t.compilation) {
        return Some(VARIOUS_ARTISTS_LABEL.to_string());
    }
    if let Some(common) = single_common_value(tracks.iter().map(|t| t.album_artist.as_deref())) {
        return Some(common.to_string());
    }
    if let Some(common) = single_common_value(tracks.iter().map(|t| t.artist.as_deref())) {
        return Some(common.to_string());
    }
    None
}

/// Returns `Some(value)` when every non-empty entry in the iterator trims to
/// the same string, ignoring `None` and empty values. Returns `None` when the
/// iterator is empty, all entries are empty, or entries disagree.
fn single_common_value<'a, I>(values: I) -> Option<&'a str>
where
    I: IntoIterator<Item = Option<&'a str>>,
{
    let mut consensus: Option<&'a str> = None;
    for value in values {
        let trimmed = match value.map(str::trim).filter(|v| !v.is_empty()) {
            Some(v) => v,
            None => continue,
        };
        match consensus {
            None => consensus = Some(trimmed),
            Some(existing) if existing == trimmed => {}
            Some(_) => return None,
        }
    }
    consensus
}

pub mod cache;
pub mod metadata;

pub use metadata::online::OnlineMetadata;

/// The complete music library, organized as a list of [`Artist`]s.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenreSummary {
    pub name: String,
    pub track_count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderSummary {
    pub name: String,
    pub track_count: usize,
}

/// A music artist containing one or more [`Album`]s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub albums: Vec<Album>,
    #[serde(default)]
    pub genre: Option<String>,
}

/// A music album containing its [`Track`]s and optional cover art.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Album {
    pub title: String,
    pub year: u16,
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub path: PathBuf,
    #[serde(default)]
    pub total_duration_secs: u32,
    #[serde(default)]
    pub cover: Option<CoverArt>,
}

/// A single audio track with its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub number: u8,
    pub title: String,
    pub duration_secs: u32,
    #[serde(default)]
    pub duration_millis: Option<u64>,
    #[serde(default)]
    pub bitrate_kbps: Option<u32>,
    #[serde(default)]
    pub codec: Option<String>,
    pub path: PathBuf,
    #[serde(default)]
    pub artist: Option<String>,
    /// `ALBUMARTIST` tag value. Kept per-track so album-level aggregation can
    /// detect consensus without re-reading files. Defaults to `None` for
    /// legacy cache entries.
    #[serde(default)]
    pub album_artist: Option<String>,
    /// Compilation flag (TCMP / COMPILATION). Defaults to `false` for legacy
    /// cache entries that predate this field.
    #[serde(default)]
    pub compilation: bool,
    #[serde(default)]
    pub year: Option<u16>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub embedded_cover: Option<EmbeddedCover>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverArt {
    pub source_path: PathBuf,
    pub cached_path: PathBuf,
    pub modified_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedCover {
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
struct MetadataLocks {
    genre: bool,
    year: bool,
}

/// Groups the mutable tracking state used during a library scan so that
/// individual helper functions do not need to accept 5-6 separate `HashSet`
/// parameters.
struct ScanState {
    cache_index: cache::CacheIndex,
    used_cache_keys: std::collections::HashSet<String>,
    used_track_keys: std::collections::HashSet<String>,
    used_track_ids: std::collections::HashSet<String>,
    used_metadata_keys: std::collections::HashSet<String>,
    used_cover_filenames: std::collections::HashSet<String>,
}

impl ScanState {
    fn new(root: &Path, use_cache: bool) -> Self {
        let cache_index = if use_cache {
            match cache::load_index(root) {
                Ok(index) => index,
                Err(error) => {
                    warn!(error = %error, "Unable to load cache index; scanning without cache");
                    cache::CacheIndex::default()
                }
            }
        } else {
            cache::CacheIndex::default()
        };
        Self {
            cache_index,
            used_cache_keys: std::collections::HashSet::new(),
            used_track_keys: std::collections::HashSet::new(),
            used_track_ids: std::collections::HashSet::new(),
            used_metadata_keys: std::collections::HashSet::new(),
            used_cover_filenames: std::collections::HashSet::new(),
        }
    }

    fn record_track_keys(&mut self, root: &Path, tracks: &[Track]) {
        for track in tracks {
            self.used_track_keys.insert(cache::track_key(root, &track.path));
        }
    }

    fn record_track_ids(&mut self, root: &Path, tracks: &[Track]) {
        for track in tracks {
            self.used_track_ids.insert(cache::track_id(root, &track.path));
        }
    }

    fn record_metadata_key(&mut self, artist_name: &str, album: &Album) {
        let key = metadata::online::metadata_cache_key_for_album(artist_name, &album.title);
        self.used_metadata_keys.insert(key);
    }

    fn record_cover_cache_file(&mut self, album: &Album) {
        let Some(cover) = album.cover.as_ref() else {
            return;
        };
        let Some(file_name) = cover.cached_path.file_name().and_then(|name| name.to_str()) else {
            return;
        };
        self.used_cover_filenames.insert(file_name.to_string());
    }

    fn store_album(&mut self, root: &Path, album_path: &Path, album: &Album) {
        if let Ok(key) = cache::store_album(root, &mut self.cache_index, album_path, album) {
            self.used_cache_keys.insert(key);
        }
    }

    fn finalize(mut self, root: &Path) {
        if let Err(error) = cache::finalize(
            root,
            &mut self.cache_index,
            &self.used_cache_keys,
            &self.used_track_keys,
            &self.used_track_ids,
            &self.used_metadata_keys,
            &self.used_cover_filenames,
        ) {
            warn!(error = %error, "Unable to persist cache index");
        }
    }
}

impl Catalog {
    pub fn empty() -> Self {
        Self {
            artists: Vec::new(),
        }
    }

    pub fn genres(&self, unknown_genre_label: &str) -> Vec<GenreSummary> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for artist in &self.artists {
            for album in &artist.albums {
                for track in &album.tracks {
                    let mut had_genre = false;
                    if let Some(genre) = &track.genre {
                        for name in split_genre_field(genre) {
                            had_genre = true;
                            *counts.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                    if !had_genre {
                        *counts
                            .entry(unknown_genre_label.to_string())
                            .or_insert(0) += 1;
                    }
                }
            }
        }
        let mut genres: Vec<_> = counts
            .into_iter()
            .map(|(name, track_count)| GenreSummary { name, track_count })
            .collect();
        genres.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        genres
    }

    #[allow(dead_code)]
    pub fn folders(&self) -> Vec<FolderSummary> {
        let mut folders = Vec::new();
        for artist in &self.artists {
            for album in &artist.albums {
                if album.tracks.is_empty() {
                    continue;
                }
                let album_folder = if album.year > 0 {
                    format!("{:04} - {}", album.year, album.title)
                } else {
                    album.title.clone()
                };
                let name = format!("{}/{}", artist.name, album_folder);
                folders.push(FolderSummary {
                    name,
                    track_count: album.tracks.len(),
                });
            }
        }
        folders.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.name.cmp(&b.name))
        });
        folders
    }

    pub fn prune_missing_cover_art(&mut self) {
        for artist in &mut self.artists {
            for album in &mut artist.albums {
                if let Some(cover) = &album.cover {
                    if !cover.cached_path.exists() {
                        album.cover = None;
                    }
                }
            }
        }
    }
}

/// Scans the library at `root`, using the on-disk cache for unchanged albums.
pub fn scan_library(
    root: impl AsRef<Path>,
    _settings: &UserSettings,
    root_artist_label: &str,
) -> io::Result<Catalog> {
    scan_library_with_cache(root, true, root_artist_label)
}

/// Scans the library at `root` from scratch, ignoring any cached data.
pub fn scan_library_full(
    root: impl AsRef<Path>,
    _settings: &UserSettings,
    root_artist_label: &str,
) -> io::Result<Catalog> {
    scan_library_with_cache(root, false, root_artist_label)
}

pub fn persist_album_metadata_override(
    root: &Path,
    artist: &str,
    album: &str,
    genre: Option<String>,
    year: Option<u16>,
) -> io::Result<()> {
    let metadata_override = metadata::online::UserMetadataOverride {
        genre,
        year,
        genre_overridden: true,
        year_overridden: true,
        edited_at: 0,
    };
    metadata::online::store_user_metadata_override(root, artist, album, metadata_override)
}

pub async fn fetch_album_online_metadata(
    root: &Path,
    settings: &UserSettings,
    artist: &str,
    album: &str,
    force_refresh: bool,
) -> io::Result<Option<OnlineMetadata>> {
    metadata::online::fetch_album_metadata(root, settings, artist, album, force_refresh).await
}

fn scan_library_with_cache(
    root: impl AsRef<Path>,
    use_cache: bool,
    root_artist_label: &str,
) -> io::Result<Catalog> {
    let root = root.as_ref();
    let mut artists = Vec::new();
    let mut root_artist_albums = Vec::new();
    let mut seen_artist_dirs = std::collections::HashSet::new();
    let mut state = ScanState::new(root, use_cache);

    if !root.exists() {
        return Ok(Catalog::empty());
    }

    if let Some(album) = scan_root_album(root, use_cache, &mut state, root_artist_label)? {
        root_artist_albums.push(album);
    }

    for artist_entry in read_sorted_dirs(root)? {
        let artist_path = artist_entry.path();
        let artist_key = normalized_path_key(root, &artist_path);
        if !seen_artist_dirs.insert(artist_key) {
            warn!(
                path = %artist_path.display(),
                "Skipping duplicate artist directory"
            );
            continue;
        }
        let artist_name = artist_entry
            .file_name()
            .to_string_lossy()
            .trim()
            .to_string();

        if dir_has_audio_files(&artist_path)? {
            let (year, title) = parse_album_folder(&artist_name);
            if let Some(album) =
                scan_album_dir(root, &artist_path, &artist_name, year, title, use_cache, &mut state)?
            {
                root_artist_albums.push(album);
            }
            continue;
        }

        let albums = scan_artist_albums(root, &artist_path, &artist_name, use_cache, &mut state)?;

        if !albums.is_empty() {
            let genre = dominant_genre(
                albums
                    .iter()
                    .flat_map(|album| album.tracks.iter())
                    .flat_map(|track| track.genre.as_deref()),
            );
            artists.push(Artist {
                name: artist_name,
                albums,
                genre,
            });
        }
    }

    if !root_artist_albums.is_empty() {
        // Root-level albums (no artist folder level) previously all landed
        // under the generic "unknown artist" bucket. Group them by their
        // tag-inferred album artist instead; albums whose tracks carry no
        // usable tag still fall back to `root_artist_label`.
        let mut tag_buckets: BTreeMap<String, Vec<Album>> = BTreeMap::new();
        let mut unknown_bucket: Vec<Album> = Vec::new();

        for album in root_artist_albums {
            match infer_album_artist(&album.tracks) {
                Some(name) => tag_buckets.entry(name).or_default().push(album),
                None => unknown_bucket.push(album),
            }
        }

        // Preserve the historical placement of the "unknown" synthetic artist
        // at index 0 when it ends up populated; tag-derived artists are added
        // after in alphabetical order so the UI displays a stable sequence.
        if !unknown_bucket.is_empty() {
            let genre = dominant_genre(
                unknown_bucket
                    .iter()
                    .flat_map(|album| album.tracks.iter())
                    .flat_map(|track| track.genre.as_deref()),
            );
            artists.insert(
                0,
                Artist {
                    name: root_artist_label.to_string(),
                    albums: unknown_bucket,
                    genre,
                },
            );
        }

        for (name, albums) in tag_buckets {
            let genre = dominant_genre(
                albums
                    .iter()
                    .flat_map(|album| album.tracks.iter())
                    .flat_map(|track| track.genre.as_deref()),
            );
            artists.push(Artist { name, albums, genre });
        }
    }

    let mut catalog = Catalog { artists };
    catalog.prune_missing_cover_art();
    state.finalize(root);

    Ok(catalog)
}

/// Scans all album directories under a single artist directory.
fn scan_artist_albums(
    root: &Path,
    artist_path: &Path,
    artist_name: &str,
    use_cache: bool,
    state: &mut ScanState,
) -> io::Result<Vec<Album>> {
    let mut albums = Vec::new();
    let mut seen_album_dirs = std::collections::HashSet::new();

    for album_entry in read_sorted_dirs(artist_path)? {
        let (year, title) = parse_album_folder(&album_entry.file_name().to_string_lossy());
        let album_path = album_entry.path();
        let album_key = normalized_path_key(root, &album_path);
        if !seen_album_dirs.insert(album_key) {
            warn!(
                path = %album_path.display(),
                "Skipping duplicate album directory"
            );
            continue;
        }
        let cached_album = if use_cache {
            match cache::load_album(root, &album_path) {
                Ok(cached) => cached,
                Err(error) => {
                    warn!(
                        error = %error,
                        path = %album_path.display(),
                        "Unable to load cached album; rescanning"
                    );
                    None
                }
            }
        } else {
            None
        };

        let mut album = if let Some(cached) = cached_album {
            let tracks = scan_tracks_with_cache_in_dir(
                root,
                &album_path,
                &cached.album.tracks,
                state.cache_index.track_entries(),
                &mut state.used_track_keys,
                &mut state.used_track_ids,
                true,
            )?;
            if tracks.is_empty() {
                continue;
            }
            let year = resolve_album_year(year, &tracks);
            let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
            let cover =
                select_album_cover(root, &album_path, &tracks, cached.album.cover.as_ref())?;
            let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
            Album {
                title: title.clone(),
                year,
                tracks,
                genre,
                path: album_path.clone(),
                total_duration_secs,
                cover,
            }
        } else {
            let tracks = scan_tracks(&album_path)?;
            if tracks.is_empty() {
                continue;
            }
            state.record_track_keys(root, &tracks);
            state.record_track_ids(root, &tracks);
            let year = resolve_album_year(year, &tracks);
            let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
            let cover = select_album_cover(root, &album_path, &tracks, None)?;
            let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
            Album {
                title: title.clone(),
                year,
                tracks,
                genre,
                path: album_path.clone(),
                total_duration_secs,
                cover,
            }
        };

        apply_user_metadata_override(root, artist_name, &mut album);
        state.record_metadata_key(artist_name, &album);
        state.record_cover_cache_file(&album);

        if !album.tracks.is_empty() {
            state.store_album(root, &album_path, &album);
            albums.push(album);
        }
    }

    Ok(albums)
}

fn scan_tracks(dir: &Path) -> io::Result<Vec<Track>> {
    scan_tracks_in_dir(dir, true)
}

fn scan_album_dir(
    root: &Path,
    album_path: &Path,
    artist_name: &str,
    year: u16,
    title: String,
    use_cache: bool,
    state: &mut ScanState,
) -> io::Result<Option<Album>> {
    let cached_album = if use_cache {
        match cache::load_album(root, album_path) {
            Ok(cached) => cached,
            Err(error) => {
                warn!(
                    error = %error,
                    path = %album_path.display(),
                    "Unable to load cached album; rescanning"
                );
                None
            }
        }
    } else {
        None
    };

    let mut album = if let Some(cached) = cached_album {
        let tracks = scan_tracks_with_cache_in_dir(
            root,
            album_path,
            &cached.album.tracks,
            state.cache_index.track_entries(),
            &mut state.used_track_keys,
            &mut state.used_track_ids,
            true,
        )?;
        if tracks.is_empty() {
            return Ok(None);
        }
        let year = resolve_album_year(year, &tracks);
        let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
        let cover = select_album_cover(root, album_path, &tracks, cached.album.cover.as_ref())?;
        let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
        Album {
            title,
            year,
            tracks,
            genre,
            path: album_path.to_path_buf(),
            total_duration_secs,
            cover,
        }
    } else {
        let tracks = scan_tracks(album_path)?;
        if tracks.is_empty() {
            return Ok(None);
        }
        state.record_track_keys(root, &tracks);
        state.record_track_ids(root, &tracks);
        let year = resolve_album_year(year, &tracks);
        let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
        let cover = select_album_cover(root, album_path, &tracks, None)?;
        let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
        Album {
            title,
            year,
            tracks,
            genre,
            path: album_path.to_path_buf(),
            total_duration_secs,
            cover,
        }
    };

    apply_user_metadata_override(root, artist_name, &mut album);
    state.record_metadata_key(artist_name, &album);
    state.record_cover_cache_file(&album);
    state.store_album(root, album_path, &album);

    Ok(Some(album))
}

fn apply_user_metadata_override(root: &Path, artist_name: &str, album: &mut Album) -> MetadataLocks {
    let user_override =
        metadata::online::load_user_metadata_override(root, artist_name, &album.title)
            .ok()
            .flatten();
    let mut locks = MetadataLocks::default();
    if let Some(metadata_override) = user_override {
        if metadata_override.genre_overridden {
            locks.genre = true;
            apply_album_genre(album, metadata_override.genre.clone());
        }
        if metadata_override.year_overridden {
            album.year = metadata_override.year.unwrap_or(0);
            locks.year = true;
        }
    }
    locks
}

pub fn merge_album_online_metadata(
    root: &Path,
    artist_name: &str,
    album: &mut Album,
    metadata: &OnlineMetadata,
    enrichment_confirmed: bool,
) {
    let locks = apply_user_metadata_override(root, artist_name, album);
    if !locks.genre {
        let merged_genre =
            metadata::merge_genre(album.genre.clone(), metadata.genre.clone(), enrichment_confirmed);
        if let Some(genre) = merged_genre {
            if enrichment_confirmed {
                apply_album_genre(album, Some(genre));
            } else if album.genre.is_none() {
                apply_album_genre_if_missing(album, &genre);
            }
        }
    }

    if !locks.year {
        let merged_year = metadata::merge_year(album.year, metadata.year, enrichment_confirmed);
        if merged_year > 0 {
            album.year = merged_year;
        }
    }
}

fn apply_album_genre(album: &mut Album, genre: Option<String>) {
    album.genre = genre.clone();
    for track in &mut album.tracks {
        track.genre = genre.clone();
    }
}

fn apply_album_genre_if_missing(album: &mut Album, genre: &str) {
    album.genre = Some(genre.to_string());
    for track in &mut album.tracks {
        if track.genre.is_none() {
            track.genre = Some(genre.to_string());
        }
    }
}

fn resolve_album_year(year: u16, tracks: &[Track]) -> u16 {
    if year > 0 {
        return year;
    }
    dominant_year(tracks.iter().filter_map(|track| track.year)).unwrap_or(year)
}

fn dominant_year(years: impl Iterator<Item = u16>) -> Option<u16> {
    let mut counts: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for year in years {
        if year > 0 {
            *counts.entry(year).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(year, _)| year)
}

fn select_album_cover(
    root: &Path,
    album_dir: &Path,
    tracks: &[Track],
    cached_cover: Option<&CoverArt>,
) -> io::Result<Option<CoverArt>> {
    // Prefer external cover files (cover.png, cover.jpg, etc.) placed by the user
    if let Some(cover) = scan_cover_art(root, album_dir)? {
        return Ok(Some(cover));
    }
    // Fall back to covers embedded in audio file tags
    if let Some(cover) = cover_from_embedded(root, tracks)? {
        return Ok(Some(cover));
    }
    // Last resort: reuse a cached cover from a previous scan if still on disk
    if let Some(cover) = cached_cover {
        if cover.cached_path.exists() {
            return Ok(Some(cover.clone()));
        }
    }
    Ok(None)
}

fn cover_from_embedded(root: &Path, tracks: &[Track]) -> io::Result<Option<CoverArt>> {
    let mut selected: Option<(&Path, &EmbeddedCover)> = None;
    for track in tracks {
        let Some(cover) = track.embedded_cover.as_ref() else {
            continue;
        };
        if cover.data.is_empty() {
            continue;
        }
        if cover.data.len() > metadata::MAX_EMBEDDED_COVER_BYTES {
            warn!(
                path = %track.path.display(),
                size_bytes = cover.data.len(),
                max_bytes = metadata::MAX_EMBEDDED_COVER_BYTES,
                "Ignoring embedded cover larger than maximum size"
            );
            continue;
        }
        selected = Some((&track.path, cover));
        break;
    }

    let Some((track_path, embedded_cover)) = selected else {
        return Ok(None);
    };

    let modified_secs = file_modified_secs(track_path)?;
    let cache_dir = cache::ensure_cover_cache_dir(root)?;
    let extension =
        embedded_cover_extension(embedded_cover.mime_type.as_deref(), &embedded_cover.data);
    let cache_filename = cache_cover_filename_with_extension(track_path, modified_secs, extension);
    let cached_path = cache_dir.join(cache_filename);

    if !cached_path.exists() {
        fs::write(&cached_path, &embedded_cover.data)?;
    }

    Ok(Some(CoverArt {
        source_path: track_path.to_path_buf(),
        cached_path,
        modified_secs,
    }))
}

fn embedded_cover_extension(mime_type: Option<&str>, data: &[u8]) -> &'static str {
    match mime_type {
        Some(mime) if mime.eq_ignore_ascii_case("image/jpeg") => "jpg",
        Some(mime) if mime.eq_ignore_ascii_case("image/jpg") => "jpg",
        Some(mime) if mime.eq_ignore_ascii_case("image/png") => "png",
        Some(mime) if mime.eq_ignore_ascii_case("image/webp") => "webp",
        _ => image_extension_from_data(data),
    }
}

fn image_extension_from_data(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "png"
    } else if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        "webp"
    } else if data.starts_with(b"GIF8") {
        "gif"
    } else if data.starts_with(b"BM") {
        "bmp"
    } else {
        "jpg"
    }
}

fn scan_tracks_in_dir(dir: &Path, warn_on_dirs: bool) -> io::Result<Vec<Track>> {
    let mut tracks = Vec::new();
    let mut index = 1u8;
    let mut seen_tracks = std::collections::HashSet::new();

    let entries = sorted_track_paths(dir, warn_on_dirs)?;

    for path in entries {
        let dedupe_key = normalized_path_key(dir, &path);
        if !seen_tracks.insert(dedupe_key) {
            warn!(path = %path.display(), "Skipping duplicate track path");
            continue;
        }
        if !is_audio_file(&path) {
            info!(path = %path.display(), "Ignoring non-audio file");
            continue;
        }

        let stem = match path.file_stem().and_then(|value| value.to_str()) {
            Some(stem) if !stem.trim().is_empty() => stem,
            Some(_) => {
                warn!(path = %path.display(), "Ignoring track with empty name");
                continue;
            }
            None => {
                warn!(path = %path.display(), "Ignoring track with unreadable name");
                continue;
            }
        };
        let (number, parsed_title) = parse_track_filename(stem);
        let mut track_number = number.unwrap_or_else(|| {
            let current = index;
            index = index.saturating_add(1);
            current
        });
        let mut title = parsed_title;

        let metadata = metadata::track_metadata(&path);
        let duration_secs = metadata.duration_secs.unwrap_or(0);
        if let Some(metadata_number) = metadata.track_number {
            track_number = metadata_number;
        }
        if let Some(metadata_title) = metadata.title {
            title = metadata_title;
        }

        tracks.push(Track {
            number: track_number,
            title,
            duration_secs,
            duration_millis: metadata.duration_millis,
            bitrate_kbps: metadata.bitrate_kbps,
            codec: metadata.codec,
            path,
            artist: metadata.artist,
            album_artist: metadata.album_artist,
            compilation: metadata.compilation,
            year: metadata.year,
            genre: metadata.genre,
            embedded_cover: metadata.embedded_cover,
        });
    }

    tracks.sort_by_key(|track| track.number);
    Ok(tracks)
}

fn scan_tracks_with_cache_in_dir(
    root: &Path,
    dir: &Path,
    cached_tracks: &[Track],
    track_entries: &std::collections::HashMap<String, cache::TrackEntry>,
    used_track_keys: &mut std::collections::HashSet<String>,
    used_track_ids: &mut std::collections::HashSet<String>,
    warn_on_dirs: bool,
) -> io::Result<Vec<Track>> {
    let mut tracks = Vec::new();
    let mut index = 1u8;
    let mut seen_tracks = std::collections::HashSet::new();
    let entries = sorted_track_paths(dir, warn_on_dirs)?;
    let cached_by_path: std::collections::HashMap<PathBuf, &Track> = cached_tracks
        .iter()
        .map(|track| (track.path.clone(), track))
        .collect();

    for path in entries {
        let dedupe_key = normalized_path_key(root, &path);
        if !seen_tracks.insert(dedupe_key) {
            warn!(path = %path.display(), "Skipping duplicate track path");
            continue;
        }
        if !is_audio_file(&path) {
            info!(path = %path.display(), "Ignoring non-audio file");
            continue;
        }

        let stem = match path.file_stem().and_then(|value| value.to_str()) {
            Some(stem) if !stem.trim().is_empty() => stem,
            Some(_) => {
                warn!(path = %path.display(), "Ignoring track with empty name");
                continue;
            }
            None => {
                warn!(path = %path.display(), "Ignoring track with unreadable name");
                continue;
            }
        };
        let (number, parsed_title) = parse_track_filename(stem);
        let mut track_number = number.unwrap_or_else(|| {
            let current = index;
            index = index.saturating_add(1);
            current
        });
        let mut title = parsed_title;

        let key = cache::track_key(root, &path);
        let id = cache::track_id(root, &path);
        used_track_keys.insert(key.clone());
        used_track_ids.insert(id.clone());
        let cached_track = cached_by_path.get(&path);
        let mut duration_secs = 0;
        let mut duration_millis = None;
        let mut bitrate_kbps = None;
        let mut codec = None;
        let mut artist = None;
        let mut album_artist = None;
        let mut compilation = false;
        let mut year = None;
        let mut genre = None;
        let mut embedded_cover = None;
        let mut used_cache = false;

        let signature = cache::track_signature(&path).ok();

        if let (Some(entry), Some(signature)) = (track_entries.get(&key), signature.as_ref()) {
            if entry.matches_signature(signature) {
                let cached = cache::load_track_metadata(root, entry.id()).ok().flatten();
                if let Some(cached) = cached {
                    if signature.matches_cache(&cached) {
                        let cached_track = cached.metadata().clone().into_track(path.clone());
                        track_number = cached_track.number;
                        title = cached_track.title;
                        duration_secs = cached_track.duration_secs;
                        duration_millis = cached_track.duration_millis;
                        bitrate_kbps = cached_track.bitrate_kbps;
                        codec = cached_track.codec;
                        artist = cached_track.artist;
                        album_artist = cached_track.album_artist;
                        compilation = cached_track.compilation;
                        year = cached_track.year;
                        genre = cached_track.genre;
                        embedded_cover = cached_track.embedded_cover;
                        used_cache = true;
                    }
                } else if let Some(cached_track) = cached_track {
                    track_number = cached_track.number;
                    title = cached_track.title.clone();
                    duration_secs = cached_track.duration_secs;
                    duration_millis = cached_track.duration_millis;
                    bitrate_kbps = cached_track.bitrate_kbps;
                    codec = cached_track.codec.clone();
                    artist = cached_track.artist.clone();
                    album_artist = cached_track.album_artist.clone();
                    compilation = cached_track.compilation;
                    year = cached_track.year;
                    genre = cached_track.genre.clone();
                    embedded_cover = cached_track.embedded_cover.clone();
                    used_cache = true;
                }
            }
        }

        if !used_cache {
            let metadata = metadata::track_metadata(&path);
            duration_secs = metadata.duration_secs.unwrap_or(0);
            duration_millis = metadata.duration_millis;
            bitrate_kbps = metadata.bitrate_kbps;
            codec = metadata.codec;
            artist = metadata.artist;
            album_artist = metadata.album_artist;
            compilation = metadata.compilation;
            year = metadata.year;
            genre = metadata.genre;
            embedded_cover = metadata.embedded_cover;
            if let Some(metadata_number) = metadata.track_number {
                track_number = metadata_number;
            }
            if let Some(metadata_title) = metadata.title {
                title = metadata_title;
            }
        }

        tracks.push(Track {
            number: track_number,
            title,
            duration_secs,
            duration_millis,
            bitrate_kbps,
            codec,
            path,
            artist,
            album_artist,
            compilation,
            year,
            genre,
            embedded_cover,
        });
    }

    tracks.sort_by_key(|track| track.number);
    Ok(tracks)
}

fn scan_root_album(
    root: &Path,
    use_cache: bool,
    state: &mut ScanState,
    root_artist_label: &str,
) -> io::Result<Option<Album>> {
    let album_path = root.to_path_buf();
    let cached_album = if use_cache {
        match cache::load_album(root, &album_path) {
            Ok(cached) => cached,
            Err(error) => {
                warn!(
                    error = %error,
                    path = %album_path.display(),
                    "Unable to load cached root album; rescanning"
                );
                None
            }
        }
    } else {
        None
    };

    let title = root_album_title(root);
    let mut album = if let Some(cached) = cached_album {
        let tracks = scan_tracks_with_cache_in_dir(
            root,
            &album_path,
            &cached.album.tracks,
            state.cache_index.track_entries(),
            &mut state.used_track_keys,
            &mut state.used_track_ids,
            false,
        )?;
        if tracks.is_empty() {
            return Ok(None);
        }
        let year = resolve_album_year(0, &tracks);
        let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
        let cover = select_album_cover(root, &album_path, &tracks, cached.album.cover.as_ref())?;
        let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
        Album {
            title,
            year,
            tracks,
            genre,
            path: album_path.clone(),
            total_duration_secs,
            cover,
        }
    } else {
        let tracks = scan_tracks_in_dir(&album_path, false)?;
        if tracks.is_empty() {
            return Ok(None);
        }
        state.record_track_keys(root, &tracks);
        state.record_track_ids(root, &tracks);
        let year = resolve_album_year(0, &tracks);
        let genre = dominant_genre(tracks.iter().flat_map(|track| track.genre.as_deref()));
        let cover = select_album_cover(root, &album_path, &tracks, None)?;
        let total_duration_secs = tracks.iter().map(|track| track.duration_secs).sum();
        Album {
            title,
            year,
            tracks,
            genre,
            path: album_path.clone(),
            total_duration_secs,
            cover,
        }
    };

    apply_user_metadata_override(root, root_artist_label, &mut album);
    state.record_metadata_key(root_artist_label, &album);
    state.record_cover_cache_file(&album);
    state.store_album(root, &album_path, &album);

    Ok(Some(album))
}

fn scan_cover_art(root: &Path, album_dir: &Path) -> io::Result<Option<CoverArt>> {
    let Some(source_path) = find_cover_file(album_dir)? else {
        return Ok(None);
    };

    let modified_secs = file_modified_secs(&source_path)?;
    let cache_dir = cache::ensure_cover_cache_dir(root)?;
    let cache_filename = cache_cover_filename(&source_path, modified_secs);
    let cached_path = cache_dir.join(cache_filename);

    if !cached_path.exists() {
        fs::copy(&source_path, &cached_path)?;
    }

    Ok(Some(CoverArt {
        source_path,
        cached_path,
        modified_secs,
    }))
}

fn find_cover_file(album_dir: &Path) -> io::Result<Option<PathBuf>> {
    let mut candidates = Vec::new();
    let read_dir = match fs::read_dir(album_dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            warn!(
                error = %error,
                path = %album_dir.display(),
                "Skipping cover scan: unable to read directory"
            );
            return Ok(None);
        }
    };

    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    error = %error,
                    path = %album_dir.display(),
                    "Skipping unreadable entry during cover scan"
                );
                continue;
            }
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        let extension = extension.to_lowercase();
        if !matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();
        candidates.push((cover_priority(&stem), path));
    }

    candidates.sort_by(|(a_priority, a_path), (b_priority, b_path)| {
        a_priority.cmp(b_priority).then_with(|| a_path.cmp(b_path))
    });

    Ok(candidates.into_iter().map(|(_, path)| path).next())
}

fn cover_priority(stem: &str) -> usize {
    const PRIORITY: [&str; 5] = ["cover", "folder", "front", "artwork", "album"];
    PRIORITY
        .iter()
        .position(|label| *label == stem)
        .unwrap_or(PRIORITY.len())
}

fn cache_cover_filename(path: &Path, modified_secs: u64) -> String {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("img");
    cache_cover_filename_with_extension(path, modified_secs, extension)
}

fn cache_cover_filename_with_extension(path: &Path, modified_secs: u64, extension: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    modified_secs.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{hash:x}.{extension}")
}

fn file_modified_secs(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(duration.as_secs())
}

fn read_sorted_dirs(root: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let read_dir = match fs::read_dir(root) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            warn!(
                error = %error,
                path = %root.display(),
                "Skipping directory scan: unable to read directory"
            );
            return Ok(Vec::new());
        }
    };

    let mut entries = Vec::new();
    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    error = %error,
                    path = %root.display(),
                    "Skipping unreadable entry during directory scan"
                );
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            entries.push(entry);
        } else {
            warn!(
                path = %path.display(),
                "Ignoring non-conforming entry; expected a directory"
            );
        }
    }
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn sorted_track_paths(dir: &Path, warn_on_dirs: bool) -> io::Result<Vec<PathBuf>> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            warn!(
                error = %error,
                path = %dir.display(),
                "Skipping tracks scan: unable to read directory"
            );
            return Ok(Vec::new());
        }
    };

    let mut entries = Vec::new();
    for entry_result in read_dir {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    entries.push(path);
                } else if path.is_dir() && warn_on_dirs {
                    warn!(
                        path = %path.display(),
                        "Ignoring non-conforming subdirectory inside album"
                    );
                }
            }
            Err(error) => {
                warn!(
                    error = %error,
                    path = %dir.display(),
                    "Skipping unreadable entry"
                );
            }
        }
    }

    entries.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    Ok(entries)
}

fn dir_has_audio_files(dir: &Path) -> io::Result<bool> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            warn!(
                error = %error,
                path = %dir.display(),
                "Skipping directory scan: unable to read directory"
            );
            return Ok(false);
        }
    };

    for entry_result in read_dir {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && is_audio_file(&path) {
                    return Ok(true);
                }
            }
            Err(error) => {
                warn!(
                    error = %error,
                    path = %dir.display(),
                    "Skipping unreadable entry during directory scan"
                );
            }
        }
    }

    Ok(false)
}

fn dominant_genre<'a>(genres: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts: std::collections::HashMap<&'a str, usize> = std::collections::HashMap::new();
    for genre in genres {
        for trimmed in split_genre_field(genre) {
            *counts.entry(trimmed).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(genre, _)| genre.to_string())
}

pub(crate) fn split_genre_field(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|ch| matches!(ch, ';' | '/' | '\\' | ',' | '|'))
        .map(|genre| genre.trim())
        .filter(|genre| !genre.is_empty())
}

fn is_audio_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(str::to_lowercase),
        Some(ext)
            if matches!(
                ext.as_str(),
                "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus" | "aif" | "aiff"
                    | "wma"
            )
    )
}

fn normalized_path_key(root: &Path, path: &Path) -> String {
    let normalized = match path.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            warn!(
                error = %error,
                path = %path.display(),
                "Failed to canonicalize path for deduplication"
            );
            path.to_path_buf()
        }
    };
    let relative = normalized.strip_prefix(root).unwrap_or(&normalized);
    relative.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn root_album_title(root: &Path) -> String {
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            let fallback = root.to_string_lossy();
            let trimmed = fallback.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
    name.unwrap_or_else(|| "Library".to_string())
}

fn parse_album_folder(name: &str) -> (u16, String) {
    let trimmed = name.trim();
    let mut year_end = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            year_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if year_end >= 4 {
        let year_part = &trimmed[..year_end];
        if let Ok(year) = year_part.parse::<u16>() {
            let title = trimmed[year_end..]
                .trim_start_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
                .trim();
            let title = if title.is_empty() { trimmed } else { title };
            return (year, title.to_string());
        }
    }

    (0, trimmed.to_string())
}

fn parse_track_filename(name: &str) -> (Option<u8>, String) {
    let trimmed = name.trim();
    let mut number_end = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            number_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if number_end > 0 {
        let number_part = &trimmed[..number_end];
        let title = trimmed[number_end..]
            .trim_start_matches(|c: char| c == '-' || c == '_' || c == '.' || c.is_whitespace())
            .trim();
        let title = if title.is_empty() { trimmed } else { title };
        return (number_part.parse::<u8>().ok(), title.to_string());
    }

    (None, trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UserSettings;
    use crate::config::InterfaceLanguage;
    use crate::ui::i18n;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_wav(path: &Path, duration_secs: u32) -> io::Result<()> {
        let sample_rate = 44_100u32;
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let bytes_per_sample = bits_per_sample / 8;
        let num_samples = sample_rate * duration_secs;
        let data_size = num_samples * bytes_per_sample as u32 * num_channels as u32;
        let riff_size = 36 + data_size;
        let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
        let block_align = num_channels * bytes_per_sample;

        let mut file = File::create(path)?;
        file.write_all(b"RIFF")?;
        file.write_all(&riff_size.to_le_bytes())?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        file.write_all(&vec![0u8; data_size as usize])?;
        Ok(())
    }

    #[test]
    fn parse_track_filename_extracts_number_and_title() {
        let (number, title) = parse_track_filename("01 - Lumiere");
        assert_eq!(number, Some(1));
        assert_eq!(title, "Lumiere");

        let (number, title) = parse_track_filename("7_Titre");
        assert_eq!(number, Some(7));
        assert_eq!(title, "Titre");

        let (number, title) = parse_track_filename("Sans numero");
        assert_eq!(number, None);
        assert_eq!(title, "Sans numero");
    }

    #[test]
    fn scan_library_builds_catalog_from_folders() {
        let dir = tempdir().expect("tempdir");
        let artist_dir = dir.path().join("Artiste");
        let album_dir = artist_dir.join("2022 - Album");
        fs::create_dir_all(&album_dir).expect("create album dirs");

        File::create(album_dir.join("01 - Intro.mp3")).expect("create track");
        File::create(album_dir.join("02 - Suite.flac")).expect("create track");
        File::create(album_dir.join("notes.txt")).expect("create note");

        let root_artist_label = i18n::strings(InterfaceLanguage::English).root_artist_label;
        let catalog =
            scan_library(dir.path(), &UserSettings::default(), root_artist_label)
                .expect("scan library");
        assert_eq!(catalog.artists.len(), 1);
        let artist = &catalog.artists[0];
        assert_eq!(artist.name, "Artiste");
        assert_eq!(artist.albums.len(), 1);
        let album = &artist.albums[0];
        assert_eq!(album.year, 2022);
        assert_eq!(album.title, "Album");
        assert_eq!(album.tracks.len(), 2);
        assert_eq!(album.tracks[0].title, "Intro");
        assert_eq!(album.tracks[1].number, 2);
    }

    #[test]
    fn scan_library_supports_root_level_tracks() {
        let dir = tempdir().expect("tempdir");
        File::create(dir.path().join("01 - Racine.mp3")).expect("create track");
        File::create(dir.path().join("02 - Autre.flac")).expect("create track");

        let root_artist_label = i18n::strings(InterfaceLanguage::English).root_artist_label;
        let catalog =
            scan_library(dir.path(), &UserSettings::default(), root_artist_label)
                .expect("scan library");
        assert_eq!(catalog.artists.len(), 1);
        let artist = &catalog.artists[0];
        assert_eq!(artist.name, root_artist_label);
        assert_eq!(artist.albums.len(), 1);
        let album = &artist.albums[0];
        assert_eq!(album.title, root_album_title(dir.path()));
        assert_eq!(album.year, 0);
        assert_eq!(album.tracks.len(), 2);
    }

    #[test]
    fn scan_library_supports_albums_directly_under_root() {
        let dir = tempdir().expect("tempdir");
        let album_dir = dir.path().join("2023 - Album Racine");
        fs::create_dir_all(&album_dir).expect("create album dir");
        File::create(album_dir.join("01 - Intro.mp3")).expect("create track");
        File::create(album_dir.join("02 - Suite.flac")).expect("create track");

        let root_artist_label = i18n::strings(InterfaceLanguage::English).root_artist_label;
        let catalog =
            scan_library(dir.path(), &UserSettings::default(), root_artist_label)
                .expect("scan library");
        assert_eq!(catalog.artists.len(), 1);
        let artist = &catalog.artists[0];
        assert_eq!(artist.name, root_artist_label);
        assert_eq!(artist.albums.len(), 1);
        let album = &artist.albums[0];
        assert_eq!(album.title, "Album Racine");
        assert_eq!(album.year, 2023);
        assert_eq!(album.tracks.len(), 2);
    }

    #[test]
    fn scan_tracks_reads_audio_duration() {
        let dir = tempdir().expect("tempdir");
        let wav_path = dir.path().join("01 - Test.wav");
        write_wav(&wav_path, 1).expect("write wav");

        let tracks = scan_tracks(dir.path()).expect("scan tracks");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].duration_secs, 1);
    }

    #[test]
    fn scan_tracks_empty_directory_returns_empty_list() {
        let dir = tempdir().expect("tempdir");
        let tracks = scan_tracks(dir.path()).expect("scan tracks");
        assert!(tracks.is_empty());
    }

    #[test]
    fn scan_library_keeps_album_without_year() {
        let dir = tempdir().expect("tempdir");
        let artist_dir = dir.path().join("Artiste");
        let album_dir = artist_dir.join("Album sans annee");
        fs::create_dir_all(&album_dir).expect("create album dirs");
        File::create(album_dir.join("Intro.mp3")).expect("create track");

        let root_artist_label = i18n::strings(InterfaceLanguage::English).root_artist_label;
        let catalog =
            scan_library(dir.path(), &UserSettings::default(), root_artist_label)
                .expect("scan library");
        let album = &catalog.artists[0].albums[0];
        assert_eq!(album.year, 0);
        assert_eq!(album.title, "Album sans annee");
    }

    #[test]
    fn scan_tracks_handles_missing_numbers_and_mixed_extensions() {
        let dir = tempdir().expect("tempdir");
        File::create(dir.path().join("Intro.mp3")).expect("create track");
        File::create(dir.path().join("02 - Suite.FLAC")).expect("create track");
        File::create(dir.path().join("notes.txt")).expect("create note");

        let tracks = scan_tracks(dir.path()).expect("scan tracks");
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].number, 1);
        assert_eq!(tracks[0].title, "Intro");
        assert_eq!(tracks[1].number, 2);
    }

    #[cfg(unix)]
    #[test]
    fn scan_tracks_skips_unreadable_filenames() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let dir = tempdir().expect("tempdir");
        let mut bytes = vec![0xff, 0xfe];
        bytes.extend_from_slice(b".mp3");
        let invalid_name = OsString::from_vec(bytes);
        let path = dir.path().join(invalid_name);
        File::create(&path).expect("create invalid track");

        let tracks = scan_tracks(dir.path()).expect("scan tracks");
        assert!(tracks.is_empty());
    }

    #[test]
    fn parse_album_folder_extracts_year_and_title() {
        assert_eq!(parse_album_folder("2022 - Album"), (2022, "Album".to_string()));
        assert_eq!(parse_album_folder("1999_Titre"), (1999, "Titre".to_string()));
        assert_eq!(parse_album_folder("Album sans annee"), (0, "Album sans annee".to_string()));
        assert_eq!(parse_album_folder("  2001 - Spaced  "), (2001, "Spaced".to_string()));
        assert_eq!(parse_album_folder("12 Short"), (0, "12 Short".to_string()));
    }

    #[test]
    fn split_genre_field_handles_delimiters() {
        let genres: Vec<&str> = split_genre_field("Rock; Pop / Jazz").collect();
        assert_eq!(genres, vec!["Rock", "Pop", "Jazz"]);

        let genres: Vec<&str> = split_genre_field("Electronic|Ambient").collect();
        assert_eq!(genres, vec!["Electronic", "Ambient"]);

        let genres: Vec<&str> = split_genre_field("Solo").collect();
        assert_eq!(genres, vec!["Solo"]);

        let genres: Vec<&str> = split_genre_field("").collect();
        assert!(genres.is_empty());
    }

    #[test]
    fn dominant_genre_picks_most_frequent() {
        let genres = vec!["Rock", "Pop", "Rock", "Jazz", "Rock", "Pop"];
        assert_eq!(dominant_genre(genres.into_iter()), Some("Rock".to_string()));

        let genres: Vec<&str> = vec![];
        assert_eq!(dominant_genre(genres.into_iter()), None);
    }

    #[test]
    fn dominant_year_picks_most_frequent() {
        assert_eq!(dominant_year(vec![2020, 2020, 2021].into_iter()), Some(2020));
        assert_eq!(dominant_year(vec![0, 0, 0].into_iter()), None);
        assert_eq!(dominant_year(std::iter::empty()), None);
    }

    #[test]
    fn is_audio_file_accepts_known_extensions() {
        assert!(is_audio_file(Path::new("song.mp3")));
        assert!(is_audio_file(Path::new("track.FLAC")));
        assert!(is_audio_file(Path::new("file.opus")));
        assert!(is_audio_file(Path::new("file.aiff")));
        assert!(!is_audio_file(Path::new("notes.txt")));
        assert!(!is_audio_file(Path::new("cover.jpg")));
        assert!(!is_audio_file(Path::new("no_extension")));
    }

    #[test]
    fn resolve_album_year_uses_folder_year_when_available() {
        assert_eq!(resolve_album_year(2020, &[]), 2020);
    }

    fn make_track(number: u8, title: &str, year: Option<u16>) -> Track {
        Track {
            number,
            title: title.to_string(),
            duration_secs: 10,
            duration_millis: None,
            bitrate_kbps: None,
            codec: None,
            path: PathBuf::from(format!("{title}.mp3")),
            artist: None,
            album_artist: None,
            compilation: false,
            year,
            genre: None,
            embedded_cover: None,
        }
    }

    #[test]
    fn resolve_album_year_falls_back_to_track_year() {
        let tracks = vec![make_track(1, "A", Some(2019)), make_track(2, "B", Some(2019))];
        assert_eq!(resolve_album_year(0, &tracks), 2019);
    }

    fn track_with_tags(
        number: u8,
        artist: Option<&str>,
        album_artist: Option<&str>,
        compilation: bool,
    ) -> Track {
        Track {
            number,
            title: format!("t{number}"),
            duration_secs: 10,
            duration_millis: None,
            bitrate_kbps: None,
            codec: None,
            path: PathBuf::from(format!("t{number}.mp3")),
            artist: artist.map(str::to_string),
            album_artist: album_artist.map(str::to_string),
            compilation,
            year: None,
            genre: None,
            embedded_cover: None,
        }
    }

    #[test]
    fn infer_album_artist_uses_common_album_artist_tag() {
        let tracks = vec![
            track_with_tags(1, Some("Mothervibes"), Some("SHIFT UP"), false),
            track_with_tags(2, Some("Hyunmin Cho"), Some("SHIFT UP"), false),
            track_with_tags(3, Some("Youngjee Lee"), Some("SHIFT UP"), false),
        ];
        assert_eq!(infer_album_artist(&tracks), Some("SHIFT UP".to_string()));
    }

    #[test]
    fn infer_album_artist_falls_back_to_common_track_artist() {
        let tracks = vec![
            track_with_tags(1, Some("The Artist"), None, false),
            track_with_tags(2, Some("The Artist"), None, false),
        ];
        assert_eq!(infer_album_artist(&tracks), Some("The Artist".to_string()));
    }

    #[test]
    fn infer_album_artist_flags_compilation_even_when_tags_agree() {
        let tracks = vec![
            track_with_tags(1, Some("A"), Some("Same Artist"), true),
            track_with_tags(2, Some("B"), Some("Same Artist"), false),
        ];
        assert_eq!(
            infer_album_artist(&tracks),
            Some("Various Artists".to_string())
        );
    }

    #[test]
    fn infer_album_artist_returns_none_when_nothing_is_shared() {
        let tracks = vec![
            track_with_tags(1, Some("Artist A"), Some("Label A"), false),
            track_with_tags(2, Some("Artist B"), Some("Label B"), false),
        ];
        assert_eq!(infer_album_artist(&tracks), None);
    }

    #[test]
    fn infer_album_artist_ignores_none_entries() {
        // Two tracks have the tag, one is untagged. The untagged track must
        // not block consensus on the tagged value.
        let tracks = vec![
            track_with_tags(1, None, Some("Hidden Composer"), false),
            track_with_tags(2, None, Some("Hidden Composer"), false),
            track_with_tags(3, None, None, false),
        ];
        assert_eq!(
            infer_album_artist(&tracks),
            Some("Hidden Composer".to_string())
        );
    }
}
