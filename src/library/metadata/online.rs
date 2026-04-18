use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::warn;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::config::UserSettings;
use crate::library::cache;

const INITIAL_BACKOFF_SECS: u64 = 30;
const MAX_BACKOFF_SECS: u64 = 60 * 60;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnlineMetadata {
    pub genre: Option<String>,
    pub year: Option<u16>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetadataOverride {
    pub genre: Option<String>,
    pub year: Option<u16>,
    #[serde(default)]
    pub genre_overridden: bool,
    #[serde(default)]
    pub year_overridden: bool,
    #[serde(default)]
    pub edited_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedOnlineMetadata {
    fetched_at: u64,
    metadata: OnlineMetadata,
    #[serde(default)]
    user_override: Option<UserMetadataOverride>,
    #[serde(default)]
    backoff_until: u64,
    #[serde(default)]
    backoff_secs: u64,
}

pub fn load_user_metadata_override(
    root: &Path,
    artist: &str,
    album: &str,
) -> io::Result<Option<UserMetadataOverride>> {
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_key = metadata_cache_key(artist, album);
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    if !cache_path.exists() {
        return Ok(None);
    }
    Ok(load_cached_metadata(&cache_path).and_then(|entry| entry.user_override))
}

pub fn store_user_metadata_override(
    root: &Path,
    artist: &str,
    album: &str,
    mut metadata_override: UserMetadataOverride,
) -> io::Result<()> {
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_key = metadata_cache_key(artist, album);
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    let existing = if cache_path.exists() {
        load_cached_metadata(&cache_path)
    } else {
        None
    };
    metadata_override.edited_at = current_epoch_secs();
    let payload = CachedOnlineMetadata {
        fetched_at: existing.as_ref().map(|entry| entry.fetched_at).unwrap_or(0),
        metadata: existing
            .as_ref()
            .map(|entry| entry.metadata.clone())
            .unwrap_or_default(),
        user_override: Some(metadata_override),
        backoff_until: existing.as_ref().map(|entry| entry.backoff_until).unwrap_or(0),
        backoff_secs: existing.as_ref().map(|entry| entry.backoff_secs).unwrap_or(0),
    };
    write_metadata_cache(&cache_path, &payload)
}

pub async fn fetch_album_metadata(
    root: &Path,
    settings: &UserSettings,
    artist: &str,
    album: &str,
    force_refresh: bool,
) -> io::Result<Option<OnlineMetadata>> {
    let api_key = settings.metadata_api_key.trim();
    if api_key.is_empty() {
        return Ok(None);
    }

    let cache_key = metadata_cache_key(artist, album);
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    let ttl_secs = u64::from(settings.metadata_cache_ttl_hours).saturating_mul(3600);
    let now_secs = current_epoch_secs();

    let cached = if cache_path.exists() {
        load_cached_metadata(&cache_path)
    } else {
        None
    };

    if !force_refresh {
        if let Some(entry) = &cached {
            if ttl_secs > 0 && now_secs.saturating_sub(entry.fetched_at) < ttl_secs {
                return Ok(Some(entry.metadata.clone()));
            }
        }
    }

    if let Some(entry) = &cached {
        if entry.backoff_until > now_secs {
            warn!(
                backoff_until = entry.backoff_until,
                backoff_secs = entry.backoff_secs,
                "Skipping online metadata fetch due to active backoff"
            );
            if entry.fetched_at == 0 && entry.metadata == OnlineMetadata::default() {
                return Ok(None);
            }
            return Ok(Some(entry.metadata.clone()));
        }
    }

    let base_metadata = cached
        .as_ref()
        .map(|entry| entry.metadata.clone())
        .unwrap_or_default();
    let metadata = match enrich_with_lastfm_metadata(
        base_metadata.clone(),
        api_key,
        artist,
        album,
    )
    .await
    {
        Ok(metadata) => metadata,
        Err(error) => match error {
            MetadataFetchError::RateLimited { status } => {
                let (payload, fallback) = build_rate_limit_payload(
                    cached.as_ref(),
                    now_secs,
                    status,
                    base_metadata,
                );
                if let Err(error) = write_metadata_cache(&cache_path, &payload) {
                    warn!(error = %error, "Failed to write online metadata cache");
                }
                return Ok(fallback);
            }
            MetadataFetchError::Http(error) => {
                if error.is_timeout() {
                    warn!(error = %error, "Timed out fetching online metadata");
                } else {
                    warn!(error = %error, "Failed to fetch online metadata");
                }
                return Ok(cached.map(|entry| entry.metadata));
            }
        },
    };

    let payload = CachedOnlineMetadata {
        fetched_at: now_secs,
        metadata: metadata.clone(),
        user_override: cached.and_then(|entry| entry.user_override),
        backoff_until: 0,
        backoff_secs: 0,
    };

    if let Err(error) = write_metadata_cache(&cache_path, &payload) {
        warn!(error = %error, "Failed to write online metadata cache");
    }

    Ok(Some(metadata))
}

async fn enrich_with_lastfm_metadata(
    mut metadata: OnlineMetadata,
    api_key: &str,
    artist: &str,
    album: &str,
) -> Result<OnlineMetadata, MetadataFetchError> {
    let lastfm_metadata = fetch_lastfm_metadata(api_key, artist, album).await?;
    if metadata.genre.is_none() {
        metadata.genre = lastfm_metadata.genre;
    }
    if metadata.year.is_none() {
        metadata.year = lastfm_metadata.year;
    }
    Ok(metadata)
}

async fn fetch_lastfm_metadata(
    api_key: &str,
    artist: &str,
    album: &str,
) -> Result<OnlineMetadata, MetadataFetchError> {
    let user_agent = format!("Grape/{}", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(user_agent)
        .build()
        .map_err(MetadataFetchError::Http)?;
    let response = client
        .get("https://ws.audioscrobbler.com/2.0/")
        .query(&[
            ("method", "album.getInfo"),
            ("api_key", api_key),
            ("artist", artist),
            ("album", album),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(MetadataFetchError::Http)?;

    let status = response.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
    {
        return Err(MetadataFetchError::RateLimited { status });
    }

    let response = response.error_for_status().map_err(MetadataFetchError::Http)?;

    let payload: serde_json::Value = response.json().await.map_err(MetadataFetchError::Http)?;
    let genre = extract_genre(&payload);
    let year = extract_year(&payload);

    Ok(OnlineMetadata { genre, year })
}

fn extract_genre(payload: &serde_json::Value) -> Option<String> {
    let tag_value = payload
        .pointer("/album/toptags/tag")
        .or_else(|| payload.pointer("/album/tags/tag"));

    match tag_value {
        Some(value) if value.is_array() => value
            .as_array()
            .and_then(|tags| tags.iter().find_map(|tag| tag.get("name")))
            .and_then(|name| name.as_str())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        Some(value) if value.is_object() => value
            .get("name")
            .and_then(|name| name.as_str())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        _ => None,
    }
}

fn extract_year(payload: &serde_json::Value) -> Option<u16> {
    let release = payload
        .pointer("/album/releasedate")
        .and_then(|value| value.as_str())
        .or_else(|| {
            payload
                .pointer("/album/wiki/published")
                .and_then(|value| value.as_str())
        });
    release.and_then(parse_year)
}

fn parse_year(value: &str) -> Option<u16> {
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            if digits.len() == 4 {
                if let Ok(year) = digits.parse::<u16>() {
                    if year > 0 {
                        return Some(year);
                    }
                }
                digits.clear();
            }
        } else {
            digits.clear();
        }
    }
    None
}

fn metadata_cache_key(artist: &str, album: &str) -> String {
    fn normalize_key_part(value: &str) -> String {
        value
            .nfkd()
            .filter(|character| !is_combining_mark(*character))
            .collect::<String>()
            .to_lowercase()
    }

    let input = format!(
        "{}::{}",
        normalize_key_part(artist.trim()),
        normalize_key_part(album.trim())
    );
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hash;
    use std::hash::Hasher;
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub(crate) fn metadata_cache_key_for_album(artist: &str, album: &str) -> String {
    metadata_cache_key(artist, album)
}

fn load_cached_metadata(path: &Path) -> Option<CachedOnlineMetadata> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<CachedOnlineMetadata>(&contents) {
            Ok(entry) => Some(entry),
            Err(error) => {
                warn!(error = %error, "Failed to parse cached online metadata");
                None
            }
        },
        Err(error) => {
            warn!(error = %error, "Failed to read cached online metadata");
            None
        }
    }
}

fn write_metadata_cache(path: &Path, payload: &CachedOnlineMetadata) -> io::Result<()> {
    let contents = serde_json::to_string_pretty(payload)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, contents)?;
    fs::rename(&temp_path, path)
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug)]
enum MetadataFetchError {
    Http(reqwest::Error),
    RateLimited { status: reqwest::StatusCode },
}

fn build_rate_limit_payload(
    cached: Option<&CachedOnlineMetadata>,
    now_secs: u64,
    status: reqwest::StatusCode,
    base_metadata: OnlineMetadata,
) -> (CachedOnlineMetadata, Option<OnlineMetadata>) {
    let previous_backoff = cached.map(|entry| entry.backoff_secs).unwrap_or(0);
    let next_backoff = next_backoff_secs(previous_backoff);
    let backoff_until = now_secs.saturating_add(next_backoff);
    warn!(
        status = %status,
        backoff_secs = next_backoff,
        backoff_until = backoff_until,
        "Received rate limit response for online metadata"
    );

    let payload = CachedOnlineMetadata {
        fetched_at: cached.map(|entry| entry.fetched_at).unwrap_or(0),
        metadata: cached
            .map(|entry| entry.metadata.clone())
            .unwrap_or(base_metadata),
        user_override: cached.and_then(|entry| entry.user_override.clone()),
        backoff_until,
        backoff_secs: next_backoff,
    };

    let fallback = cached.map(|entry| entry.metadata.clone());
    (payload, fallback)
}

fn next_backoff_secs(previous_backoff: u64) -> u64 {
    if previous_backoff == 0 {
        INITIAL_BACKOFF_SECS
    } else {
        previous_backoff
            .saturating_mul(2)
            .min(MAX_BACKOFF_SECS)
    }
}
