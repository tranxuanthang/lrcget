use anyhow::Result;
use std::fs::{write,remove_file};
use std::path::PathBuf;
use std::path::Path;
use crate::lrclib::get::request;
use crate::lrclib::get::Response;
use crate::persistent_entities::PersistentTrack;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum GetLyricsError {
  #[error("This track does not exist in LRCLIB database")]
  NotFound
}

pub async fn download_lyrics_for_track(track: PersistentTrack) -> Result<Response> {
  let lyrics = request(&track.title, &track.album_name, &track.artist_name, track.duration).await?;

  apply_lyrics_for_track(track, lyrics).await
}

pub async fn apply_string_lyrics_for_track(track: &PersistentTrack, plain_lyrics: &str, synced_lyrics: &str) -> Result<()> {
  save_plain_lyrics(&track.file_path, plain_lyrics)?;
  save_synced_lyrics(&track.file_path, synced_lyrics)?;

  Ok(())
}

pub async fn apply_lyrics_for_track(track: PersistentTrack, lyrics: Response) -> Result<Response> {
  match &lyrics {
    Response::SyncedLyrics(synced_lyrics, _) => {
      save_synced_lyrics(&track.file_path, &synced_lyrics)?;
      Ok(lyrics)
    }
    Response::UnsyncedLyrics(plain_lyrics) => {
      save_plain_lyrics(&track.file_path, &plain_lyrics)?;
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

  let rm_result = remove_file(lrc_path);
  match rm_result {
    _ => ()
  }

  if lyrics.is_empty() {
    let rm_result = remove_file(txt_path);
    match rm_result {
      _ => ()
    }
  } else {
    write(txt_path, lyrics)?;
  }
  Ok(())
}

fn save_synced_lyrics(track_path: &str, lyrics: &str) -> Result<()> {
  let txt_path = build_txt_path(track_path)?;
  let lrc_path = build_lrc_path(track_path)?;
  if lyrics.is_empty() {
    let rm_result = remove_file(lrc_path);
    match rm_result {
      _ => ()
    }
  } else {
    let rm_result = remove_file(txt_path);
    match rm_result {
      _ => ()
    }
    write(lrc_path, lyrics)?;
  }
  Ok(())
}

fn save_instrumental(track_path: &str) -> Result<()> {
  let txt_path = build_txt_path(track_path)?;
  let lrc_path = build_lrc_path(track_path)?;

  let rm_result = remove_file(&lrc_path);
  match rm_result {
    _ => ()
  }

  let rm_result = remove_file(txt_path);
  match rm_result {
    _ => ()
  }

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
