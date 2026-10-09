use crate::lyricsfile::ParsedLyricsfile;
use crate::parser::lrc::parse_lrc;
use crate::persistent_entities::PersistentTrack;
use anyhow::{Context, Result};
use lofty::config::WriteOptions;
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::id3::v2::{
    BinaryFrame, CommentFrame, Frame, FrameId, Id3v2Tag, SyncTextContentType,
    SynchronizedTextFrame, TimestampFormat, UnsynchronizedTextFrame,
};
use lofty::mpeg::MpegFile;
use lofty::TextEncoding;
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
    let path_lower = track.file_path.to_lowercase();
    if !path_lower.ends_with(".mp3") && !path_lower.ends_with(".flac") {
        return Ok(ExportResult {
            format: ExportFormat::Embedded,
            path: None,
            status: ExportStatus::Skipped("unsupported audio format (MP3/FLAC only)".to_owned()),
        });
    }
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
    export_track_with_embed_gate(track, parsed, formats, || true)
}

/// Consult the current experimental setting at each embedded operation, not at batch creation.
pub fn export_track_with_embed_gate(
    track: &PersistentTrack,
    parsed: &ParsedLyricsfile,
    formats: &[ExportFormat],
    mut embed_allowed: impl FnMut() -> bool,
) -> Vec<ExportResult> {
    let mut results = Vec::with_capacity(formats.len());

    for format in formats {
        if *format == ExportFormat::Embedded && !embed_allowed() {
            results.push(ExportResult {
                format: *format,
                path: None,
                status: ExportStatus::Skipped(
                    "experimental embedding disabled or unavailable".to_owned(),
                ),
            });
            continue;
        }
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
    use lofty::file::TaggedFileExt;
    use lofty::id3::v2::Id3v2Tag;
    use lofty::probe::Probe;

    let file_probe = Probe::open(track_path).context("Failed to open MP3 file")?;
    let mut file = file_probe
        .guess_file_type()
        .context("Failed to guess file type")?
        .read()
        .context("Failed to read MP3 file")?;
    let mut primary_tag = file
        .remove(file.primary_tag_type())
        .context("Failed to find ID3v2 tag")?;
    let mut id3v2: Id3v2Tag = primary_tag.into();

    // Fix malformed COMMENT frames: re-insert with valid language code
    let removed_comments: Vec<_> = id3v2.remove(&FrameId::new("COMM")?).collect();
    for frame in removed_comments {
        if let Frame::Comment(comment) = frame {
            id3v2.insert(Frame::Comment(CommentFrame::new(
                comment.encoding,
                [b'X', b'X', b'X'],
                comment.description,
                comment.content,
            )));
        }
    }

    // Insert unsynchronized lyrics (USLT)
    insert_uslt_frame(&mut id3v2, plain_lyrics).context("Failed to insert USLT frame")?;
    // Insert synchronized lyrics (SYLT)
    insert_sylt_frame(&mut id3v2, synced_lyrics).context("Failed to insert SYLT frame")?;

    primary_tag = id3v2.into();
    file.insert_tag(primary_tag);
    file.save_to_path(track_path, WriteOptions::default())
        .context("Failed to save MP3 file")?;

    Ok(())
}

/// Insert USLT (unsynchronized lyrics) frame into ID3v2 tag
fn insert_uslt_frame(id3v2: &mut Id3v2Tag, plain_lyrics: &str) -> Result<()> {
    // Players may select an older language/description variant over the new lyrics.
    let _ = id3v2.remove(&FrameId::new("USLT")?);
    if !plain_lyrics.is_empty() {
        let uslt_frame = UnsynchronizedTextFrame::new(
            TextEncoding::UTF8,
            [b'X', b'X', b'X'],
            "".to_string(),
            plain_lyrics.to_string(),
        );
        id3v2.insert(Frame::UnsynchronizedText(uslt_frame));
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
        // Prepare the replacement before removing every old language variant.
        let _ = id3v2.remove(&sylt_frame_id);
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
    let parsed = parse_lrc(synced_lyrics);

    let converted_lyrics: Vec<(u32, String)> = parsed
        .timed_lines
        .iter()
        .map(|timed_line| (timed_line.timestamp_ms as u32, timed_line.text.clone()))
        .collect();

    Ok(converted_lyrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_frames(tag: &Id3v2Tag, name: &str) -> usize {
        let id = FrameId::new(name).unwrap();
        tag.into_iter().filter(|frame| frame.id() == &id).count()
    }

    fn tag_with_old_lyrics() -> Id3v2Tag {
        use lofty::id3::v2::TextInformationFrame;
        let mut tag = Id3v2Tag::new();
        tag.insert(Frame::Text(TextInformationFrame::new(
            FrameId::new("TIT2").unwrap(),
            TextEncoding::UTF8,
            "original title",
        )));
        for (language, description) in [(*b"eng", "old"), (*b"XXX", "other")] {
            tag.insert(Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
                TextEncoding::UTF8,
                language,
                description,
                "old words",
            )));
            let synced = SynchronizedTextFrame::new(
                TextEncoding::UTF8,
                language,
                TimestampFormat::MS,
                SyncTextContentType::Lyrics,
                Some(description.into()),
                vec![(0, "old words".into())],
            );
            tag.insert(Frame::Binary(BinaryFrame::new(
                FrameId::new("SYLT").unwrap(),
                synced.as_bytes(WriteOptions::default()).unwrap(),
            )));
        }
        tag
    }

    fn assert_replacement(tag: &Id3v2Tag) {
        assert_eq!(count_frames(tag, "USLT"), 1);
        assert_eq!(count_frames(tag, "SYLT"), 1);
        match tag.get(&FrameId::new("USLT").unwrap()).unwrap() {
            Frame::UnsynchronizedText(frame) => {
                assert_eq!(frame.content, "new words");
                assert_eq!(frame.encoding, TextEncoding::UTF8);
                assert_eq!(frame.language, *b"XXX");
            }
            _ => panic!("expected USLT"),
        }
        match tag.get(&FrameId::new("SYLT").unwrap()).unwrap() {
            Frame::Binary(frame) => {
                let parsed = SynchronizedTextFrame::parse(&frame.data, frame.flags()).unwrap();
                assert_eq!(parsed.content, vec![(15250, "new words".into())]);
                assert_eq!(parsed.encoding, TextEncoding::UTF8);
                assert_eq!(parsed.timestamp_format, TimestampFormat::MS);
                assert_eq!(parsed.content_type, SyncTextContentType::Lyrics);
                assert_eq!(parsed.language, *b"XXX");
            }
            _ => panic!("expected SYLT"),
        }
        assert_eq!(
            tag.get_text(&FrameId::new("TIT2").unwrap()),
            Some("original title")
        );
    }

    #[test]
    fn embedded_lyrics_replace_all_variants_and_repeated_exports_are_idempotent() {
        let mut tag = tag_with_old_lyrics();
        for _ in 0..3 {
            insert_uslt_frame(&mut tag, "new words").unwrap();
            insert_sylt_frame(&mut tag, "[00:15.25]new words").unwrap();
            assert_replacement(&tag);
        }
    }

    #[test]
    fn empty_lyrics_remove_all_variants_without_removing_other_tags() {
        let mut tag = tag_with_old_lyrics();
        insert_uslt_frame(&mut tag, "").unwrap();
        insert_sylt_frame(&mut tag, "").unwrap();
        assert_eq!(count_frames(&tag, "USLT"), 0);
        assert_eq!(count_frames(&tag, "SYLT"), 0);
        assert_eq!(
            tag.get_text(&FrameId::new("TIT2").unwrap()),
            Some("original title")
        );
    }

    #[test]
    fn mp3_export_replaces_stale_lyrics_and_preserves_artwork_and_audio() {
        use lofty::config::ParseOptions;
        use lofty::id3::v2::AttachedPictureFrame;
        use lofty::picture::{MimeType, Picture, PictureType};
        use lofty::tag::TagExt;
        use std::io::Cursor;

        let directory = std::env::temp_dir().join(format!(
            "lrcget-replacement-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let path = directory.join("generated.mp3");
        std::fs::write(&path, include_bytes!("../tests/fixtures/embedding.mp3")).unwrap();
        let mut tag = tag_with_old_lyrics();
        let png = data_encoding::BASE64.decode(
            b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII="
        ).unwrap();
        let picture = Picture::unchecked(png)
            .pic_type(PictureType::CoverFront)
            .mime_type(MimeType::Png)
            .build();
        tag.insert(Frame::Picture(AttachedPictureFrame::new(
            TextEncoding::UTF8,
            picture,
        )));
        tag.save_to_path(&path, WriteOptions::default()).unwrap();

        let read = || {
            let mut data = Cursor::new(std::fs::read(&path).unwrap());
            MpegFile::read_from(&mut data, ParseOptions::default()).unwrap()
        };
        let before = read();
        let old_cover = before
            .id3v2()
            .unwrap()
            .get(&FrameId::new("APIC").unwrap())
            .unwrap()
            .clone();
        assert_eq!(count_frames(before.id3v2().unwrap(), "USLT"), 2);
        assert_eq!(count_frames(before.id3v2().unwrap(), "SYLT"), 2);
        let old_audio = encoded_fixture_audio(&path);
        for _ in 0..2 {
            embed_lyrics_mp3(path.to_str().unwrap(), "new words", "[00:15.25]new words").unwrap();
            let after = read();
            assert_replacement(after.id3v2().unwrap());
            assert_eq!(
                after.id3v2().unwrap().original_version(),
                lofty::id3::v2::Id3v2Version::V4
            );
            assert_eq!(
                after.id3v2().unwrap().get(&FrameId::new("APIC").unwrap()),
                Some(&old_cover)
            );
            assert_eq!(
                after.properties().duration(),
                before.properties().duration()
            );
            assert_eq!(encoded_fixture_audio(&path), old_audio);
        }
    }

    fn encoded_fixture_audio(path: &Path) -> Vec<Vec<u8>> {
        use symphonia::core::{
            errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions,
            probe::Hint,
        };
        let source = MediaSourceStream::new(
            Box::new(std::fs::File::open(path).unwrap()),
            Default::default(),
        );
        let mut format = symphonia::default::get_probe()
            .format(
                &Hint::new(),
                source,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .unwrap()
            .format;
        let mut packets = Vec::new();
        loop {
            match format.next_packet() {
                Ok(packet) => packets.push(packet.data.to_vec()),
                Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break
                }
                Err(error) => panic!("unable to read fixture audio: {error}"),
            }
        }
        assert!(!packets.is_empty());
        packets
    }

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
