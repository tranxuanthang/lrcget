//! LRC (LyRiCs) file format parser
//!
//! A lightweight parser for LRC format synced lyrics.
//! Supports timestamp tags with 1-3 digit precision for milliseconds.

use regex::Regex;
use std::sync::LazyLock;

/// A single timed line with millisecond timestamp and text
#[derive(Debug, Clone, PartialEq)]
pub struct TimedLine {
    /// Timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Lyric text
    pub text: String,
}

/// Parsed LRC content
#[derive(Debug, Clone, Default)]
pub struct ParsedLrc {
    /// Timestamped lyric lines, sorted by timestamp
    pub timed_lines: Vec<TimedLine>,
    /// ID tags (metadata) like [ti:title], [ar:artist]
    pub id_tags: Vec<(String, String)>,
}

/// Regex for parsing timestamp tags: [mm:ss.xxx] or [mm:ss.xx] or [mm:ss.x]
/// Supports 1-3 digits after the decimal point (centiseconds to milliseconds)
static TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(\d{1,2}):(\d{1,2})\.(\d{1,3})\]").unwrap()
});

/// Regex for parsing ID tags: [key:value]
static ID_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^:\]]+):([^\]]*)\]").unwrap());

/// Parse LRC format string into structured data
///
/// # Arguments
/// * `input` - The LRC content to parse
///
/// # Returns
/// A `ParsedLrc` struct containing timed lines and ID tags
///
/// # Examples
/// ```
/// use lrcget::parser::lrc::parse_lrc;
///
/// let lrc = "[00:01.500] Line one\n[00:02.75] Line two";
/// let parsed = parse_lrc(lrc);
///
/// assert_eq!(parsed.timed_lines.len(), 2);
/// assert_eq!(parsed.timed_lines[0].timestamp_ms, 1500);
/// ```
pub fn parse_lrc(input: &str) -> ParsedLrc {
    let mut result = ParsedLrc::default();
    let mut all_timed_lines: Vec<TimedLine> = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Extract all timestamp tags from the beginning of the line
        let mut timestamps: Vec<i64> = Vec::new();
        let mut remaining = line;

        // Keep matching timestamp tags at the start
        while let Some(caps) = TIMESTAMP_RE.captures(remaining) {
            let full_match = caps.get(0).unwrap();
            let start = full_match.start();
            let end = full_match.end();

            // Ensure the match is at the very beginning
            if start != 0 {
                break;
            }

            // Parse the timestamp components
            let minutes: i64 = caps[1].parse().unwrap_or(0);
            let seconds: i64 = caps[2].parse().unwrap_or(0);
            let fraction_str = &caps[3];

            // Convert fraction to milliseconds based on digit count
            let millis = match fraction_str.len() {
                1 => fraction_str.parse::<i64>().unwrap_or(0) * 100, // deciseconds (0.1s = 100ms)
                2 => fraction_str.parse::<i64>().unwrap_or(0) * 10,  // centiseconds (0.01s = 10ms)
                3 => fraction_str.parse::<i64>().unwrap_or(0),       // milliseconds (0.001s = 1ms)
                _ => continue, // Should not happen due to regex
            };

            let total_ms = minutes * 60_000 + seconds * 1_000 + millis;
            timestamps.push(total_ms);

            // Move past this tag
            remaining = &remaining[end..];
        }

        // Check for ID tags (like [ti:Title], [au:instrumental])
        // These are tags that don't match the timestamp pattern
        for caps in ID_TAG_RE.captures_iter(line) {
            let key = caps[1].trim().to_lowercase();
            let value = caps[2].trim().to_string();

            // Skip if this was already matched as a timestamp
            let full_tag = format!("[{}:{}]", &caps[1], &caps[2]);
            if TIMESTAMP_RE.is_match(&full_tag) {
                continue;
            }

            result.id_tags.push((key, value));
        }

        // If we found timestamps, add the line for each timestamp
        if !timestamps.is_empty() {
            let text = remaining.trim().to_string();
            for timestamp in timestamps {
                all_timed_lines.push(TimedLine {
                    timestamp_ms: timestamp,
                    text: text.clone(),
                });
            }
        }
    }

    // Sort by timestamp
    all_timed_lines.sort_by_key(|line| line.timestamp_ms);
    result.timed_lines = all_timed_lines;

    result
}

/// Check if the LRC content indicates an instrumental track
///
/// Returns true if there's an [au:instrumental] tag (case-insensitive)
pub fn is_instrumental_lrc(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    input_lower.contains("[au:instrumental]") || input_lower.contains("[au: instrumental]")
}

/// Format milliseconds as LRC timestamp [mm:ss.xx]
/// Always outputs 2-digit centisecond precision for compatibility
pub fn format_timestamp(timestamp_ms: i64) -> String {
    let safe_ms = timestamp_ms.max(0);
    let minutes = safe_ms / 60_000;
    let seconds = (safe_ms % 60_000) / 1_000;
    let centiseconds = (safe_ms % 1_000) / 10;

    format!("[{:02}:{:02}.{:02}]", minutes, seconds, centiseconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_lrc() {
        let lrc = "[00:01.50] Line one\n[00:02.75] Line two";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 2);
        // [00:01.50] = 0 min + 1 sec + 50 centiseconds = 1500ms
        assert_eq!(parsed.timed_lines[0].timestamp_ms, 1_500);
        assert_eq!(parsed.timed_lines[0].text, "Line one");
        // [00:02.75] = 0 min + 2 sec + 75 centiseconds = 2750ms
        assert_eq!(parsed.timed_lines[1].timestamp_ms, 2_750);
        assert_eq!(parsed.timed_lines[1].text, "Line two");
    }

    #[test]
    fn test_parse_3digit_milliseconds() {
        // This is the key test - 3-digit milliseconds should work
        let lrc = "[01:35.492] I wonder where they are";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 1);
        // 1 min 35 sec 492 ms = 95492 ms
        assert_eq!(parsed.timed_lines[0].timestamp_ms, 95_492);
        assert_eq!(parsed.timed_lines[0].text, "I wonder where they are");
    }

    #[test]
    fn test_parse_1digit_deciseconds() {
        let lrc = "[01:35.4] One digit";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 1);
        // 1 min 35 sec 4 deciseconds = 95400 ms
        assert_eq!(parsed.timed_lines[0].timestamp_ms, 95_400);
    }

    #[test]
    fn test_parse_2digit_centiseconds() {
        let lrc = "[01:35.49] Two digits";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 1);
        // 1 min 35 sec 49 centiseconds = 95490 ms
        assert_eq!(parsed.timed_lines[0].timestamp_ms, 95_490);
    }

    #[test]
    fn test_multiple_timestamps_one_line() {
        // Multiple timestamps for the same lyric line
        let lrc = "[00:01.500][00:03.250] Repeated line";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 2);
        assert_eq!(parsed.timed_lines[0].timestamp_ms, 1_500);
        assert_eq!(parsed.timed_lines[1].timestamp_ms, 3_250);
        assert_eq!(parsed.timed_lines[0].text, "Repeated line");
        assert_eq!(parsed.timed_lines[1].text, "Repeated line");
    }

    #[test]
    fn test_id_tags() {
        let lrc = "[ti:Song Title]\n[ar:Artist Name]\n[au: instrumental]\n[00:01.00] First line";
        let parsed = parse_lrc(lrc);

        assert!(parsed.id_tags.iter().any(|(k, v)| k == "ti" && v == "Song Title"));
        assert!(parsed.id_tags.iter().any(|(k, v)| k == "ar" && v == "Artist Name"));
        assert!(parsed
            .id_tags
            .iter()
            .any(|(k, v)| k == "au" && v == "instrumental"));
        assert_eq!(parsed.timed_lines.len(), 1);
    }

    #[test]
    fn test_is_instrumental() {
        assert!(is_instrumental_lrc("[au: instrumental]"));
        assert!(is_instrumental_lrc("[au:instrumental]"));
        assert!(is_instrumental_lrc("[AU:INSTRUMENTAL]"));
        assert!(is_instrumental_lrc("Some lyrics\n[au: instrumental]\nMore lyrics"));
        assert!(!is_instrumental_lrc("[00:01.00] Regular lyrics"));
        assert!(!is_instrumental_lrc("[au: lyrics]"));
    }

    #[test]
    fn test_empty_lines_ignored() {
        let lrc = "[00:01.00] First\n\n[00:02.00] Second\n   \n[00:03.00] Third";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 3);
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "[00:00.00]");
        assert_eq!(format_timestamp(1_500), "[00:01.50]");
        assert_eq!(format_timestamp(95_492), "[01:35.49]"); // Note: rounds to centiseconds
        assert_eq!(format_timestamp(60_000), "[01:00.00]");
    }

    #[test]
    fn test_sorting_by_timestamp() {
        // Lines in wrong order should be sorted
        let lrc = "[00:03.00] Third\n[00:01.00] First\n[00:02.00] Second";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines[0].text, "First");
        assert_eq!(parsed.timed_lines[1].text, "Second");
        assert_eq!(parsed.timed_lines[2].text, "Third");
    }

    #[test]
    fn test_no_timestamps_returns_empty() {
        let lrc = "Just plain text\nNo timestamps here";
        let parsed = parse_lrc(lrc);

        assert_eq!(parsed.timed_lines.len(), 0);
        assert_eq!(parsed.id_tags.len(), 0);
    }
}
