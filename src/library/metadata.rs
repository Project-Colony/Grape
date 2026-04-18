use std::collections::HashSet;
use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::PictureType;
use lofty::tag::ItemKey;
use tracing::warn;

use crate::library::EmbeddedCover;

pub mod online;

pub const MAX_EMBEDDED_COVER_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Default, Clone)]
pub struct TrackMetadata {
    pub duration_secs: Option<u32>,
    pub duration_millis: Option<u64>,
    pub bitrate_kbps: Option<u32>,
    pub codec: Option<String>,
    pub title: Option<String>,
    pub track_number: Option<u8>,
    /// The ARTIST tag of this individual track (not the album artist).
    pub artist: Option<String>,
    /// The ALBUMARTIST tag, used to group albums under a common artist even
    /// when individual track ARTIST values differ (soundtracks, compilations,
    /// splits). Kept distinct from `artist` so callers can apply standard
    /// music-player precedence rules.
    pub album_artist: Option<String>,
    /// iTunes/Vorbis compilation flag (TCMP / COMPILATION). When true the
    /// album should be grouped under a "Various Artists" virtual artist
    /// instead of deriving the artist from individual tracks.
    pub compilation: bool,
    pub genre: Option<String>,
    pub year: Option<u16>,
    pub embedded_cover: Option<EmbeddedCover>,
}

pub fn track_metadata(path: &Path) -> TrackMetadata {
    let tagged_file = match lofty::read_from_path(path) {
        Ok(tagged_file) => tagged_file,
        Err(error) => {
            warn!(
                error = %error,
                path = %path.display(),
                "Failed to decode audio metadata"
            );
            return TrackMetadata::default();
        }
    };

    let properties = tagged_file.properties();
    let duration = properties.duration();
    let duration_secs = u32::try_from(duration.as_secs()).ok();
    let duration_millis = u64::try_from(duration.as_millis()).ok();
    let bitrate_kbps = properties
        .audio_bitrate()
        .or_else(|| properties.overall_bitrate())
        .filter(|bitrate| *bitrate > 0);
    let codec = Some(format!("{:?}", tagged_file.file_type()));

    let title = extract_first_string(&tagged_file, &[ItemKey::TrackTitle]);
    let track_number = extract_track_number(&tagged_file);
    // Track artist and album artist are intentionally kept separate. Callers
    // use `album_artist` to group albums; the UI displays `artist` as the
    // per-track performer. Merging them lost information and polluted the
    // track subtitle with the album artist name ("Mothervibes; SHIFT UP").
    let artist = extract_first_string(
        &tagged_file,
        &[ItemKey::TrackArtist, ItemKey::TrackArtists],
    )
    .and_then(|value| split_artist_field(&value));
    let album_artist = extract_first_string(&tagged_file, &[ItemKey::AlbumArtist])
        .and_then(|value| split_artist_field(&value));
    let compilation = extract_compilation_flag(&tagged_file);
    let genre = extract_genre(&tagged_file);
    let year = extract_year(&tagged_file);

    let embedded_cover = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .and_then(|tag| {
            tag.get_picture_type(PictureType::CoverFront)
                .or_else(|| tag.pictures().first())
        })
        .and_then(|picture| {
            let data = picture.data();
            if data.is_empty() {
                return None;
            }
            if data.len() > MAX_EMBEDDED_COVER_BYTES {
                warn!(
                    path = %path.display(),
                    size_bytes = data.len(),
                    max_bytes = MAX_EMBEDDED_COVER_BYTES,
                    "Ignoring embedded cover larger than maximum size"
                );
                return None;
            }
            Some(EmbeddedCover {
                mime_type: picture.mime_type().map(|mime| mime.as_str().to_string()),
                data: data.to_vec(),
            })
        });

    TrackMetadata {
        duration_secs,
        duration_millis,
        bitrate_kbps,
        codec,
        title,
        track_number,
        artist,
        album_artist,
        compilation,
        genre,
        year,
        embedded_cover,
    }
}

fn extract_compilation_flag(tagged_file: &impl TaggedFileExt) -> bool {
    let mut tags = Vec::new();
    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }
    tags.extend(tagged_file.tags());

    for tag in tags {
        for value in tag.get_strings(&ItemKey::FlagCompilation) {
            let trimmed = value.trim();
            if matches!(trimmed, "1" | "true" | "True" | "TRUE" | "yes" | "Yes" | "YES") {
                return true;
            }
        }
    }
    false
}

pub fn merge_genre(
    local_genre: Option<String>,
    online_genre: Option<String>,
    enrichment_confirmed: bool,
) -> Option<String> {
    if enrichment_confirmed {
        online_genre.or(local_genre)
    } else if local_genre.is_some() {
        local_genre
    } else {
        online_genre
    }
}

pub fn merge_year(local_year: u16, online_year: Option<u16>, enrichment_confirmed: bool) -> u16 {
    if enrichment_confirmed {
        online_year.unwrap_or(local_year)
    } else if local_year > 0 {
        local_year
    } else {
        online_year.unwrap_or(0)
    }
}

fn extract_genre(tagged_file: &impl TaggedFileExt) -> Option<String> {
    let mut genres = Vec::new();
    let mut seen = HashSet::new();
    let mut tags = Vec::new();

    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }

    tags.extend(tagged_file.tags());

    for tag in tags {
        let values = tag
            .get_strings(&ItemKey::Genre)
            .chain(tag.get_locators(&ItemKey::Genre));

        for value in values {
            for genre in split_genre_field(value) {
                let normalized = genre.to_lowercase();
                if seen.insert(normalized) {
                    genres.push(genre.to_string());
                }
            }
        }
    }

    if genres.is_empty() {
        None
    } else {
        Some(genres.join("; "))
    }
}

fn extract_first_string(tagged_file: &impl TaggedFileExt, keys: &[ItemKey]) -> Option<String> {
    let mut tags = Vec::new();
    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }
    tags.extend(tagged_file.tags());

    for tag in tags {
        for key in keys {
            let mut values = tag.get_strings(key);
            if let Some(value) = values.next() {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

fn split_artist_field(value: &str) -> Option<String> {
    value
        .split(|ch| matches!(ch, ';' | '/' | '\\' | ',' | '|'))
        .map(|artist| artist.trim())
        .find(|artist| !artist.is_empty())
        .map(|artist| artist.to_string())
}

fn extract_track_number(tagged_file: &impl TaggedFileExt) -> Option<u8> {
    let mut tags = Vec::new();
    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }
    tags.extend(tagged_file.tags());

    for tag in tags {
        for value in tag.get_strings(&ItemKey::TrackNumber) {
            if let Some(number) = parse_numeric_prefix(value) {
                if number > 0 {
                    return u8::try_from(number).ok();
                }
            }
        }
    }
    None
}

fn extract_year(tagged_file: &impl TaggedFileExt) -> Option<u16> {
    let mut tags = Vec::new();
    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }
    tags.extend(tagged_file.tags());

    let keys = [
        ItemKey::RecordingDate,
        ItemKey::ReleaseDate,
        ItemKey::OriginalReleaseDate,
        ItemKey::Year,
    ];

    for tag in tags {
        for key in &keys {
            for value in tag.get_strings(key) {
                if let Some(year) = parse_year(value) {
                    return Some(year);
                }
            }
        }
    }
    None
}

fn parse_numeric_prefix(value: &str) -> Option<u32> {
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
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

fn split_genre_field(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|ch| matches!(ch, ';' | '/' | '\\' | ',' | '|'))
        .map(|genre| genre.trim())
        .filter(|genre| !genre.is_empty())
}
