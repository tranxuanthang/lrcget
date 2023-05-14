use anyhow::Result;
use std::fs::write;
use std::path::PathBuf;
use std::path::Path;
use crate::lrclib;
use crate::lrclib::LrclibResponseOption;
use crate::persistent_entities::PersistentTrack;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
enum GetLyricsError {
  #[error("This track only has unsynced lyrics")]
  OnlyUnsyncedLyrics,
  #[error("This track is instrumental")]
  IsInstrumental,
  #[error("This track does not exist in lrclib database")]
  NotFound
}

pub async fn download_lyrics_for_track(track: PersistentTrack) -> Result<String> {
  let lyrics = lrclib::retrieve_lyrics(&track.title, &track.album_name, &track.artist_name, track.duration).await?;
  match lyrics {
    LrclibResponseOption::SyncedLyrics(synced_lyrics) => {
      save_lyrics(&track.file_path, &synced_lyrics)?;
      Ok(synced_lyrics.to_owned())
    },
    LrclibResponseOption::UnsyncedLyrics(_) => {
      Err(GetLyricsError::OnlyUnsyncedLyrics.into())
    },
    LrclibResponseOption::IsInstrumental => {
      Err(GetLyricsError::IsInstrumental.into())
    },
    LrclibResponseOption::None => {
      Err(GetLyricsError::NotFound.into())
    }
  }
}

fn save_lyrics(track_path: &str, lyrics: &str) -> Result<()> {
  let lrc_path = build_lrc_path(track_path)?;
  write(lrc_path, lyrics)?;
  Ok(())
}

fn build_lrc_path(track_path: &str) -> Result<PathBuf> {
    let path = Path::new(track_path);
    let parent_path = path.parent().unwrap();
    let file_name_without_extension = path.file_stem().unwrap().to_str().unwrap();
    let lrc_path = Path::new(parent_path).join(format!("{}.{}", file_name_without_extension, "lrc"));

    Ok(lrc_path)
}
