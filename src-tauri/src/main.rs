#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

pub mod db;
pub mod fs_track;
pub mod library;
pub mod persistent_entities;
pub mod lrclib;
pub mod lyrics;
pub mod state;

use persistent_entities::{PersistentTrack, PersistentAlbum, PersistentArtist, PersistentConfig};
use tauri::{State, Manager, AppHandle};
use rusqlite::Connection;
use state::{AppState, ServiceAccess};

#[tauri::command]
async fn get_directories(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let directories =db::get_directories(conn);
  match directories {
    Ok(directories) => Ok(directories),
    Err(error) => Err(format!("Cannot get existing directories from database. Error: {}", error))
  }
}

#[tauri::command]
async fn set_directories(directories: Vec<String>, app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  db::set_directories(directories, conn).map_err(|err| err.to_string())?;

  Ok(())
}

#[tauri::command]
async fn get_init(app_state: State<'_, AppState>) -> Result<bool, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let init = library::get_init(conn).map_err(|err| err.to_string())?;

  Ok(init)
}

#[tauri::command]
async fn get_config(app_state: State<'_, AppState>) -> Result<PersistentConfig, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let config = db::get_config(conn).map_err(|err| err.to_string())?;

  Ok(config)
}

#[tauri::command]
async fn set_config(skip_not_needed_tracks: bool, try_embed_lyrics: bool, app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  db::set_config(skip_not_needed_tracks, try_embed_lyrics, conn).map_err(|err| err.to_string())?;

  Ok(())
}

#[tauri::command]
async fn initialize_library(app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  library::initialize_library(conn).map_err(|err| err.to_string())?;

  Ok(())
}

#[tauri::command]
async fn uninitialize_library(app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();

  library::uninitialize_library(conn).map_err(|err| err.to_string())?;

  Ok(())
}

#[tauri::command]
async fn refresh_library(app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();

  library::uninitialize_library(conn).map_err(|err| err.to_string())?;
  library::initialize_library(conn).map_err(|err| err.to_string())?;

  Ok(())
}

#[tauri::command]
async fn get_tracks(app_state: State<'_, AppState>) -> Result<Vec<PersistentTrack>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let tracks = library::get_tracks(conn).map_err(|err| err.to_string())?;

  Ok(tracks)
}

#[tauri::command]
async fn get_albums(app_state: State<'_, AppState>) -> Result<Vec<PersistentAlbum>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let albums = library::get_albums(conn).map_err(|err| err.to_string())?;

  Ok(albums)
}

#[tauri::command]
async fn get_artists(app_state: State<'_, AppState>) -> Result<Vec<PersistentArtist>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let artists = library::get_artists(conn).map_err(|err| err.to_string())?;

  Ok(artists)
}

#[tauri::command]
async fn get_album_tracks(album_id: i64, app_state: State<'_, AppState>) -> Result<Vec<PersistentTrack>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let tracks = library::get_album_tracks(album_id, conn).map_err(|err| err.to_string())?;

  Ok(tracks)
}

#[tauri::command]
async fn get_artist_tracks(artist_id: i64, app_state: State<'_, AppState>) -> Result<Vec<PersistentTrack>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let tracks = library::get_artist_tracks(artist_id, conn).map_err(|err| err.to_string())?;

  Ok(tracks)
}

#[tauri::command]
async fn download_lyrics(track_id: i64, app_handle: AppHandle) -> Result<(), String> {
  let track = app_handle.db(|db|  db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let synced_lyrics = lyrics::download_lyrics_for_track(track).await.map_err(|err| err.to_string())?;
  app_handle.db(|db: &Connection|  db::update_track_lrc_lyrics(track_id, &synced_lyrics, db)).map_err(|err| err.to_string())?;

  std::thread::spawn(move || {
    app_handle.emit_all("reload-database", ()).unwrap();
  });

  Ok(())
}

#[tauri::command]
fn open_devtools(window: tauri::Window) {
  #[cfg(debug_assertions)]
  {
    window.open_devtools();
  }
}

fn main() {
  tauri::Builder::default()
    .manage(AppState { db: Default::default() })
    .setup(|app| {
      let handle = app.handle();

      let app_state: State<AppState> = handle.state();
      let db = db::initialize_database(&handle).expect("Database initialize should succeed");
      *app_state.db.lock().unwrap() = Some(db);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_directories,
      set_directories,
      get_init,
      get_config,
      set_config,
      initialize_library,
      uninitialize_library,
      refresh_library,
      get_tracks,
      get_albums,
      get_artists,
      get_album_tracks,
      get_artist_tracks,
      download_lyrics,
      open_devtools
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
