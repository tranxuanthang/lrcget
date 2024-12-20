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
pub mod player;
pub mod utils;

use persistent_entities::{PersistentTrack, PersistentAlbum, PersistentArtist, PersistentConfig};
use player::Player;
use tauri::{State, Manager, AppHandle};
use rusqlite::Connection;
use state::{AppState, ServiceAccess};
use serde::Serialize;
use regex::Regex;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishLyricsProgress {
  request_challenge: String,
  solve_challenge: String,
  publish_lyrics: String
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagLyricsProgress {
  request_challenge: String,
  solve_challenge: String,
  flag_lyrics: String
}

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
async fn set_config(skip_not_needed_tracks: bool, try_embed_lyrics: bool, theme_mode: &str, app_state: State<'_, AppState>) -> Result<(), String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  db::set_config(skip_not_needed_tracks, try_embed_lyrics, theme_mode, conn).map_err(|err| err.to_string())?;

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
async fn get_track_ids(
    search_query: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let search_query = search_query.filter(|s| !s.is_empty());
  let track_ids = library::get_track_ids(search_query, conn)
    .map_err(|err| err.to_string())?;

  Ok(track_ids)
}

#[tauri::command]
async fn get_no_lyrics_track_ids(app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let track_ids = library::get_no_lyrics_track_ids(conn).map_err(|err| err.to_string())?;

  Ok(track_ids)
}

#[tauri::command]
async fn get_track(track_id: i64, app_state: State<'_, AppState>) -> Result<PersistentTrack, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let track = library::get_track(track_id, conn).map_err(|err| err.to_string())?;

  Ok(track)
}


#[tauri::command]
async fn get_albums(app_state: State<'_, AppState>) -> Result<Vec<PersistentAlbum>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let albums = library::get_albums(conn).map_err(|err| err.to_string())?;

  Ok(albums)
}

#[tauri::command]
async fn get_album_ids(app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let album_ids = library::get_album_ids(conn).map_err(|err| err.to_string())?;

  Ok(album_ids)
}

#[tauri::command]
async fn get_album(album_id: i64, app_state: State<'_, AppState>) -> Result<PersistentAlbum, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let album = library::get_album(album_id, conn).map_err(|err| err.to_string())?;

  Ok(album)
}

#[tauri::command]
async fn get_artists(app_state: State<'_, AppState>) -> Result<Vec<PersistentArtist>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let artists = library::get_artists(conn).map_err(|err| err.to_string())?;

  Ok(artists)
}

#[tauri::command]
async fn get_artist_ids(app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let artist_ids = library::get_artist_ids(conn).map_err(|err| err.to_string())?;

  Ok(artist_ids)
}

#[tauri::command]
async fn get_artist(artist_id: i64, app_state: State<'_, AppState>) -> Result<PersistentArtist, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let artist = library::get_artist(artist_id, conn).map_err(|err| err.to_string())?;

  Ok(artist)
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
async fn get_album_track_ids(album_id: i64, app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let track_ids = library::get_album_track_ids(album_id, conn).map_err(|err| err.to_string())?;

  Ok(track_ids)
}

#[tauri::command]
async fn get_artist_track_ids(artist_id: i64, app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
  let conn_guard = app_state.db.lock().unwrap();
  let conn = conn_guard.as_ref().unwrap();
  let track_ids = library::get_artist_track_ids(artist_id, conn).map_err(|err| err.to_string())?;

  Ok(track_ids)
}

#[tauri::command]
async fn download_lyrics(track_id: i64, app_handle: AppHandle) -> Result<String, String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let is_try_embed_lyrics = app_handle.db(|db| db::get_config(db)).map_err(|err| err.to_string())?.try_embed_lyrics;
  let lyrics = lyrics::download_lyrics_for_track(track, is_try_embed_lyrics).await.map_err(|err| err.to_string())?;
  match lyrics {
    lrclib::get::Response::SyncedLyrics(synced_lyrics, plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_synced_lyrics(track_id, &synced_lyrics, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      app_handle.emit_all("reload-track-id", track_id).unwrap();
      Ok("Synced lyrics downloaded".to_owned())
    }
    lrclib::get::Response::UnsyncedLyrics(plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_plain_lyrics(track_id, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      app_handle.emit_all("reload-track-id", track_id).unwrap();
      Ok("Plain lyrics downloaded".to_owned())
    }
    lrclib::get::Response::IsInstrumental => {
      app_handle.db(|db: &Connection| db::update_track_instrumental(track_id, db)).map_err(|err| err.to_string())?;
      Ok("Marked track as instrumental".to_owned())
    }
    lrclib::get::Response::None => {
      Err(lyrics::GetLyricsError::NotFound.to_string())
    }
  }
}

#[tauri::command]
async fn apply_lyrics(track_id: i64, lrclib_response: lrclib::get::RawResponse, app_handle: AppHandle) -> Result<String, String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let is_try_embed_lyrics = app_handle.db(|db| db::get_config(db)).map_err(|err| err.to_string())?.try_embed_lyrics;

  let lyrics = lrclib::get::Response::from_raw_response(lrclib_response);
  let lyrics = lyrics::apply_lyrics_for_track(track, lyrics, is_try_embed_lyrics).await.map_err(|err| err.to_string())?;

  match lyrics {
    lrclib::get::Response::SyncedLyrics(synced_lyrics, plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_synced_lyrics(track_id, &synced_lyrics, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-track-id", track_id).unwrap();
      });
      Ok("Synced lyrics downloaded".to_owned())
    }
    lrclib::get::Response::UnsyncedLyrics(plain_lyrics) => {
      app_handle.db(|db: &Connection| db::update_track_plain_lyrics(track_id, &plain_lyrics, db)).map_err(|err| err.to_string())?;
      std::thread::spawn(move || {
        app_handle.emit_all("reload-track-id", track_id).unwrap();
      });
      Ok("Plain lyrics downloaded".to_owned())
    }
    lrclib::get::Response::IsInstrumental => {
      app_handle.db(|db: &Connection| db::update_track_instrumental(track_id, db)).map_err(|err| err.to_string())?;
      Ok("Marked track as instrumental".to_owned())
    }
    lrclib::get::Response::None => {
      Err(lyrics::GetLyricsError::NotFound.to_string())
    }
  }
}

#[tauri::command]
async fn retrieve_lyrics(title: String, album_name: String, artist_name: String, duration: f64) -> Result<lrclib::get::RawResponse, String> {
  let response = lrclib::get::request_raw(&title, &album_name, &artist_name, duration).await.map_err(|err| err.to_string())?;

  Ok(response)
}

#[tauri::command]
async fn retrieve_lyrics_by_id(id: i64) -> Result<lrclib::get_by_id::RawResponse, String> {
  let response = lrclib::get_by_id::request_raw(id).await.map_err(|err| err.to_string())?;

  Ok(response)
}

#[tauri::command]
async fn search_lyrics(title: String, album_name: String, artist_name: String, q: String) -> Result<lrclib::search::Response, String> {
  let response = lrclib::search::request(&title, &album_name, &artist_name, &q).await.map_err(|err| err.to_string())?;

  Ok(response)
}

#[tauri::command]
async fn save_lyrics(track_id: i64, plain_lyrics: String, synced_lyrics: String, app_handle: AppHandle) -> Result<String, String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;
  let is_try_embed_lyrics = app_handle.db(|db| db::get_config(db)).map_err(|err| err.to_string())?.try_embed_lyrics;

  // Create a regex to match "[au: instrumental]" or "[au:instrumental]"
  let re = Regex::new(r"\[au:\s*instrumental\]").expect("Invalid regex");
  let is_instrumental = re.is_match(&synced_lyrics);

  lyrics::apply_string_lyrics_for_track(&track, &plain_lyrics, &synced_lyrics, is_try_embed_lyrics).await.map_err(|err| err.to_string())?;

  if is_instrumental {
    app_handle.db(|db: &Connection| db::update_track_instrumental(track.id, db)).map_err(|err| err.to_string())?;
  } else if !synced_lyrics.is_empty() {
    app_handle.db(|db: &Connection| db::update_track_synced_lyrics(track.id, &synced_lyrics, &plain_lyrics, db)).map_err(|err| err.to_string())?;
  } else if !plain_lyrics.is_empty() {
    app_handle.db(|db: &Connection| db::update_track_plain_lyrics(track.id, &plain_lyrics, db)).map_err(|err| err.to_string())?;
  } else {
    app_handle.db(|db: &Connection| db::update_track_null_lyrics(track.id, db)).map_err(|err| err.to_string())?;
  }

  app_handle.emit_all("reload-track-id", track_id).unwrap();

  Ok("Lyrics saved successfully".to_owned())
}

#[tauri::command]
async fn publish_lyrics(title: String, album_name: String, artist_name: String, duration: f64, plain_lyrics: String, synced_lyrics: String, app_handle: AppHandle) -> Result<(), String> {
  let mut progress = PublishLyricsProgress {
    request_challenge: "Pending".to_owned(),
    solve_challenge: "Pending".to_owned(),
    publish_lyrics: "Pending".to_owned()
  };
  progress.request_challenge = "In Progress".to_owned();
  app_handle.emit_all("publish-lyrics-progress", &progress).unwrap();
  let challenge_response = lrclib::request_challenge::request().await.map_err(|err| err.to_string())?;
  progress.request_challenge = "Done".to_owned();
  progress.solve_challenge = "In Progress".to_owned();
  app_handle.emit_all("publish-lyrics-progress", &progress).unwrap();
  let nonce = lrclib::challenge_solver::solve_challenge(&challenge_response.prefix, &challenge_response.target);
  progress.solve_challenge = "Done".to_owned();
  progress.publish_lyrics = "In Progress".to_owned();
  app_handle.emit_all("publish-lyrics-progress", &progress).unwrap();
  let publish_token = format!("{}:{}", challenge_response.prefix, nonce);
  lrclib::publish::request(&title, &album_name, &artist_name, duration, &plain_lyrics, &synced_lyrics, &publish_token).await.map_err(|err| err.to_string())?;
  progress.publish_lyrics = "Done".to_owned();
  app_handle.emit_all("publish-lyrics-progress", &progress).unwrap();
  Ok(())
}

#[tauri::command]
async fn flag_lyrics(track_id: i64, flag_reason: String, app_handle: AppHandle) -> Result<(), String> {
  let mut progress = FlagLyricsProgress {
    request_challenge: "Pending".to_owned(),
    solve_challenge: "Pending".to_owned(),
    flag_lyrics: "Pending".to_owned()
  };
  progress.request_challenge = "In Progress".to_owned();
  app_handle.emit_all("flag-lyrics-progress", &progress).unwrap();
  let challenge_response = lrclib::request_challenge::request().await.map_err(|err| err.to_string())?;
  progress.request_challenge = "Done".to_owned();
  progress.solve_challenge = "In Progress".to_owned();
  app_handle.emit_all("flag-lyrics-progress", &progress).unwrap();
  let nonce = lrclib::challenge_solver::solve_challenge(&challenge_response.prefix, &challenge_response.target);
  progress.solve_challenge = "Done".to_owned();
  progress.flag_lyrics = "In Progress".to_owned();
  app_handle.emit_all("flag-lyrics-progress", &progress).unwrap();
  let publish_token = format!("{}:{}", challenge_response.prefix, nonce);
  lrclib::flag::request(track_id, &flag_reason, &publish_token).await.map_err(|err| err.to_string())?;
  progress.flag_lyrics = "Done".to_owned();
  app_handle.emit_all("flag-lyrics-progress", &progress).unwrap();
  Ok(())
}


#[tauri::command]
fn play_track(track_id: i64, app_state: tauri::State<AppState>, app_handle: AppHandle) -> Result<(), String> {
  let track = app_handle.db(|db| db::get_track_by_id(track_id, db)).map_err(|err| err.to_string())?;

  let mut player_guard = app_state.player.lock().unwrap();

  if let Some(ref mut player) = *player_guard {
    player.play(track).map_err(|err| err.to_string())?;
  }

  Ok(())
}

#[tauri::command]
fn pause_track(app_state: tauri::State<AppState>) -> Result<(), String> {
  let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

  if let Some(ref mut player) = *player_guard {
    player.pause();
  }

  Ok(())
}

#[tauri::command]
fn resume_track(app_state: tauri::State<AppState>) -> Result<(), String> {
  let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

  if let Some(ref mut player) = *player_guard {
    player.resume();
  }

  Ok(())
}

#[tauri::command]
fn seek_track(position: f64, app_state: tauri::State<AppState>) -> Result<(), String> {
  let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

  if let Some(ref mut player) = *player_guard {
    player.seek(position);
  }

  Ok(())
}

#[tauri::command]
fn stop_track(app_state: tauri::State<AppState>) -> Result<(), String> {
  let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

  if let Some(ref mut player) = *player_guard {
    player.stop();
  }

  Ok(())
}

#[tauri::command]
fn set_volume(volume: f64, app_state: tauri::State<AppState>) -> Result<(), String> {
  let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

  if let Some(ref mut player) = *player_guard {
    player.set_volume(volume);
  }

  Ok(())
}

#[tauri::command]
fn open_devtools(window: tauri::Window) {
  {
    window.open_devtools();
  }
}

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .manage(AppState { db: Default::default(), player: Default::default() })
    .setup(|app| {
      let handle = app.handle();

      let app_state: State<AppState> = handle.state();
      let db = db::initialize_database(&handle).expect("Database initialize should succeed");
      *app_state.db.lock().unwrap() = Some(db);

      let player = Player::new().expect("Failed to initialize audio player");
      *app_state.player.lock().unwrap() = Some(player);

      let handle_clone = handle.clone();

      tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(40));
        loop {
          interval.tick().await;
          {
            let app_state: State<AppState> = handle_clone.state();
            let player_guard = app_state.player.lock();

            match player_guard {
              Ok(mut player_guard) => {
                if let Some(ref mut player) = *player_guard {
                  player.renew_state();

                  let emit_player_state = handle_clone.emit_all("player-state", &player);

                  if let Err(e) = emit_player_state {
                    eprintln!("Failed to emit player state: {}", e);
                  }
                }
              }
              Err(e) => eprintln!("Failed to lock player: {}", e),
            }
          }
        }
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
      get_track_ids,
      get_no_lyrics_track_ids,
      get_track,
      get_albums,
      get_album_ids,
      get_album,
      get_artists,
      get_artist_ids,
      get_artist,
      get_album_tracks,
      get_artist_tracks,
      get_album_track_ids,
      get_artist_track_ids,
      download_lyrics,
      apply_lyrics,
      retrieve_lyrics,
      retrieve_lyrics_by_id,
      search_lyrics,
      save_lyrics,
      publish_lyrics,
      flag_lyrics,
      play_track,
      pause_track,
      resume_track,
      seek_track,
      stop_track,
      set_volume,
      open_devtools
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
