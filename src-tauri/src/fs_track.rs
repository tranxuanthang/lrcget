use lofty::error::LoftyError;
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;
use anyhow::Result;
use rusqlite::Connection;
use tauri::AppHandle;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use thiserror::Error;
use std::time::Instant;
use crate::db;
use tauri::Manager;
use walkdir::{WalkDir, DirEntry};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FsTrack {
  file_path: String,
  file_name: String,
  title: String,
  album: String,
  artist: String,
  album_artist: String,
  duration: f64,
  txt_lyrics: Option<String>,
  lrc_lyrics: Option<String>
}

#[derive(Error, Debug)]
pub enum FsTrackError {
  #[error("Cannot parse the tag info from track: `{0}`. Error: `{1}`")]
  ParseFailed(String, LoftyError),
  #[error("No title was found from track: `{0}`")]
  TitleNotFound(String),
  #[error("No album name was found from track: `{0}`")]
  AlbumNotFound(String),
  #[error("No artist name was found from track: `{0}`")]
  ArtistNotFound(String),
  #[error("No primary tag was found from track: `{0}`")]
  PrimaryTagNotFound(String)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgress {
  progress: Option<f64>,
  files_scanned: usize,
  files_count: Option<usize>
}

impl FsTrack {
  fn new(file_path: String, file_name: String, title: String, album: String, artist: String, album_artist: String, duration: f64, txt_lyrics: Option<String>, lrc_lyrics: Option<String>) -> FsTrack {
    FsTrack {
      file_path,
      file_name,
      title,
      album,
      artist,
      album_artist,
      duration,
      txt_lyrics,
      lrc_lyrics
    }
  }

  fn new_from_path(path: &Path) -> Result<FsTrack> {
    let file_path = path.display().to_string();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let tagged_file = read_from_path(&file_path).or_else(|err| Err(FsTrackError::ParseFailed(file_path.to_owned(), err)))?;
    let tag = tagged_file.primary_tag().ok_or(FsTrackError::PrimaryTagNotFound(file_path.to_owned()))?.to_owned();
    let properties = tagged_file.properties();
    let title = tag.title().ok_or(FsTrackError::TitleNotFound(file_path.to_owned()))?.to_string();
    let album = tag.album().ok_or(FsTrackError::AlbumNotFound(file_path.to_owned()))?.to_string();
    let artist = tag.artist().ok_or(FsTrackError::ArtistNotFound(file_path.to_owned()))?.to_string();
    let album_artist = tag
        .get_string(&lofty::tag::ItemKey::AlbumArtist)
        .map(|s| s.to_string())
        .unwrap_or_else(|| artist.clone());
    let duration = properties.duration().as_secs_f64();

    let mut track = FsTrack::new(file_path, file_name, title, album, artist, album_artist, duration, None, None);
    track.txt_lyrics = track.get_txt_lyrics();
    track.lrc_lyrics = track.get_lrc_lyrics();

    Ok(track)
  }

  pub fn file_path(&self) -> String {
    self.file_path.to_owned()
  }

  pub fn file_name(&self) -> String {
    self.file_name.to_owned()
  }

  pub fn title(&self) -> String {
    self.title.to_owned()
  }

  pub fn album(&self) -> String {
    self.album.to_owned()
  }

  pub fn artist(&self) -> String {
    self.artist.to_owned()
  }

  pub fn album_artist(&self) -> String {
    self.album_artist.to_owned()
  }

  pub fn duration(&self) -> f64 {
    self.duration
  }

  pub fn txt_lyrics(&self) -> Option<String> {
    self.txt_lyrics.to_owned()
  }

  pub fn lrc_lyrics(&self) -> Option<String> {
    self.lrc_lyrics.to_owned()
  }

  fn get_txt_path(&self) -> String {
    let path = PathBuf::from(self.file_path.to_owned());
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let parent_path = path.parent().unwrap();
    let file_name_without_extension = std::path::Path::new(&file_name)
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();
    let mut txt_file_name = file_name_without_extension.to_owned();
    txt_file_name.push_str(".txt");

    let txt_file_path = parent_path.join(txt_file_name).display().to_string();

    txt_file_path
  }

  fn get_txt_lyrics(&self) -> Option<String> {
    let txt_file_path = self.get_txt_path();
    let txt_content = std::fs::read_to_string(txt_file_path);

    match txt_content {
      Ok(txt_content) => Some(txt_content),
      Err(_) => None
    }
  }

  fn get_lrc_path(&self) -> String {
    let path = PathBuf::from(self.file_path.to_owned());
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let parent_path = path.parent().unwrap();
    let file_name_without_extension = std::path::Path::new(&file_name)
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();
    let mut lrc_file_name = file_name_without_extension.to_owned();
    lrc_file_name.push_str(".lrc");

    let lrc_file_path = parent_path.join(lrc_file_name).display().to_string();

    lrc_file_path
  }

  fn get_lrc_lyrics(&self) -> Option<String> {
    let lrc_file_path = self.get_lrc_path();
    let lrc_content = std::fs::read_to_string(lrc_file_path);

    match lrc_content {
      Ok(lrc_content) => Some(lrc_content),
      Err(_) => None
    }
  }
}

fn load_tracks_from_entry_batch(entry_batch: &Vec<DirEntry>) -> Result<Vec<FsTrack>> {
  let track_results: Vec<Result<FsTrack>> = entry_batch
    .par_iter()
    .map(|file| FsTrack::new_from_path(file.path()))
    .collect();

  let mut tracks: Vec<FsTrack> = vec![];

  for track_result in track_results {
    match track_result {
      Ok(track) => {
        tracks.push(track);
      }
      Err(error) => {
        println!("{}", error);
      }
    }
  }

  Ok(tracks)
}

pub fn load_tracks_from_directories(directories: &Vec<String>, conn: &mut Connection, app_handle: AppHandle) -> Result<()> {
  let now = Instant::now();
  let files_count = count_files_from_directories(directories)?;
  println!("Files count: {}", files_count);
  let mut files_scanned: usize = 0;
  for directory in directories.iter() {
    let mut entry_batch: Vec<DirEntry> = vec![];
    for entry in WalkDir::new(directory).follow_links(true).into_iter().filter_map(|e| e.ok()) {
      if let Some(extension) = entry.path().extension() {
          match extension.to_str().unwrap_or("").to_lowercase().as_str() {
              "mp3" | "m4a" | "flac" | "ogg" | "opus" | "wav" => {
                  entry_batch.push(entry);
              },
              _ => {}
          }
      }
    if entry_batch.len() == 100 {
      let tracks = load_tracks_from_entry_batch(&entry_batch)?;

      db::add_tracks(&tracks, conn)?;
      files_scanned += entry_batch.len();
      app_handle.emit_all("initialize-progress", ScanProgress { progress: None, files_scanned, files_count: Some(files_count) }).unwrap();
      entry_batch.clear();
    }
  }

    let tracks = load_tracks_from_entry_batch(&entry_batch)?;
    db::add_tracks(&tracks, conn)?;
    files_scanned += entry_batch.len();
    app_handle.emit_all("initialize-progress", ScanProgress { progress: None, files_scanned, files_count: Some(files_count) }).unwrap();
  }
  println!("==> Scanning tracks take: {}ms", now.elapsed().as_millis());

  Ok(())
}


pub fn count_files_from_directories(directories: &Vec<String>) -> Result<usize> {
  let mut files_count = 0;

  for directory in directories.iter() {
      println!("Scanning directory: {}", directory);

      for entry in WalkDir::new(directory).follow_links(true).into_iter().filter_map(Result::ok) {
          let path = entry.path();

          if let Some(extension) = path.extension() {
              match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                  "mp3" | "m4a" | "flac" | "ogg" | "opus" | "wav" => {
                      files_count += 1;
                  },
                  _ => {}
              }
          }
      }
  }

  Ok(files_count)
}
