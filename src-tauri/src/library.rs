use anyhow::Result;
use rusqlite::Connection;
use crate::persistent_entities::{PersistentTrack, PersistentAlbum, PersistentArtist};
use crate::fs_track::{self, FsTrack};
use crate::db;

pub fn initialize_library(conn: &Connection) -> Result<()> {
  let init = db::get_init(conn)?;
  if init {
    return Ok(())
  }

  let directories = db::get_directories(conn)?;
  let tracks = fs_track::load_tracks_from_directories(&directories)?;
  let result = add_tracks(tracks, conn);
  match result {
    Ok(()) => {
      db::set_init(true, conn)?;
      Ok(())
    },
    Err(err) => {
      let uninitialization = uninitialize_library(conn);
      if let Err(uninit_error) = uninitialization {
        println!("Uninitialization library errored. Message: {}", uninit_error.to_string());
      }
      Err(err)
    }
  }
}

pub fn uninitialize_library(conn: &Connection) -> Result<()> {
  db::clean_library(conn)?;
  db::set_init(false, conn)?;
  Ok(())
}

pub fn add_tracks(tracks: Vec<FsTrack>, conn: &Connection) -> Result<()> {
  for track in tracks.iter() {
    db::add_track(&track, conn)?;
  }
  Ok(())
}

pub fn get_tracks(conn: &Connection) -> Result<Vec<PersistentTrack>> {
  db::get_tracks(conn)
}

pub fn get_albums(conn: &Connection) -> Result<Vec<PersistentAlbum>> {
  db::get_albums(conn)
}

pub fn get_artists(conn: &Connection) -> Result<Vec<PersistentArtist>> {
  db::get_artists(conn)
}

pub fn get_album_tracks(album_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
  db::get_album_tracks(album_id, conn)
}

pub fn get_artist_tracks(artist_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
  db::get_artist_tracks(artist_id, conn)
}

pub fn get_init(conn: &Connection) -> Result<bool> {
  db::get_init(conn)
}
