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
use axum::{
  http::{HeaderValue, Method},
  Router
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

#[tauri::command]
async fn get_directories(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let directories = db::get_directories(conn);
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
async fn initialize_library(app_state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
  let mut conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_mut().unwrap();
  library::initialize_library(conn, app_handle).map_err(|err| err.to_string())?;

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
async fn refresh_library(app_state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
  let mut conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_mut().unwrap();

  library::uninitialize_library(conn).map_err(|err| err.to_string())?;
  library::initialize_library(conn, app_handle).map_err(|err| err.to_string())?;

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
async fn download_lyrics(track_id: i64, app_handle: AppHandle) -> Result<String, String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let lyrics = lyrics::download_lyrics_for_track(track).await.map_err(|err| err.to_string())?;
  match lyrics {
    lrclib::get::Response::SyncedLyrics(synced_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_lrc_lyrics(track_id, &synced_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-database", ()).unwrap();
      });
      Ok("Synced lyrics downloaded".to_owned())
    }
    lrclib::get::Response::UnsyncedLyrics(plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_txt_lyrics(track_id, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-database", ()).unwrap();
      });
      Ok("Plain lyrics downloaded".to_owned())
    }
    lrclib::get::Response::IsInstrumental => {
      Err(lyrics::GetLyricsError::IsInstrumental.to_string())
    }
    lrclib::get::Response::None => {
      Err(lyrics::GetLyricsError::NotFound.to_string())
    }
  }
}

#[tauri::command]
async fn apply_lyrics(track_id: i64, lrclib_response: lrclib::get::RawResponse, app_handle: AppHandle) -> Result<String, String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let lyrics = lrclib::get::Response::from_raw_response(lrclib_response);
  let lyrics = lyrics::apply_lyrics_for_track(track, lyrics).await.map_err(|err| err.to_string())?;

  match lyrics {
    lrclib::get::Response::SyncedLyrics(synced_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_lrc_lyrics(track_id, &synced_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-database", ()).unwrap();
      });
      Ok("Synced lyrics downloaded".to_owned())
    }
    lrclib::get::Response::UnsyncedLyrics(plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_txt_lyrics(track_id, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-database", ()).unwrap();
      });
      Ok("Plain lyrics downloaded".to_owned())
    }
    lrclib::get::Response::IsInstrumental => {
      Err(lyrics::GetLyricsError::IsInstrumental.to_string())
    }
    lrclib::get::Response::None => {
      Err(lyrics::GetLyricsError::NotFound.to_string())
    }
  }
}

#[tauri::command]
async fn retrieve_lyrics(title: String, album_name: String, artist_name: String, duration: f64, app_handle: AppHandle) -> Result<lrclib::get::RawResponse, String> {
  let response = lrclib::get::request_raw(&title, &album_name, &artist_name, duration).await.map_err(|err| err.to_string())?;

  Ok(response)
}

#[tauri::command]
async fn search_lyrics(title: String, album_name: String, artist_name: String, app_handle: AppHandle) -> Result<lrclib::search::Response, String> {
  let response = lrclib::search::request(&title, &album_name, &artist_name).await.map_err(|err| err.to_string())?;

  Ok(response)
}

#[tauri::command]
fn open_devtools(window: tauri::Window) {
  {
    window.open_devtools();
  }
}

#[tauri::command]
fn convert_file_src_2(path: String) -> String {
  format!("http://localhost:16780{}", path)
}

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .manage(AppState { db: Default::default() })
    .setup(|app| {
      let handle = app.handle();

      let app_state: State<AppState> = handle.state();
      let db = db::initialize_database(&handle).expect("Database initialize should succeed");
      *app_state.db.lock().unwrap() = Some(db);

      #[cfg(target_os = "linux")]
      tokio::spawn(async move {
        let serve_dir = ServeDir::new("/");

        let axum_app = Router::new()
          .nest_service("/", serve_dir)
          .layer(
            CorsLayer::new()
              .allow_origin("*".parse::<HeaderValue>().unwrap())
              .allow_methods([Method::GET])
          );

        axum::Server::bind(&"127.0.0.1:16780".parse().unwrap())
          .serve(axum_app.into_make_service()).await.unwrap();
      });

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
      apply_lyrics,
      retrieve_lyrics,
      search_lyrics,
      open_devtools,
      convert_file_src_2
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
