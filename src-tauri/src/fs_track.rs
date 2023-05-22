use globwalk::glob;
use lofty::{read_from_path, AudioFile};
use lofty::TaggedFileExt;
use lofty::Accessor;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FsTrack {
  file_path: String,
  file_name: String,
  title: String,
  album: String,
  artist: String,
  duration: f64,
  lrc_lyrics: Option<String>
}

#[derive(Error, Debug)]
pub enum FsTrackError {
  #[error("No title was found from track")]
  TitleNotFound,
  #[error("No album name was found from track")]
  AlbumNotFound,
  #[error("No artist name was found from track")]
  ArtistNotFound,
  #[error("No primary tag was found from track")]
  PrimaryTagNotFound
}

impl FsTrack {
  fn new(file_path: String, file_name: String, title: String, album: String, artist: String, duration: f64, lrc_lyrics: Option<String>) -> FsTrack {
    FsTrack {
      file_path,
      file_name,
      title,
      album,
      artist,
      duration,
      lrc_lyrics,
    }
  }

  fn new_from_path(path: &Path) -> Result<FsTrack> {
    let file_path = path.display().to_string();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let tagged_file = read_from_path(&file_path)?;
    let tag = tagged_file.primary_tag().ok_or(FsTrackError::PrimaryTagNotFound)?;
    let owned_tag = tag.to_owned();
    let properties = tagged_file.properties();
    let title = owned_tag.title().ok_or(FsTrackError::TitleNotFound)?.to_string();
    let album = owned_tag.album().ok_or(FsTrackError::AlbumNotFound)?.to_string();
    let artist = owned_tag.artist().ok_or(FsTrackError::ArtistNotFound)?.to_string();
    let duration = properties.duration().as_secs_f64();

    let mut track = FsTrack::new(file_path, file_name, title, album, artist, duration, None);
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

  pub fn duration(&self) -> f64 {
    self.duration
  }

  pub fn lrc_lyrics(&self) -> Option<String> {
    self.lrc_lyrics.to_owned()
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

pub fn load_tracks_from_directories(directories: &Vec<String>) -> Result<Vec<FsTrack>> {
  let files = get_files_from_directories(&directories)?;

  let mut tracks: Vec<FsTrack> = vec![];

  let tracks_result: Vec<(String, Result<FsTrack>)> = files.par_iter()
    .map(|file| (file.to_owned(), FsTrack::new_from_path(Path::new(file))))
    .collect();

  for (file, track) in tracks_result {
    match track {
      Ok(track) => {
        tracks.push(track)
      }
      Err(err) => {
        println!("Error while creating new track entry. File path: \"{}\", error: \"{}\"", file, err);
      }
    }
  }

  Ok(tracks)
}

pub fn get_files_from_directories(directories: &Vec<String>) -> Result<Vec<String>> {
  let mut files: Vec<String> = vec![];
  for directory in directories.iter() {
    let files_in_dir = glob(format!("{}/**/*.{{mp3,m4a,flac,ogg}}", directory))?;
    for file in files_in_dir {
      let file = file?;
      let path = file.path();
      files.push(path.display().to_string());
    }
  }

  Ok(files)
}
