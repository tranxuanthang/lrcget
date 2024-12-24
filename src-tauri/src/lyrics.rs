use anyhow::Result;
use lofty::{
  config::{ParseOptions, WriteOptions},
  file::AudioFile,
  flac::FlacFile,
  id3::v2::{
    BinaryFrame, Frame, FrameId, Id3v2Tag, SyncTextContentType,
    SynchronizedTextFrame, TimestampFormat, UnsynchronizedTextFrame,
  },
  mpeg::MpegFile,
  TextEncoding,
};
use std::fs::{remove_file, write, OpenOptions};
use std::path::PathBuf;
use std::path::Path;
use crate::lrclib::get::request;
use crate::lrclib::get::Response;
use crate::persistent_entities::PersistentTrack;
use thiserror::Error;
use lrc::Lyrics;

#[derive(Error, Clone, Debug)]
pub enum GetLyricsError {
  #[error("This track does not exist in LRCLIB database")]
  NotFound
}

pub async fn download_lyrics_for_track(track: PersistentTrack, is_try_embed_lyrics: bool, lrclib_instance: &str) -> Result<Response> {
  let lyrics = request(&track.title, &track.album_name, &track.artist_name, track.duration, lrclib_instance).await?;

  apply_lyrics_for_track(track, lyrics, is_try_embed_lyrics).await
}

pub async fn apply_string_lyrics_for_track(track: &PersistentTrack, plain_lyrics: &str, synced_lyrics: &str, is_try_embed_lyrics: bool) -> Result<()> {
  save_plain_lyrics(&track.file_path, plain_lyrics)?;
  save_synced_lyrics(&track.file_path, synced_lyrics)?;

  if is_try_embed_lyrics {
    embed_lyrics(&track.file_path, &plain_lyrics, &synced_lyrics);
  }

  Ok(())
}

pub async fn apply_lyrics_for_track(track: PersistentTrack, lyrics: Response, is_try_embed_lyrics: bool) -> Result<Response> {
  match &lyrics {
    Response::SyncedLyrics(synced_lyrics, plain_lyrics) => {
      save_synced_lyrics(&track.file_path, &synced_lyrics)?;
      if is_try_embed_lyrics {
        embed_lyrics(&track.file_path, &plain_lyrics, &synced_lyrics);
      }
      Ok(lyrics)
    }
    Response::UnsyncedLyrics(plain_lyrics) => {
      save_plain_lyrics(&track.file_path, &plain_lyrics)?;
      if is_try_embed_lyrics {
        embed_lyrics(&track.file_path, &plain_lyrics, "");
      }
      Ok(lyrics)
    }
    Response::IsInstrumental => {
      save_instrumental(&track.file_path)?;
      Ok(lyrics)
    }
    _ => {
      Ok(lyrics)
    }
  }
}

fn save_plain_lyrics(track_path: &str, lyrics: &str) -> Result<()> {
  let txt_path = build_txt_path(track_path)?;
  let lrc_path = build_lrc_path(track_path)?;

  let _ = remove_file(lrc_path);

  if lyrics.is_empty() {
    let _ = remove_file(txt_path);
  } else {
    write(txt_path, lyrics)?;
  }
  Ok(())
}

fn save_synced_lyrics(track_path: &str, lyrics: &str) -> Result<()> {
  let txt_path = build_txt_path(track_path)?;
  let lrc_path = build_lrc_path(track_path)?;
  if lyrics.is_empty() {
    let _ = remove_file(lrc_path);
  } else {
    let _ = remove_file(txt_path);
    write(lrc_path, lyrics)?;
  }
  Ok(())
}

fn save_instrumental(track_path: &str) -> Result<()> {
  let txt_path = build_txt_path(track_path)?;
  let lrc_path = build_lrc_path(track_path)?;

  let _ = remove_file(&lrc_path);
  let _ = remove_file(txt_path);

  write(lrc_path, "[au: instrumental]")?;

  Ok(())
}

fn build_txt_path(track_path: &str) -> Result<PathBuf> {
    let path = Path::new(track_path);
    let parent_path = path.parent().unwrap();
    let file_name_without_extension = path.file_stem().unwrap().to_str().unwrap();
    let txt_path = Path::new(parent_path).join(format!("{}.{}", file_name_without_extension, "txt"));

    Ok(txt_path)
}

fn build_lrc_path(track_path: &str) -> Result<PathBuf> {
    let path = Path::new(track_path);
    let parent_path = path.parent().unwrap();
    let file_name_without_extension = path.file_stem().unwrap().to_str().unwrap();
    let lrc_path = Path::new(parent_path).join(format!("{}.{}", file_name_without_extension, "lrc"));

    Ok(lrc_path)
}

fn embed_lyrics(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) {
  if track_path.to_lowercase().ends_with(".mp3") {
    match embed_lyrics_mp3(track_path, plain_lyrics, synced_lyrics) {
        Ok(_) => (),
        Err(e) => println!("Error embedding lyrics in MP3: {}", e),
    }
  } else if track_path.to_lowercase().ends_with(".flac") {
    match embed_lyrics_flac(track_path, plain_lyrics, synced_lyrics) {
        Ok(_) => (),
        Err(e) => println!("Error embedding lyrics in FLAC: {}", e),
    }
  }
}

fn embed_lyrics_flac(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
  let mut file_content = OpenOptions::new()
    .read(true)
    .write(true)
    .open(track_path)?;
  let mut flac_file = FlacFile::read_from(&mut file_content, ParseOptions::new())?;

  if let Some(vorbis_comments) = flac_file.vorbis_comments_mut() {
    if !plain_lyrics.is_empty() {
      vorbis_comments.insert("UNSYNCEDLYRICS".to_string(), plain_lyrics.to_string());
    } else {
      let _ = vorbis_comments.remove("UNSYNCEDLYRICS");
    }

    if !synced_lyrics.is_empty() {
      vorbis_comments.insert("LYRICS".to_string(), synced_lyrics.to_string());
    } else {
      let _ = vorbis_comments.remove("LYRICS");
    }

    flac_file.save_to_path(track_path, WriteOptions::default())?;
  }

  Ok(())
}

fn embed_lyrics_mp3(track_path: &str, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
  let mut file_content = OpenOptions::new()
    .read(true)
    .write(true)
    .open(track_path)?;
  let mut mp3_file = MpegFile::read_from(&mut file_content, ParseOptions::new())?;

  if let Some(id3v2) = mp3_file.id3v2_mut() {
    insert_id3v2_uslt_frame(id3v2, plain_lyrics)?;
    insert_id3v2_sylt_frame(id3v2, synced_lyrics)?;

    mp3_file.save_to_path(track_path, WriteOptions::default())?;
  }

  Ok(())
}

fn insert_id3v2_uslt_frame(id3v2: &mut Id3v2Tag, plain_lyrics: &str) -> Result<()> {
  if !plain_lyrics.is_empty() {
    let uslt_frame = UnsynchronizedTextFrame::new(
      TextEncoding::UTF8,
      [b'X', b'X', b'X'],
      "".to_string(),
      plain_lyrics.to_string()
    );
    id3v2.insert(Frame::UnsynchronizedText(uslt_frame));
  } else {
    let _ = id3v2.remove(&FrameId::new("USLT")?);
  }

  Ok(())
}

fn insert_id3v2_sylt_frame(id3v2: &mut Id3v2Tag, synced_lyrics: &str) -> Result<()> {
  if !synced_lyrics.is_empty() {
    let synced_lyrics_vec = synced_lyrics_to_sylt_vec(synced_lyrics)?;

    let sylt_frame = SynchronizedTextFrame::new(
      TextEncoding::UTF8,
      [b'X', b'X', b'X'],
      TimestampFormat::MS,
      SyncTextContentType::Lyrics,
      None,
      synced_lyrics_vec
    );

    let sylt_frame_byte = sylt_frame.as_bytes()?;
    let sylt_frame_id = FrameId::new("SYLT")?;
    id3v2.insert(Frame::Binary(BinaryFrame::new(sylt_frame_id, sylt_frame_byte)));
  } else {
    let _ = id3v2.remove(&FrameId::new("SYLT")?);
  }

  Ok(())
}

fn synced_lyrics_to_sylt_vec(synced_lyrics: &str) -> Result<Vec<(u32, String)>> {
  let lyrics = Lyrics::from_str(synced_lyrics)?;
  let lyrics_vec = lyrics.get_timed_lines();

  let converted_lyrics: Vec<(u32, String)> = lyrics_vec
    .iter()
    .map(|(time_tag, text)| (time_tag.get_timestamp() as u32, text.to_string()))
    .collect();

  Ok(converted_lyrics)
}
