use crate::lyricsfile::ParsedLyricsfile;
use crate::persistent_entities::PersistentTrack;
use anyhow::{Context, Result};
use lofty::config::WriteOptions;
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::id3::v2::{
    BinaryFrame, Frame, FrameId, Id3v2Tag, SyncTextContentType, SynchronizedTextFrame,
    TimestampFormat, UnsynchronizedTextFrame,
};
use lofty::mpeg::MpegFile;
use lofty::TextEncoding;
use lrc::Lyrics;
use serde::Serialize;
use std::fs::{remove_file, write};
use std::io::Seek;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during export operations
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Failed to build export path: {0}")]
    PathBuildError(String),

    #[error("Failed to write file: {0}")]
    WriteError(String),

    #[error("Failed to embed lyrics: {0}")]
    EmbedError(String),

    #[error("Invalid lyrics data: {0}")]
    InvalidData(String),
}

/// Export format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Plain text format (.txt)
    Txt,
    /// Standard LRC format (.lrc)
    Lrc,
    /// Embedded in audio file metadata
    Embedded,
}

/// Status of an export operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "message")]
pub enum ExportStatus {
    /// Export was successful
    Success,
    /// Export was skipped (e.g., no lyrics available for this format)
    Skipped(String),
    /// Export failed with an error
    Error(String),
}

/// Result of an export operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub format: ExportFormat,
    pub path: Option<PathBuf>,
    pub status: ExportStatus,
}

/// Build the file path for a lyrics sidecar file
pub fn build_sidecar_path(track_path: &str, extension: &str) -> Result<PathBuf, ExportError> {
    let path = Path::new(track_path);
    let parent_path = path
        .parent()
        .ok_or_else(|| ExportError::PathBuildError("Track has no parent directory".to_string()))?;
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ExportError::PathBuildError("Invalid track filename".to_string()))?;

    Ok(parent_path.join(format!("{}.{}", file_stem, extension)))
}

/// Generate plain text lyrics content from parsed lyricsfile
pub fn generate_txt_content(parsed: &ParsedLyricsfile) -> Option<String> {
    if parsed.is_instrumental {
        return None;
    }

    parsed.plain_lyrics.clone().filter(|s| !s.trim().is_empty())
}

/// Generate standard LRC format content from parsed lyricsfile
pub fn generate_lrc_content(parsed: &ParsedLyricsfile) -> Option<String> {
    if parsed.is_instrumental {
        return Some(crate::lyricsfile::INSTRUMENTAL_LRC.to_string());
    }

    parsed
        .synced_lyrics
        .clone()
        .filter(|s| !s.trim().is_empty())
}

/// Export lyrics for a single track in the specified format
pub fn export_track_format(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
    format: ExportFormat,
) -> Result<ExportResult, ExportError> {
    match format {
        ExportFormat::Txt => export_txt(track, parsed),
        ExportFormat::Lrc => export_lrc(track, parsed),
        ExportFormat::Embedded => export_embedded(track, parsed),
    }
}

/// Export plain text lyrics to .txt file
fn export_txt(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
) -> Result<ExportResult, ExportError> {
    let content = match generate_txt_content(parsed) {
        Some(content) => content,
        None => {
            // Not an error - just no plain lyrics available
            return Ok(ExportResult {
                format: ExportFormat::Txt,
                path: None,
                status: ExportStatus::Skipped("no plain lyrics available".to_string()),
            });
        }
    };

    let txt_path = build_sidecar_path(&track.file_path, "txt")?;

    // Remove conflicting .lrc file if it exists
    let lrc_path = build_sidecar_path(&track.file_path, "lrc").ok();
    if let Some(ref lrc_path) = lrc_path {
        let _ = remove_file(lrc_path);
    }

    write(&txt_path, content).map_err(|e| ExportError::WriteError(e.to_string()))?;

    Ok(ExportResult {
        format: ExportFormat::Txt,
        path: Some(txt_path),
        status: ExportStatus::Success,
    })
}

/// Export synced lyrics to .lrc file
fn export_lrc(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
) -> Result<ExportResult, ExportError> {
    let content = match generate_lrc_content(parsed) {
        Some(content) => content,
        None => {
            // Not an error - just no synced lyrics available
            return Ok(ExportResult {
                format: ExportFormat::Lrc,
                path: None,
                status: ExportStatus::Skipped("no synced lyrics available".to_string()),
            });
        }
    };

    let lrc_path = build_sidecar_path(&track.file_path, "lrc")?;

    // Remove conflicting .txt file if it exists
    let txt_path = build_sidecar_path(&track.file_path, "txt").ok();
    if let Some(ref txt_path) = txt_path {
        let _ = remove_file(txt_path);
    }

    write(&lrc_path, content).map_err(|e| ExportError::WriteError(e.to_string()))?;

    Ok(ExportResult {
        format: ExportFormat::Lrc,
        path: Some(lrc_path),
        status: ExportStatus::Success,
    })
}

/// Export lyrics by embedding into audio file metadata
fn export_embedded(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
) -> Result<ExportResult, ExportError> {
    let plain_lyrics = parsed.plain_lyrics.clone().unwrap_or_default();
    let synced_lyrics = if parsed.is_instrumental {
        crate::lyricsfile::INSTRUMENTAL_LRC.to_string()
    } else {
        parsed.synced_lyrics.clone().unwrap_or_default()
    };

    embed_lyrics(&track.file_path, &plain_lyrics, &synced_lyrics)
        .map_err(|e| ExportError::EmbedError(e.to_string()))?;

    Ok(ExportResult {
        format: ExportFormat::Embedded,
        path: Some(PathBuf::from(&track.file_path)),
        status: ExportStatus::Success,
    })
}

/// Export lyrics for a track in multiple formats
pub fn export_track(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
    formats: &[ExportFormat],
) -> Vec<ExportResult> {
    let mut results = Vec::with_capacity(formats.len());

    for format in formats {
        match export_track_format(track, parsed, *format) {
            Ok(result) => results.push(result),
            Err(e) => results.push(ExportResult {
                format: *format,
                path: None,
                status: ExportStatus::Error(e.to_string()),
            }),
        }
    }

    results
}

/// Embed lyrics into audio file metadata (MP3/FLAC)
pub fn embed_lyrics(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
    let path_lower = track_path.to_lowercase();

    if path_lower.ends_with(".mp3") {
        embed_lyrics_mp3(track_path, plain_lyrics, synced_lyrics)
    } else if path_lower.ends_with(".flac") {
        embed_lyrics_flac(track_path, plain_lyrics, synced_lyrics)
    } else {
        // Not an error - just not supported for this format
        Ok(())
    }
}

/// Embed lyrics into FLAC file using Vorbis comments
fn embed_lyrics_flac(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
    use lofty::config::ParseOptions;
    use std::fs::OpenOptions;

    let mut file_content = OpenOptions::new()
        .read(true)
        .write(true)
        .open(track_path)
        .context("Failed to open FLAC file")?;

    let mut flac_file = FlacFile::read_from(&mut file_content, ParseOptions::new())
        .context("Failed to parse FLAC file")?;

    if let Some(vorbis_comments) = flac_file.vorbis_comments_mut() {
        // Handle unsynced lyrics (USLT equivalent in FLAC)
        if !plain_lyrics.is_empty() {
            vorbis_comments.insert("UNSYNCEDLYRICS".to_string(), plain_lyrics.to_string());
        } else {
            let _ = vorbis_comments.remove("UNSYNCEDLYRICS");
        }

        // Handle synced lyrics (SYLT equivalent in FLAC)
        if !synced_lyrics.is_empty() {
            vorbis_comments.insert("LYRICS".to_string(), synced_lyrics.to_string());
        } else {
            let _ = vorbis_comments.remove("LYRICS");
        }

        file_content
            .seek(std::io::SeekFrom::Start(0))
            .context("Failed to seek in FLAC file")?;
        flac_file
            .save_to(&mut file_content, WriteOptions::default())
            .context("Failed to save FLAC file")?;
    }

    Ok(())
}

/// Embed lyrics into MP3 file using ID3v2 tags
fn embed_lyrics_mp3(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
    use lofty::config::ParseOptions;
    use std::fs::OpenOptions;

    let mut file_content = OpenOptions::new()
        .read(true)
        .write(true)
        .open(track_path)
        .context("Failed to open MP3 file")?;

    let mut mp3_file = MpegFile::read_from(&mut file_content, ParseOptions::new())
        .context("Failed to parse MP3 file")?;

    if let Some(id3v2) = mp3_file.id3v2_mut() {
        // Insert unsynchronized lyrics (USLT)
        insert_uslt_frame(id3v2, plain_lyrics).context("Failed to insert USLT frame")?;

        // Insert synchronized lyrics (SYLT)
        insert_sylt_frame(id3v2, synced_lyrics).context("Failed to insert SYLT frame")?;

        file_content
            .seek(std::io::SeekFrom::Start(0))
            .context("Failed to seek in MP3 file")?;
        mp3_file
            .save_to(&mut file_content, WriteOptions::default())
            .context("Failed to save MP3 file")?;
    }

    Ok(())
}

/// Insert USLT (unsynchronized lyrics) frame into ID3v2 tag
fn insert_uslt_frame(id3v2: &mut Id3v2Tag, plain_lyrics: &str) -> Result<()> {
    if !plain_lyrics.is_empty() {
        let uslt_frame = UnsynchronizedTextFrame::new(
            TextEncoding::UTF8,
            [b'X', b'X', b'X'],
            "".to_string(),
            plain_lyrics.to_string(),
        );
        id3v2.insert(Frame::UnsynchronizedText(uslt_frame));
    } else {
        let _ = id3v2.remove(&FrameId::new("USLT")?);
    }

    Ok(())
}

/// Insert SYLT (synchronized lyrics) frame into ID3v2 tag
fn insert_sylt_frame(id3v2: &mut Id3v2Tag, synced_lyrics: &str) -> Result<()> {
    if !synced_lyrics.is_empty() {
        let synced_lyrics_vec = synced_lyrics_to_sylt_vec(synced_lyrics)?;

        let sylt_frame = SynchronizedTextFrame::new(
            TextEncoding::UTF8,
            [b'X', b'X', b'X'],
            TimestampFormat::MS,
            SyncTextContentType::Lyrics,
            None,
            synced_lyrics_vec,
        );

        let sylt_frame_byte = sylt_frame.as_bytes(WriteOptions::default())?;
        let sylt_frame_id = FrameId::new("SYLT")?;
        id3v2.insert(Frame::Binary(BinaryFrame::new(
            sylt_frame_id,
            sylt_frame_byte,
        )));
    } else {
        let _ = id3v2.remove(&FrameId::new("SYLT")?);
    }

    Ok(())
}

/// Convert synced LRC lyrics to SYLT vector format
fn synced_lyrics_to_sylt_vec(synced_lyrics: &str) -> Result<Vec<(u32, String)>> {
    let lyrics = Lyrics::from_str(synced_lyrics)
        .map_err(|e| anyhow::anyhow!("Failed to parse LRC: {}", e))?;
    let lyrics_vec = lyrics.get_timed_lines();

    let converted_lyrics: Vec<(u32, String)> = lyrics_vec
        .iter()
        .map(|(time_tag, text)| (time_tag.get_timestamp() as u32, text.to_string()))
        .collect();

    Ok(converted_lyrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sidecar_path() {
        let track_path = "/music/artist/album/song.mp3";
        let txt_path = build_sidecar_path(track_path, "txt").unwrap();
        assert_eq!(txt_path.to_str().unwrap(), "/music/artist/album/song.txt");

        let lrc_path = build_sidecar_path(track_path, "lrc").unwrap();
        assert_eq!(lrc_path.to_str().unwrap(), "/music/artist/album/song.lrc");
    }

    #[test]
    fn test_generate_txt_content() {
        let parsed = ParsedLyricsfile {
            plain_lyrics: Some("Line 1\nLine 2".to_string()),
            synced_lyrics: None,
            is_instrumental: false,
        };

        let content = generate_txt_content(&parsed);
        assert_eq!(content, Some("Line 1\nLine 2".to_string()));

        // Instrumental should return None
        let instrumental = ParsedLyricsfile {
            plain_lyrics: None,
            synced_lyrics: None,
            is_instrumental: true,
        };
        assert_eq!(generate_txt_content(&instrumental), None);
    }

    #[test]
    fn test_generate_lrc_content() {
        let parsed = ParsedLyricsfile {
            plain_lyrics: None,
            synced_lyrics: Some("[00:12.00]Line 1".to_string()),
            is_instrumental: false,
        };

        let content = generate_lrc_content(&parsed);
        assert_eq!(content, Some("[00:12.00]Line 1".to_string()));

        // Instrumental should return special marker
        let instrumental = ParsedLyricsfile {
            plain_lyrics: None,
            synced_lyrics: None,
            is_instrumental: true,
        };
        assert_eq!(
            generate_lrc_content(&instrumental),
            Some("[au: instrumental]".to_string())
        );
    }
}
