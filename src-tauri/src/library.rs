use anyhow::Result;
use rusqlite::Connection;
use tauri::AppHandle;
use crate::persistent_entities::{PersistentTrack, PersistentAlbum, PersistentArtist};
use crate::fs_track::{self, FsTrack};
use crate::db;

pub fn initialize_library(conn: &mut Connection, app_handle: AppHandle) -> Result<()> {
  let init = db::get_init(conn)?;
  if init {
    return Ok(())
  }

  db::clean_library(conn)?;

  let directories = db::get_directories(conn)?;
  let result = fs_track::load_tracks_from_directories(&directories, conn, app_handle);

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

pub fn get_track_ids(conn: &Connection) -> Result<Vec<i64>> {
  db::get_track_ids(conn)
}

pub fn get_no_lyrics_track_ids(conn: &Connection) -> Result<Vec<i64>> {
  db::get_no_lyrics_track_ids(conn)
}

pub fn get_track(id: i64, conn: &Connection) -> Result<PersistentTrack> {
  db::get_track_by_id(id, conn)
}

pub fn get_albums(conn: &Connection) -> Result<Vec<PersistentAlbum>> {
  db::get_albums(conn)
}

pub fn get_album_ids(conn: &Connection) -> Result<Vec<i64>> {
  db::get_album_ids(conn)
}

pub fn get_album(id: i64, conn: &Connection) -> Result<PersistentAlbum> {
  db::get_album_by_id(id, conn)
}

pub fn get_artists(conn: &Connection) -> Result<Vec<PersistentArtist>> {
  db::get_artists(conn)
}

pub fn get_artist_ids(conn: &Connection) -> Result<Vec<i64>> {
  db::get_artist_ids(conn)
}

pub fn get_artist(id: i64, conn: &Connection) -> Result<PersistentArtist> {
  db::get_artist_by_id(id, conn)
}

pub fn get_album_tracks(album_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
  db::get_album_tracks(album_id, conn)
}

pub fn get_artist_tracks(artist_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
  db::get_artist_tracks(artist_id, conn)
}

pub fn get_album_track_ids(album_id: i64, conn: &Connection) -> Result<Vec<i64>> {
  db::get_album_track_ids(album_id, conn)
}

pub fn get_artist_track_ids(artist_id: i64, conn: &Connection) -> Result<Vec<i64>> {
  db::get_artist_track_ids(artist_id, conn)
}

pub fn get_init(conn: &Connection) -> Result<bool> {
  db::get_init(conn)
}
