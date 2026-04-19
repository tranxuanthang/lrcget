use crate::persistent_entities::PersistentTrack;
use crate::utils::strip_timestamp;
use anyhow::Result;
use lrc::Lyrics;
use serde::{Deserialize, Serialize};

pub const LYRICSFILE_VERSION: &str = "1.0";
pub const INSTRUMENTAL_LRC: &str = "[au: instrumental]";

#[derive(Debug, Clone, Copy, Default)]
pub struct LyricsPresence {
    pub has_plain_lyrics: bool,
    pub has_synced_lyrics: bool,
    pub has_word_synced_lyrics: bool,
    pub is_instrumental: bool,
}

#[derive(Debug, Clone)]
pub struct LyricsfileTrackMetadata {
    pub title: String,
    pub album_name: String,
    pub artist_name: String,
    pub duration: f64,
}

impl LyricsfileTrackMetadata {
    pub fn from_persistent_track(track: &PersistentTrack) -> Self {
        Self {
            title: track.title.clone(),
            album_name: track.album_name.clone(),
            artist_name: track.artist_name.clone(),
            duration: track.duration,
        }
    }

    pub fn new(title: &str, album_name: &str, artist_name: &str, duration: f64) -> Self {
        Self {
            title: title.to_string(),
            album_name: album_name.to_string(),
            artist_name: artist_name.to_string(),
            duration,
        }
    }
}

#[derive(Debug)]
pub struct ParsedLyricsfile {
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub is_instrumental: bool,
}

#[derive(Debug, Serialize)]
struct LyricsfileDocument {
    version: String,
    metadata: LyricsfileMetadata,
    lines: Vec<LyricsfileLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LyricsfileMetadata {
    title: String,
    artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    instrumental: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LyricsfileLine {
    text: String,
    start_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    words: Vec<LyricsfileWord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LyricsfileWord {
    text: String,
    start_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ParsedLyricsfileDocument {
    metadata: ParsedLyricsfileMetadata,
    #[serde(default)]
    lines: Vec<LyricsfileLine>,
    plain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParsedLyricsfileMetadata {
    #[serde(default)]
    instrumental: bool,
}

pub fn build_lyricsfile(
    metadata: &LyricsfileTrackMetadata,
    plain_lyrics: Option<&str>,
    synced_lyrics: Option<&str>,
) -> Option<String> {
    let plain = normalize_non_empty(plain_lyrics);
    let synced = normalize_non_empty(synced_lyrics);
    let is_instrumental = synced
        .as_ref()
        .map(|value| is_instrumental_lyrics(value))
        .unwrap_or(false);

    if plain.is_none() && synced.is_none() {
        return None;
    }

    let synced_lines = if is_instrumental {
        Vec::new()
    } else {
        synced
            .as_ref()
            .map_or_else(Vec::new, |value| parse_lrc_lines(value))
    };

    let plain_for_document = if is_instrumental {
        None
    } else {
        plain.or_else(|| synced.as_ref().map(|value| strip_timestamp(value)))
    };

    let document = LyricsfileDocument {
        version: LYRICSFILE_VERSION.to_string(),
        metadata: LyricsfileMetadata {
            title: metadata.title.clone(),
            artist: metadata.artist_name.clone(),
            album: normalize_non_empty(Some(metadata.album_name.as_str())),
            duration_ms: duration_to_ms(metadata.duration),
            offset_ms: None,
            language: None,
            instrumental: is_instrumental,
        },
        lines: synced_lines,
        plain: plain_for_document,
    };

    serde_yaml::to_string(&document).ok()
}

pub fn parse_lyricsfile(lyricsfile: &str) -> Result<ParsedLyricsfile> {
    let document: ParsedLyricsfileDocument = serde_yaml::from_str(lyricsfile)?;

    let is_instrumental = document.metadata.instrumental;
    let synced_lyrics = if is_instrumental {
        Some(INSTRUMENTAL_LRC.to_string())
    } else {
        lines_to_lrc(&document.lines)
    };

    let plain_lyrics = normalize_non_empty(document.plain.as_deref()).or_else(|| {
        synced_lyrics
            .as_ref()
            .map(|value| strip_timestamp(value))
            .and_then(|value| normalize_non_empty(Some(value.as_str())))
    });

    Ok(ParsedLyricsfile {
        plain_lyrics,
        synced_lyrics,
        is_instrumental,
    })
}

pub fn lyrics_presence_from_lyricsfile(lyricsfile: &str) -> Result<LyricsPresence> {
    let document: ParsedLyricsfileDocument = serde_yaml::from_str(lyricsfile)?;
    let is_instrumental = document.metadata.instrumental;

    if is_instrumental {
        return Ok(LyricsPresence {
            is_instrumental: true,
            ..LyricsPresence::default()
        });
    }

    let has_synced_lyrics = !document.lines.is_empty();
    let has_word_synced_lyrics = document.lines.iter().any(|line| !line.words.is_empty());
    let has_plain_from_lines = document.lines.iter().any(|line| {
        normalize_non_empty(Some(line.text.as_str())).is_some()
            || line
                .words
                .iter()
                .any(|word| normalize_non_empty(Some(word.text.as_str())).is_some())
    });
    let has_plain_lyrics = normalize_non_empty(document.plain.as_deref()).is_some()
        || (has_synced_lyrics && has_plain_from_lines);

    Ok(LyricsPresence {
        has_plain_lyrics,
        has_synced_lyrics,
        has_word_synced_lyrics,
        is_instrumental: false,
    })
}

pub fn is_instrumental_lyrics(lyrics: &str) -> bool {
    let lowered = lyrics.to_lowercase();
    lowered.contains("[au:") && lowered.contains("instrumental")
}

fn parse_lrc_lines(synced_lyrics: &str) -> Vec<LyricsfileLine> {
    let lyrics = match Lyrics::from_str(synced_lyrics) {
        Ok(lyrics) => lyrics,
        Err(_) => return Vec::new(),
    };

    println!("lyrics: {:?}", lyrics);

    let timed_lines = lyrics.get_timed_lines();

    timed_lines
        .iter()
        .enumerate()
        .map(|(index, (timestamp, text))| {
            let start_ms = timestamp.get_timestamp() as i64;
            let end_ms = timed_lines
                .get(index + 1)
                .map(|(next_timestamp, _)| next_timestamp.get_timestamp() as i64);

            LyricsfileLine {
                text: text.to_string(),
                start_ms,
                end_ms,
                words: Vec::new(),
            }
        })
        .collect()
}

fn lines_to_lrc(lines: &[LyricsfileLine]) -> Option<String> {
    let mut output = String::new();

    for line in lines {
        let text = if !line.words.is_empty() {
            line.words.iter().map(|word| word.text.as_str()).collect()
        } else {
            line.text.clone()
        };

        output.push_str(&format!(
            "{} {}\n",
            format_lrc_timestamp(line.start_ms),
            text
        ));
    }

    normalize_non_empty(Some(output.as_str()))
}

fn format_lrc_timestamp(timestamp_ms: i64) -> String {
    let safe_ms = timestamp_ms.max(0);
    let minutes = safe_ms / 60000;
    let seconds = (safe_ms % 60000) / 1000;
    let centiseconds = (safe_ms % 1000) / 10;

    format!("[{:02}:{:02}.{:02}]", minutes, seconds, centiseconds)
}

fn duration_to_ms(duration: f64) -> Option<i64> {
    if duration > 0.0 {
        Some((duration * 1000.0).round() as i64)
    } else {
        None
    }
}

fn normalize_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::to_string)
        .filter(|content| !content.trim().is_empty())
}
