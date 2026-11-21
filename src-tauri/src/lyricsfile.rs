use crate::parser::lrc::{format_timestamp, parse_lrc};
use crate::persistent_entities::PersistentTrack;
use crate::utils::strip_timestamp;
use anyhow::Result;
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

fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct ParsedLyricsfileDocument {
    metadata: ParsedLyricsfileMetadata,
    #[serde(default, deserialize_with = "null_as_default")]
    lines: Vec<LyricsfileLine>,
    plain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParsedLyricsfileMetadata {
    #[serde(default)]
    instrumental: bool,
    offset_ms: Option<i64>,
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
    let parsed = parse_lrc(synced_lyrics);

    parsed
        .timed_lines
        .iter()
        .enumerate()
        .map(|(index, timed_line)| {
            let start_ms = timed_line.timestamp_ms;
            let end_ms = parsed
                .timed_lines
                .get(index + 1)
                .map(|next_line| next_line.timestamp_ms);

            LyricsfileLine {
                text: timed_line.text.clone(),
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

        output.push_str(&format!("{} {}\n", format_timestamp(line.start_ms), text));
    }

    normalize_non_empty(Some(output.as_str()))
}

/// Convert a Lyricsfile into Enhanced LRC, retaining word start times
/// and the final word's end time when available.
///
/// Return None for instrumental, plain-only, or line-only lyrics.
/// Mixed word-synced and line-synced lyrics retain their original lines.
/// Independent intermediate word ends and exact line ends are not representable.
pub fn lyricsfile_to_elrc(lyricsfile: &str) -> Result<Option<String>> {
    let document: ParsedLyricsfileDocument = serde_yaml::from_str(lyricsfile)?;

    if document.metadata.instrumental
        || !document
            .lines
            .iter()
            .any(|line| line.words.iter().any(|word| !word.text.trim().is_empty()))
    {
        return Ok(None);
    }

    // Lyricsfile 1.0 has not defined the sign or application of offset_ms.
    anyhow::ensure!(
        document.metadata.offset_ms.unwrap_or(0) == 0,
        "Cannot export a nonzero Lyricsfile offset: its semantics are not defined"
    );

    let mut output = String::new();

    for line in &document.lines {
        if let Some(end) = line.end_ms {
            anyhow::ensure!(end >= line.start_ms, "Invalid line end timestamp");
        }
        output.push_str(&format_elrc_timestamp(line.start_ms, '[', ']')?);

        if line.words.is_empty() {
            validate_elrc_text(&line.text)?;
            output.push_str(&line.text);
        } else {
            let text: String = line.words.iter().map(|word| word.text.as_str()).collect();
            anyhow::ensure!(
                text.trim() == line.text.trim(),
                "Word segments do not reconstruct the lyric line"
            );
            validate_elrc_text(&text)?;
            let mut previous_start = line.start_ms;

            for word in &line.words {
                anyhow::ensure!(
                    !word.text.trim().is_empty(),
                    "Empty or whitespace-only word segment"
                );
                anyhow::ensure!(
                    word.start_ms >= previous_start,
                    "Invalid or unordered word timestamp"
                );
                if let Some(end) = word.end_ms {
                    anyhow::ensure!(end >= word.start_ms, "Invalid word end timestamp");
                }
                output.push_str(&format_elrc_timestamp(word.start_ms, '<', '>')?);
                output.push_str(&word.text);
                previous_start = word.start_ms;
            }

            if let Some(end_ms) = line.words.last().and_then(|word| word.end_ms) {
                output.push_str(&format_elrc_timestamp(end_ms, '<', '>')?);
            }
        }

        output.push('\n');
    }

    Ok(normalize_non_empty(Some(output.as_str())))
}

/// ELRC has no interoperable escape mechanism for literal timing tags or
/// embedded line breaks. Reject these rather than producing ambiguous output.
fn validate_elrc_text(text: &str) -> Result<()> {
    static TIMING_TAG: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"(?:\[(?:[0-9]{1,2}:)?[0-9]{1,2}:[0-9]{1,2}(?:\.[0-9]{1,3})?\]|<(?:[0-9]{1,2}:)?[0-9]{1,2}:[0-9]{1,2}(?:\.[0-9]{1,3})?>)",
        ).expect("valid ELRC timing regex")
    });
    anyhow::ensure!(
        !text.contains('\n') && !text.contains('\r') && !TIMING_TAG.is_match(text),
        "Lyric text contains a line break or literal timing tag"
    );
    Ok(())
}

/// Preserve millisecond precision; switch to hour-based tags when minutes
/// exceed the two-digit range accepted by enhanced LRC parsers.
fn format_elrc_timestamp(timestamp_ms: i64, open: char, close: char) -> Result<String> {
    anyhow::ensure!(
        (0..360_000_000).contains(&timestamp_ms),
        "ELRC timestamp outside the supported range (0 to 99:59:59.999)"
    );
    let minutes = timestamp_ms / 60_000;
    let seconds = (timestamp_ms % 60_000) / 1_000;
    let milliseconds = timestamp_ms % 1_000;

    if minutes < 100 {
        Ok(format!(
            "{open}{minutes:02}:{seconds:02}.{milliseconds:03}{close}"
        ))
    } else {
        let hours = minutes / 60;
        let minutes = minutes % 60;
        Ok(format!(
            "{open}{hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}{close}"
        ))
    }
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

#[cfg(test)]
mod elrc_tests {
    use super::*;

    #[test]
    fn supports_long_timestamps_and_rejects_out_of_range() {
        assert_eq!(
            format_elrc_timestamp(5_999_999, '[', ']').unwrap(),
            "[99:59.999]"
        );
        assert_eq!(
            format_elrc_timestamp(6_000_123, '<', '>').unwrap(),
            "<01:40:00.123>"
        );
        assert_eq!(
            format_elrc_timestamp(359_999_999, '[', ']').unwrap(),
            "[99:59:59.999]"
        );
        assert!(format_elrc_timestamp(360_000_000, '[', ']').is_err());
        assert!(format_elrc_timestamp(-1, '<', '>').is_err());
    }

    #[test]
    fn rejects_undefined_nonzero_offset() {
        let input = r#"
metadata:
  offset_ms: -125
lines:
  - text: Test
    start_ms: 1000
    words:
      - text: Test
        start_ms: 1000
"#;
        assert!(lyricsfile_to_elrc(input).is_err());
        assert!(lyricsfile_to_elrc(&input.replace("-125", "0"))
            .unwrap()
            .is_some());
    }

    #[test]
    fn rejects_word_text_mismatch() {
        let input = r#"
metadata: {}
lines:
  - text: Hello world
    start_ms: 1000
    words:
      - text: Hello
        start_ms: 1000
"#;
        assert!(lyricsfile_to_elrc(input).is_err());
    }

    #[test]
    fn rejects_empty_word_segments_in_timed_lines() {
        let input = r#"
metadata: {}
lines:
  - text: Hello
    start_ms: 1000
    words:
      - text: Hello
        start_ms: 1000
      - text: ""
        start_ms: 1500
"#;
        assert!(lyricsfile_to_elrc(input).is_err());
    }

    #[test]
    fn rejects_invalid_intermediate_word_end() {
        let input = r#"
metadata: {}
lines:
  - text: Hello world
    start_ms: 1000
    words:
      - text: "Hello "
        start_ms: 1000
        end_ms: 999
      - text: world
        start_ms: 1500
"#;
        assert!(lyricsfile_to_elrc(input).is_err());
    }

    #[test]
    fn rejects_ambiguous_text() {
        for text in [
            "one\ntwo",
            "one\rtwo",
            "literal <00:01.23>",
            "literal [00:01.23]",
        ] {
            assert!(validate_elrc_text(text).is_err());
        }
        assert!(validate_elrc_text("Hello [chorus] <3").is_ok());
    }

    #[test]
    fn preserves_word_timestamps_and_spacing() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: Hello world
    start_ms: 12345
    end_ms: 13567
    words:
      - text: "Hello "
        start_ms: 12345
        end_ms: 12901
      - text: world
        start_ms: 12901
        end_ms: 13567
"#;

        assert_eq!(
            lyricsfile_to_elrc(input).unwrap().unwrap(),
            "[00:12.345]<00:12.345>Hello <00:12.901>world<00:13.567>\n"
        );

        // Standard LRC remains unchanged.
        assert_eq!(
            parse_lyricsfile(input).unwrap().synced_lyrics.unwrap(),
            "[00:12.34] Hello world\n"
        );
    }

    #[test]
    fn preserves_mixed_sync_and_unicode() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: "আমি ভালো"
    start_ms: 1000
    words:
      - text: "আমি "
        start_ms: 1000
      - text: "ভালো"
        start_ms: 1505
        end_ms: 2000
  - text: "Line-only lyrics"
    start_ms: 3000
  - text: ""
    start_ms: 4000
"#;

        assert_eq!(
            lyricsfile_to_elrc(input).unwrap().unwrap(),
            concat!(
                "[00:01.000]<00:01.000>আমি ",
                "<00:01.505>ভালো<00:02.000>\n",
                "[00:03.000]Line-only lyrics\n",
                "[00:04.000]\n"
            )
        );
    }

    #[test]
    fn does_not_invent_final_word_end() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: Test
    start_ms: 1000
    words:
      - text: Test
        start_ms: 1100
"#;

        assert_eq!(
            lyricsfile_to_elrc(input).unwrap().unwrap(),
            "[00:01.000]<00:01.100>Test\n"
        );
    }

    #[test]
    fn skips_line_only_lyrics() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: Test line
    start_ms: 1000
"#;

        assert_eq!(lyricsfile_to_elrc(input).unwrap(), None);
    }

    #[test]
    fn skips_plain_and_instrumental_lyrics() {
        let plain = r#"
metadata:
  instrumental: false
plain: Plain lyrics
"#;

        let instrumental = r#"
metadata:
  instrumental: true
lines: []
"#;

        assert_eq!(lyricsfile_to_elrc(plain).unwrap(), None);
        assert_eq!(lyricsfile_to_elrc(instrumental).unwrap(), None);
    }

    #[test]
    fn rejects_invalid_word_order() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: Hello world
    start_ms: 1000
    words:
      - text: "Hello "
        start_ms: 1500
      - text: world
        start_ms: 1200
"#;

        assert!(lyricsfile_to_elrc(input).is_err());
    }

    #[test]
    fn rejects_invalid_final_word_end() {
        let input = r#"
metadata:
  instrumental: false
lines:
  - text: Test
    start_ms: 1000
    words:
      - text: Test
        start_ms: 1500
        end_ms: 1200
"#;

        assert!(lyricsfile_to_elrc(input).is_err());
    }
}
