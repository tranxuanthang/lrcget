#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod db;
pub mod export;
pub mod library;
pub mod lrclib;
pub mod lyricsfile;
pub mod persistent_entities;
pub mod player;
pub mod scanner;
pub mod state;
pub mod utils;

use persistent_entities::{PersistentAlbum, PersistentArtist, PersistentConfig, PersistentTrack, PlayableTrack};
use player::Player;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use state::{AppState, Notify, NotifyType, ServiceAccess};
use tauri::{AppHandle, Emitter, Manager, State};

struct ResolvedLyricsPayload {
    plain_lyrics: String,
    synced_lyrics: String,
    is_instrumental: bool,
    provided_lyricsfile: Option<String>,
}

const LRCLIB_TRACK_NOT_FOUND: &str = "This track does not exist in LRCLIB database";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishLyricsProgress {
    request_challenge: String,
    solve_challenge: String,
    publish_lyrics: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagLyricsProgress {
    request_challenge: String,
    solve_challenge: String,
    flag_lyrics: String,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExportLyricsFormat {
    Txt,
    Lrc,
    Embedded,
}

/// Match quality for track matching results
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
enum MatchQuality {
    Strong,
    Partial,
}

/// Track matching result with quality information
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchingTrack {
    #[serde(flatten)]
    track: PersistentTrack,
    match_quality: MatchQuality,
}

/// Audio metadata extracted from a file (for file picker)
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioMetadataResponse {
    pub file_path: String,
    pub file_name: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub album_artist: String,
    pub duration: f64,
    pub track_number: Option<u32>,
}

impl From<ExportLyricsFormat> for export::ExportFormat {
    fn from(value: ExportLyricsFormat) -> Self {
        match value {
            ExportLyricsFormat::Txt => export::ExportFormat::Txt,
            ExportLyricsFormat::Lrc => export::ExportFormat::Lrc,
            ExportLyricsFormat::Embedded => export::ExportFormat::Embedded,
        }
    }
}

fn resolve_lrclib_lyrics_payload(
    lrclib_response: lrclib::get::RawResponse,
) -> Result<ResolvedLyricsPayload, String> {
    let provided_lyricsfile = lrclib_response
        .lyricsfile
        .clone()
        .filter(|content| !content.trim().is_empty());

    if let Some(lyricsfile_content) = provided_lyricsfile {
        let parsed =
            lyricsfile::parse_lyricsfile(&lyricsfile_content).map_err(|err| err.to_string())?;
        let plain_lyrics = parsed.plain_lyrics.unwrap_or_default();
        let synced_lyrics = parsed.synced_lyrics.unwrap_or_default();
        let is_instrumental = parsed.is_instrumental;

        if !is_instrumental && plain_lyrics.trim().is_empty() && synced_lyrics.trim().is_empty() {
            return Err(LRCLIB_TRACK_NOT_FOUND.to_owned());
        }

        return Ok(ResolvedLyricsPayload {
            plain_lyrics,
            synced_lyrics,
            is_instrumental,
            provided_lyricsfile: Some(lyricsfile_content),
        });
    }

    match lrclib::get::Response::from_raw_response(lrclib_response) {
        lrclib::get::Response::SyncedLyrics(synced_lyrics, plain_lyrics) => {
            Ok(ResolvedLyricsPayload {
                plain_lyrics,
                synced_lyrics,
                is_instrumental: false,
                provided_lyricsfile: None,
            })
        }
        lrclib::get::Response::UnsyncedLyrics(plain_lyrics) => Ok(ResolvedLyricsPayload {
            plain_lyrics,
            synced_lyrics: String::new(),
            is_instrumental: false,
            provided_lyricsfile: None,
        }),
        lrclib::get::Response::IsInstrumental => Ok(ResolvedLyricsPayload {
            plain_lyrics: String::new(),
            synced_lyrics: lyricsfile::INSTRUMENTAL_LRC.to_owned(),
            is_instrumental: true,
            provided_lyricsfile: None,
        }),
        lrclib::get::Response::None => Err(LRCLIB_TRACK_NOT_FOUND.to_owned()),
    }
}

#[tauri::command]
async fn get_directories(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let directories = db::get_directories(conn);
    match directories {
        Ok(directories) => Ok(directories),
        Err(error) => Err(format!(
            "Cannot get existing directories from database. Error: {}",
            error
        )),
    }
}

#[tauri::command]
async fn set_directories(
    directories: Vec<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
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
async fn set_config(
    skip_tracks_with_synced_lyrics: bool,
    skip_tracks_with_plain_lyrics: bool,
    show_line_count: bool,
    try_embed_lyrics: bool,
    theme_mode: &str,
    lrclib_instance: &str,
    volume: f64,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    db::set_config(
        skip_tracks_with_synced_lyrics,
        skip_tracks_with_plain_lyrics,
        show_line_count,
        try_embed_lyrics,
        theme_mode,
        lrclib_instance,
        volume,
        conn,
    )
    .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
async fn uninitialize_library(app_state: State<'_, AppState>) -> Result<(), String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();

    library::uninitialize_library(conn).map_err(|err| err.to_string())?;

    Ok(())
}

/// Full wipe and rescan of the library.
/// Clears tracks, albums, artists tables, deletes lyricsfiles with track associations,
/// resets init flag, and performs a full rescan.
#[tauri::command]
async fn full_scan_library(
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
    use_hash_detection: Option<bool>,
) -> Result<scanner::models::ScanResult, String> {
    // Step 1: Full wipe - clear all library data and reset init flag
    {
        let conn_guard = app_state.db.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        library::full_wipe_library(conn).map_err(|err| err.to_string())?;
    }

    // Step 2: Get directories
    let directories = {
        let conn_guard = app_state.db.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        db::get_directories(conn).map_err(|err| err.to_string())?
    };

    // Determine detection method (default to Hash for reliability)
    let detection_method = if use_hash_detection.unwrap_or(true) {
        scanner::scan::DetectionMethod::Hash
    } else {
        scanner::scan::DetectionMethod::Metadata
    };

    // Clone app_handle for use in the closure
    let app_handle_clone = app_handle.clone();

    // Step 3: Run full scan
    let scan_result = tokio::task::block_in_place(|| {
        let mut conn_guard = app_state.db.lock().unwrap();
        let conn = conn_guard.as_mut().unwrap();

        scanner::scan_library(
            &directories,
            conn,
            &|progress| {
                // Emit progress directly (synchronous)
                let _ = app_handle_clone.emit("scan-progress", progress);
            },
            detection_method,
        )
    })
    .map_err(|err| err.to_string())?;

    // Emit completion event
    let _ = app_handle.emit("scan-complete", &scan_result);

    Ok(scan_result)
}

#[tauri::command]
async fn scan_library(
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
    use_hash_detection: Option<bool>,
) -> Result<scanner::models::ScanResult, String> {
    // Get directories first (requires immutable access)
    let directories = {
        let conn_guard = app_state.db.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        db::get_directories(conn).map_err(|err| err.to_string())?
    };

    // Determine detection method (default to Hash for reliability)
    let detection_method = if use_hash_detection.unwrap_or(true) {
        scanner::scan::DetectionMethod::Hash
    } else {
        scanner::scan::DetectionMethod::Metadata
    };

    // Clone app_handle for use in the closure
    let app_handle_clone = app_handle.clone();

    // Run scan synchronously but use block_in_place to not block the runtime
    let scan_result = tokio::task::block_in_place(|| {
        let mut conn_guard = app_state.db.lock().unwrap();
        let conn = conn_guard.as_mut().unwrap();

        scanner::scan_library(
            &directories,
            conn,
            &|progress| {
                // Emit progress directly (synchronous)
                let _ = app_handle_clone.emit("scan-progress", progress);
            },
            detection_method,
        )
    })
    .map_err(|err| err.to_string())?;

    // Emit completion event
    let _ = app_handle.emit("scan-complete", &scan_result);

    Ok(scan_result)
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
    synced_lyrics_tracks: Option<bool>,
    plain_lyrics_tracks: Option<bool>,
    instrumental_tracks: Option<bool>,
    no_lyrics_tracks: Option<bool>,
    app_state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let search_query = search_query.filter(|s| !s.is_empty());
    let track_ids = library::get_track_ids(
        search_query,
        synced_lyrics_tracks.unwrap_or(true),
        plain_lyrics_tracks.unwrap_or(true),
        instrumental_tracks.unwrap_or(true),
        no_lyrics_tracks.unwrap_or(true),
        conn,
    )
    .map_err(|err| err.to_string())?;

    Ok(track_ids)
}

#[tauri::command]
async fn get_track(
    track_id: i64,
    app_state: State<'_, AppState>,
) -> Result<PersistentTrack, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let track = library::get_track(track_id, conn).map_err(|err| err.to_string())?;

    Ok(track)
}

#[tauri::command]
async fn find_matching_tracks(
    title: String,
    album_name: String,
    artist_name: String,
    duration: Option<f64>,
    app_state: State<'_, AppState>,
) -> Result<Vec<MatchingTrack>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();

    // First, try to find tracks with all criteria (strong match)
    let strong_matches = db::find_tracks_by_metadata(
        &title,
        Some(&artist_name),
        Some(&album_name),
        duration,
        conn,
    )
    .map_err(|err| err.to_string())?;

    // If we have strong matches, return them
    if !strong_matches.is_empty() {
        return Ok(strong_matches
            .into_iter()
            .map(|track| MatchingTrack {
                track,
                match_quality: MatchQuality::Strong,
            })
            .collect());
    }

    // Otherwise, search for partial matches (title only match with duration if provided)
    let partial_matches =
        db::find_tracks_by_metadata(&title, None, None, duration, conn).map_err(|err| err.to_string())?;

    // Filter out tracks where artist or album don't match at all (still partial but relevant)
    let normalized_artist = utils::prepare_input(&artist_name);
    let normalized_album = utils::prepare_input(&album_name);

    let partial_results: Vec<MatchingTrack> = partial_matches
        .into_iter()
        .map(|track| {
            let track_artist_normalized = utils::prepare_input(&track.artist_name);
            let track_album_normalized = utils::prepare_input(&track.album_name);

            // Check if artist or album matches (case-insensitive via normalization)
            let _artist_matches = track_artist_normalized == normalized_artist;
            let _album_matches = track_album_normalized == normalized_album;

            // It's a partial match if title matches and at least one of artist/album matches
            // or if we have no duration filter and just title matches
            MatchingTrack {
                track,
                match_quality: MatchQuality::Partial,
            }
        })
        .collect();

    Ok(partial_results)
}

#[tauri::command]
async fn get_audio_metadata(file_path: String) -> Result<AudioMetadataResponse, String> {
    let path = std::path::Path::new(&file_path);

    let metadata = scanner::metadata::TrackMetadata::from_path(path)
        .map_err(|err| err.to_string())?;

    Ok(AudioMetadataResponse {
        file_path: metadata.file_path,
        file_name: metadata.file_name,
        title: metadata.title,
        album: metadata.album,
        artist: metadata.artist,
        album_artist: metadata.album_artist,
        duration: metadata.duration,
        track_number: metadata.track_number,
    })
}

#[tauri::command]
async fn prepare_search_query(title: String) -> Result<String, String> {
    Ok(utils::prepare_search_input(&title))
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
async fn get_album(
    album_id: i64,
    app_state: State<'_, AppState>,
) -> Result<PersistentAlbum, String> {
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
async fn get_artist(
    artist_id: i64,
    app_state: State<'_, AppState>,
) -> Result<PersistentArtist, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let artist = library::get_artist(artist_id, conn).map_err(|err| err.to_string())?;

    Ok(artist)
}

#[tauri::command]
async fn get_album_tracks(
    album_id: i64,
    app_state: State<'_, AppState>,
) -> Result<Vec<PersistentTrack>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let tracks = library::get_album_tracks(album_id, conn).map_err(|err| err.to_string())?;

    Ok(tracks)
}

#[tauri::command]
async fn get_artist_tracks(
    artist_id: i64,
    app_state: State<'_, AppState>,
) -> Result<Vec<PersistentTrack>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let tracks = library::get_artist_tracks(artist_id, conn).map_err(|err| err.to_string())?;

    Ok(tracks)
}

#[tauri::command]
async fn get_album_track_ids(
    album_id: i64,
    without_plain_lyrics: Option<bool>,
    without_synced_lyrics: Option<bool>,
    app_state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let track_ids = library::get_album_track_ids(
        album_id,
        without_plain_lyrics.unwrap_or(false),
        without_synced_lyrics.unwrap_or(false),
        conn,
    )
    .map_err(|err| err.to_string())?;

    Ok(track_ids)
}

#[tauri::command]
async fn get_artist_track_ids(
    artist_id: i64,
    without_plain_lyrics: Option<bool>,
    without_synced_lyrics: Option<bool>,
    app_state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let track_ids = library::get_artist_track_ids(
        artist_id,
        without_plain_lyrics.unwrap_or(false),
        without_synced_lyrics.unwrap_or(false),
        conn,
    )
    .map_err(|err| err.to_string())?;

    Ok(track_ids)
}

#[tauri::command]
async fn download_lyrics(track_id: i64, app_handle: AppHandle) -> Result<String, String> {
    let track = app_handle
        .db(|db| db::get_track_by_id(track_id, db))
        .map_err(|err| err.to_string())?;
    let config = app_handle
        .db(|db| db::get_config(db))
        .map_err(|err| err.to_string())?;
    let lrclib_response = lrclib::get::request_raw(
        &track.title,
        &track.album_name,
        &track.artist_name,
        track.duration,
        &config.lrclib_instance,
    )
    .await
    .map_err(|err| err.to_string())?;
    let resolved = resolve_lrclib_lyrics_payload(lrclib_response)?;

    // Build lyricsfile content from the resolved response
    let lyricsfile_content = if let Some(ref provided) = resolved.provided_lyricsfile {
        provided.clone()
    } else {
        // Build a lyricsfile from the plain/synced lyrics
        crate::lyricsfile::build_lyricsfile(
            &crate::lyricsfile::LyricsfileTrackMetadata::new(
                &track.title,
                &track.album_name,
                &track.artist_name,
                track.duration,
            ),
            Some(&resolved.plain_lyrics),
            Some(&resolved.synced_lyrics),
        )
        .ok_or_else(|| "Failed to build lyricsfile")?
    };

    // Upsert the lyricsfile record (handles presence fields automatically)
    app_handle
        .db(|db: &Connection| {
            db::upsert_lyricsfile_for_track(
                track.id,
                &track.title,
                &track.album_name,
                &track.artist_name,
                track.duration,
                &lyricsfile_content,
                db,
            )
        })
        .map_err(|err| err.to_string())?;

    app_handle.emit("reload-track-id", track_id).unwrap();

    if resolved.is_instrumental {
        Ok("Marked track as instrumental".to_owned())
    } else if !resolved.synced_lyrics.is_empty() {
        Ok("Synced lyrics downloaded".to_owned())
    } else if !resolved.plain_lyrics.is_empty() {
        Ok("Plain lyrics downloaded".to_owned())
    } else {
        Err(LRCLIB_TRACK_NOT_FOUND.to_owned())
    }
}

#[tauri::command]
async fn apply_lyrics(
    track_id: i64,
    lrclib_response: lrclib::get::RawResponse,
    app_handle: AppHandle,
) -> Result<String, String> {
    let track = app_handle
        .db(|db| db::get_track_by_id(track_id, db))
        .map_err(|err| err.to_string())?;

    let resolved = resolve_lrclib_lyrics_payload(lrclib_response)?;

    // Build lyricsfile content from the resolved response
    let lyricsfile_content = if let Some(ref provided) = resolved.provided_lyricsfile {
        provided.clone()
    } else {
        // Build a lyricsfile from the plain/synced lyrics
        crate::lyricsfile::build_lyricsfile(
            &crate::lyricsfile::LyricsfileTrackMetadata::new(
                &track.title,
                &track.album_name,
                &track.artist_name,
                track.duration,
            ),
            Some(&resolved.plain_lyrics),
            Some(&resolved.synced_lyrics),
        )
        .ok_or_else(|| "Failed to build lyricsfile")?
    };

    // Upsert the lyricsfile record (handles presence fields automatically)
    app_handle
        .db(|db: &Connection| {
            db::upsert_lyricsfile_for_track(
                track.id,
                &track.title,
                &track.album_name,
                &track.artist_name,
                track.duration,
                &lyricsfile_content,
                db,
            )
        })
        .map_err(|err| err.to_string())?;

    std::thread::spawn(move || {
        app_handle.emit("reload-track-id", track_id).unwrap();
    });

    if resolved.is_instrumental {
        Ok("Marked track as instrumental".to_owned())
    } else if !resolved.synced_lyrics.is_empty() {
        Ok("Synced lyrics downloaded".to_owned())
    } else if !resolved.plain_lyrics.is_empty() {
        Ok("Plain lyrics downloaded".to_owned())
    } else {
        Err(LRCLIB_TRACK_NOT_FOUND.to_owned())
    }
}

#[tauri::command]
async fn retrieve_lyrics(
    title: String,
    album_name: String,
    artist_name: String,
    duration: f64,
    app_handle: AppHandle,
) -> Result<lrclib::get::RawResponse, String> {
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let response = lrclib::get::request_raw(
        &title,
        &album_name,
        &artist_name,
        duration,
        &config.lrclib_instance,
    )
    .await
    .map_err(|err| err.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn retrieve_lyrics_by_id(
    id: i64,
    app_handle: AppHandle,
) -> Result<lrclib::get_by_id::RawResponse, String> {
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let response = lrclib::get_by_id::request_raw(id, &config.lrclib_instance)
        .await
        .map_err(|err| err.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn search_lyrics(
    title: String,
    album_name: String,
    artist_name: String,
    q: String,
    app_handle: AppHandle,
) -> Result<lrclib::search::Response, String> {
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;
    let response = lrclib::search::request(
        &title,
        &album_name,
        &artist_name,
        &q,
        &config.lrclib_instance,
    )
    .await
    .map_err(|err| err.to_string())?;

    Ok(response)
}

/// Result of preparing a lyricsfile from LRCLIB
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareLyricsfileResult {
    pub lyricsfile_id: i64,
    pub lyricsfile: String,
    pub plain_lyrics: String,
    pub synced_lyrics: String,
    pub is_instrumental: bool,
    pub exists_in_db: bool,
}

#[tauri::command]
async fn prepare_lrclib_lyricsfile(
    lrclib_id: i64,
    app_handle: AppHandle,
) -> Result<PrepareLyricsfileResult, String> {
    // Get config for LRCLIB instance URL
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let lrclib_instance = config.lrclib_instance;

    // Check if we already have this lyricsfile in the database
    let existing = app_handle
        .db(|db: &Connection| {
            db::get_lyricsfile_by_lrclib(&lrclib_instance, lrclib_id, db)
        })
        .map_err(|err| err.to_string())?;

    if let Some((lyricsfile_id, lyricsfile)) = existing {
        // Already exists, parse and return
        let parsed = lyricsfile::parse_lyricsfile(&lyricsfile).map_err(|e| e.to_string())?;
        return Ok(PrepareLyricsfileResult {
            lyricsfile_id,
            lyricsfile,
            plain_lyrics: parsed.plain_lyrics.unwrap_or_default(),
            synced_lyrics: parsed.synced_lyrics.unwrap_or_default(),
            is_instrumental: parsed.is_instrumental,
            exists_in_db: true,
        });
    }

    // Fetch from LRCLIB API
    let lrclib_response = lrclib::get_by_id::request_raw(lrclib_id, &lrclib_instance)
        .await
        .map_err(|err| err.to_string())?;

    // Extract metadata from LRCLIB response
    let title = lrclib_response.name.unwrap_or_default();
    let album_name = lrclib_response.album_name.unwrap_or_default();
    let artist_name = lrclib_response.artist_name.unwrap_or_default();
    let duration = lrclib_response.duration.unwrap_or(0.0);

    // Build or use existing lyricsfile content
    let lyricsfile_content = if let Some(lyricsfile) = lrclib_response.lyricsfile {
        // LRCLIB provided a lyricsfile, use it directly
        lyricsfile
    } else {
        // Need to build lyricsfile from plain/synced lyrics
        let metadata =
            lyricsfile::LyricsfileTrackMetadata::new(&title, &album_name, &artist_name, duration);

        let plain = lrclib_response.plain_lyrics.as_deref();
        let synced = lrclib_response.synced_lyrics.as_deref();

        lyricsfile::build_lyricsfile(&metadata, plain, synced)
            .ok_or("Failed to build lyricsfile from LRCLIB response")?
    };

    // Parse for return values
    let parsed = lyricsfile::parse_lyricsfile(&lyricsfile_content).map_err(|e| e.to_string())?;

    // Save to database (track_id will be NULL)
    let lyricsfile_id = app_handle
        .db(|db: &Connection| {
            db::upsert_lyricsfile_for_lrclib(
                &lrclib_instance,
                lrclib_id,
                &title,
                &album_name,
                &artist_name,
                duration,
                &lyricsfile_content,
                db,
            )
        })
        .map_err(|err| err.to_string())?;

    Ok(PrepareLyricsfileResult {
        lyricsfile_id,
        lyricsfile: lyricsfile_content,
        plain_lyrics: parsed.plain_lyrics.unwrap_or_default(),
        synced_lyrics: parsed.synced_lyrics.unwrap_or_default(),
        is_instrumental: parsed.is_instrumental,
        exists_in_db: false,
    })
}

#[tauri::command]
async fn refresh_lrclib_lyricsfile(
    lrclib_id: i64,
    app_handle: AppHandle,
) -> Result<PrepareLyricsfileResult, String> {
    // Get config for LRCLIB instance URL
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let lrclib_instance = config.lrclib_instance;

    // Fetch fresh data from LRCLIB API (always re-download)
    let lrclib_response = lrclib::get_by_id::request_raw(lrclib_id, &lrclib_instance)
        .await
        .map_err(|err| err.to_string())?;

    // Extract metadata from LRCLIB response
    let title = lrclib_response.name.unwrap_or_default();
    let album_name = lrclib_response.album_name.unwrap_or_default();
    let artist_name = lrclib_response.artist_name.unwrap_or_default();
    let duration = lrclib_response.duration.unwrap_or(0.0);

    // Build or use existing lyricsfile content
    let lyricsfile_content = if let Some(lyricsfile) = lrclib_response.lyricsfile {
        // LRCLIB provided a lyricsfile, use it directly
        lyricsfile
    } else {
        // Need to build lyricsfile from plain/synced lyrics
        let metadata =
            lyricsfile::LyricsfileTrackMetadata::new(&title, &album_name, &artist_name, duration);

        let plain = lrclib_response.plain_lyrics.as_deref();
        let synced = lrclib_response.synced_lyrics.as_deref();

        lyricsfile::build_lyricsfile(&metadata, plain, synced)
            .ok_or("Failed to build lyricsfile from LRCLIB response")?
    };

    // Parse for return values
    let parsed = lyricsfile::parse_lyricsfile(&lyricsfile_content).map_err(|e| e.to_string())?;

    // Save to database (will update existing if lrclib_instance + lrclib_id match)
    let lyricsfile_id = app_handle
        .db(|db: &Connection| {
            db::upsert_lyricsfile_for_lrclib(
                &lrclib_instance,
                lrclib_id,
                &title,
                &album_name,
                &artist_name,
                duration,
                &lyricsfile_content,
                db,
            )
        })
        .map_err(|err| err.to_string())?;

    Ok(PrepareLyricsfileResult {
        lyricsfile_id,
        lyricsfile: lyricsfile_content,
        plain_lyrics: parsed.plain_lyrics.unwrap_or_default(),
        synced_lyrics: parsed.synced_lyrics.unwrap_or_default(),
        is_instrumental: parsed.is_instrumental,
        exists_in_db: true, // After refresh, it definitely exists
    })
}

#[tauri::command]
async fn save_lyrics(
    track_id: Option<i64>,
    lyricsfile_id: Option<i64>,
    lyricsfile: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    let lyricsfile = lyricsfile.trim();
    if lyricsfile.is_empty() {
        return Err("Lyricsfile content cannot be empty".to_string());
    }

    // Parse the lyricsfile content to validate it
    let _parsed = lyricsfile::parse_lyricsfile(lyricsfile).map_err(|err| err.to_string())?;

    // If we have a track_id, this is a library track - update the lyricsfile record
    if let Some(id) = track_id {
        let track = app_handle
            .db(|db| db::get_track_by_id(id, db))
            .map_err(|err| err.to_string())?;

        // Update or create the lyricsfile record (presence fields are set automatically)
        app_handle
            .db(|db: &Connection| {
                db::upsert_lyricsfile_for_track(
                    track.id,
                    &track.title,
                    &track.album_name,
                    &track.artist_name,
                    track.duration,
                    lyricsfile,
                    db,
                )
            })
            .map_err(|err| err.to_string())?;

        app_handle.emit("reload-track-id", id).unwrap();
    } else if let Some(id) = lyricsfile_id {
        // Standalone lyricsfile update (LRCLIB flow)
        app_handle
            .db(|db: &Connection| db::update_lyricsfile_by_id(id, lyricsfile, db))
            .map_err(|err| err.to_string())?;
    } else {
        return Err("Either track_id or lyricsfile_id must be provided".to_string());
    }

    Ok("Lyrics saved successfully".to_owned())
}

#[tauri::command]
async fn publish_lyrics(
    title: String,
    album_name: String,
    artist_name: String,
    duration: f64,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
    lyricsfile: Option<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let plain_lyrics = plain_lyrics.and_then(|lyrics| {
        let trimmed = lyrics.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    });
    let synced_lyrics = synced_lyrics.and_then(|lyrics| {
        let trimmed = lyrics.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    });
    let lyricsfile = lyricsfile.and_then(|content| {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(content)
        }
    });

    if plain_lyrics.is_none() && synced_lyrics.is_none() && lyricsfile.is_none() {
        return Err("No lyrics payload provided for publishing".to_owned());
    }

    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let mut progress = PublishLyricsProgress {
        request_challenge: "Pending".to_owned(),
        solve_challenge: "Pending".to_owned(),
        publish_lyrics: "Pending".to_owned(),
    };
    progress.request_challenge = "In Progress".to_owned();
    app_handle
        .emit("publish-lyrics-progress", &progress)
        .unwrap();
    let challenge_response = lrclib::request_challenge::request(&config.lrclib_instance)
        .await
        .map_err(|err| err.to_string())?;
    progress.request_challenge = "Done".to_owned();
    progress.solve_challenge = "In Progress".to_owned();
    app_handle
        .emit("publish-lyrics-progress", &progress)
        .unwrap();
    let nonce = lrclib::challenge_solver::solve_challenge(
        &challenge_response.prefix,
        &challenge_response.target,
    );
    progress.solve_challenge = "Done".to_owned();
    progress.publish_lyrics = "In Progress".to_owned();
    app_handle
        .emit("publish-lyrics-progress", &progress)
        .unwrap();
    let publish_token = format!("{}:{}", challenge_response.prefix, nonce);
    lrclib::publish::request(
        &title,
        &album_name,
        &artist_name,
        duration,
        plain_lyrics.as_deref(),
        synced_lyrics.as_deref(),
        lyricsfile.as_deref(),
        &publish_token,
        &config.lrclib_instance,
    )
    .await
    .map_err(|err| err.to_string())?;
    progress.publish_lyrics = "Done".to_owned();
    app_handle
        .emit("publish-lyrics-progress", &progress)
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn flag_lyrics(
    track_id: i64,
    flag_reason: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let config = app_handle
        .db(|db: &Connection| db::get_config(db))
        .map_err(|err| err.to_string())?;

    let mut progress = FlagLyricsProgress {
        request_challenge: "Pending".to_owned(),
        solve_challenge: "Pending".to_owned(),
        flag_lyrics: "Pending".to_owned(),
    };
    progress.request_challenge = "In Progress".to_owned();
    app_handle.emit("flag-lyrics-progress", &progress).unwrap();
    let challenge_response = lrclib::request_challenge::request(&config.lrclib_instance)
        .await
        .map_err(|err| err.to_string())?;
    progress.request_challenge = "Done".to_owned();
    progress.solve_challenge = "In Progress".to_owned();
    app_handle.emit("flag-lyrics-progress", &progress).unwrap();
    let nonce = lrclib::challenge_solver::solve_challenge(
        &challenge_response.prefix,
        &challenge_response.target,
    );
    progress.solve_challenge = "Done".to_owned();
    progress.flag_lyrics = "In Progress".to_owned();
    app_handle.emit("flag-lyrics-progress", &progress).unwrap();
    let publish_token = format!("{}:{}", challenge_response.prefix, nonce);
    lrclib::flag::request(
        track_id,
        &flag_reason,
        &publish_token,
        &config.lrclib_instance,
    )
    .await
    .map_err(|err| err.to_string())?;
    progress.flag_lyrics = "Done".to_owned();
    app_handle.emit("flag-lyrics-progress", &progress).unwrap();
    Ok(())
}

#[tauri::command]
async fn play_track(
    track_id: Option<i64>,
    file_path: Option<String>,
    title: Option<String>,
    album_name: Option<String>,
    artist_name: Option<String>,
    album_artist_name: Option<String>,
    duration: Option<f64>,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let playable_track: PlayableTrack = if let Some(id) = track_id {
        // Database track - fetch and convert
        let db_track = app_handle
            .db(|db| db::get_track_by_id(id, db))
            .map_err(|err| err.to_string())?;
        PlayableTrack::from(db_track)
    } else if let Some(path) = file_path {
        // File-based track - create from metadata
        let path_obj = std::path::Path::new(&path);
        let file_name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Extract metadata from file if not provided
        let (resolved_title, resolved_duration) = if title.is_none() || duration.is_none() {
            match scanner::metadata::TrackMetadata::from_path(path_obj) {
                Ok(metadata) => (
                    title.unwrap_or(metadata.title),
                    duration.unwrap_or(metadata.duration),
                ),
                Err(_) => (
                    title.unwrap_or_else(|| file_name.clone()),
                    duration.unwrap_or(0.0),
                ),
            }
        } else {
            (title.unwrap(), duration.unwrap())
        };

        PlayableTrack {
            id: None,
            file_path: path,
            file_name,
            title: resolved_title,
            album_name: album_name.unwrap_or_default(),
            artist_name: artist_name.unwrap_or_default(),
            album_artist_name,
            image_path: None,
            track_number: None,
            duration: resolved_duration,
            instrumental: false,
            lyricsfile: None,
            lyricsfile_id: None,
        }
    } else {
        return Err("Either track_id or file_path must be provided".to_string());
    };

    let mut player_guard = app_state.player.lock().unwrap();
    if let Some(ref mut player) = *player_guard {
        player.play(playable_track).map_err(|err| err.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn export_lyrics(
    track_id: i64,
    formats: Vec<ExportLyricsFormat>,
    lyricsfile: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<export::ExportResult>, String> {
    if formats.is_empty() {
        return Err("Select at least one export format".to_owned());
    }

    let track = app_handle
        .db(|db| db::get_track_by_id(track_id, db))
        .map_err(|err| err.to_string())?;

    let lyricsfile_content = lyricsfile
        .filter(|content| !content.trim().is_empty())
        .or_else(|| {
            track
                .lyricsfile
                .clone()
                .filter(|content| !content.trim().is_empty())
        })
        .ok_or_else(|| "No lyrics available for export".to_owned())?;

    let parsed =
        lyricsfile::parse_lyricsfile(&lyricsfile_content).map_err(|err| err.to_string())?;
    let export_formats = formats.into_iter().map(Into::into).collect::<Vec<_>>();

    Ok(export::export_track(&track, &parsed, &export_formats))
}

/// Detail for a single format export result
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportFormatDetail {
    pub format: String,
    pub status: export::ExportStatus,
}

/// Result summary for track export (used by mass export)
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackExportSummary {
    pub success: bool,
    pub exported: i32,
    pub skipped: i32,
    pub errors: i32,
    pub message: String,
    pub details: Vec<ExportFormatDetail>,
}

#[tauri::command]
async fn export_track_lyrics(
    track_id: i64,
    formats: Vec<ExportLyricsFormat>,
    app_handle: AppHandle,
) -> Result<TrackExportSummary, String> {
    if formats.is_empty() {
        return Ok(TrackExportSummary {
            success: true,
            exported: 0,
            skipped: 0,
            errors: 0,
            message: "No formats selected".to_owned(),
            details: vec![],
        });
    }

    let track = app_handle
        .db(|db| db::get_track_by_id(track_id, db))
        .map_err(|err| err.to_string())?;

    // Get lyrics from lyricsfile column
    let lyricsfile_content = track
        .lyricsfile
        .clone()
        .filter(|content| !content.trim().is_empty());

    if lyricsfile_content.is_none() {
        return Ok(TrackExportSummary {
            success: true,
            exported: 0,
            skipped: 1,
            errors: 0,
            message: "No lyrics available for this track".to_owned(),
            details: vec![],
        });
    }

    let parsed = lyricsfile::parse_lyricsfile(&lyricsfile_content.unwrap())
        .map_err(|err| err.to_string())?;
    let export_formats = formats.into_iter().map(Into::into).collect::<Vec<_>>();

    let results = export::export_track(&track, &parsed, &export_formats);

    // Count results based on status
    let exported = results
        .iter()
        .filter(|r| matches!(r.status, export::ExportStatus::Success))
        .count() as i32;
    let skipped = results
        .iter()
        .filter(|r| matches!(r.status, export::ExportStatus::Skipped(_)))
        .count() as i32;
    let errors = results
        .iter()
        .filter(|r| matches!(r.status, export::ExportStatus::Error(_)))
        .count() as i32;

    // Build detailed results with status info
    let details: Vec<ExportFormatDetail> = results
        .iter()
        .map(|r| ExportFormatDetail {
            format: format!("{:?}", r.format).to_lowercase(),
            status: r.status.clone(),
        })
        .collect();

    // Build message
    let message = if errors > 0 {
        let error_details: Vec<String> = results
            .iter()
            .filter(|r| matches!(r.status, export::ExportStatus::Error(_)))
            .map(|r| {
                let msg = match &r.status {
                    export::ExportStatus::Error(msg) => msg.clone(),
                    _ => String::new(),
                };
                format!("{:?}: {}", r.format, msg)
            })
            .collect();
        format!(
            "Exported {}, skipped {}, {} error(s) - {}",
            exported,
            skipped,
            errors,
            error_details.join("; ")
        )
    } else if exported > 0 {
        if skipped > 0 {
            format!("Exported {}, skipped {}", exported, skipped)
        } else {
            format!("Exported {} format(s)", exported)
        }
    } else if skipped > 0 {
        format!("Skipped {} format(s)", skipped)
    } else {
        "No formats exported".to_owned()
    };

    Ok(TrackExportSummary {
        success: errors == 0,
        exported,
        skipped,
        errors,
        message,
        details,
    })
}

#[tauri::command]
async fn get_track_ids_with_lyrics(app_state: State<'_, AppState>) -> Result<Vec<i64>, String> {
    let conn_guard = app_state.db.lock().unwrap();
    let conn = conn_guard.as_ref().unwrap();
    let track_ids = db::get_track_ids_with_lyrics(conn).map_err(|err| err.to_string())?;

    Ok(track_ids)
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
fn set_volume(
    volume: f64,
    app_state: tauri::State<AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut player_guard = app_state.player.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut player) = *player_guard {
        player.set_volume(volume);
    }
    drop(player_guard);

    // Persist volume to config
    app_handle
        .db(|db| db::set_volume_config(volume, db))
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
fn open_devtools(app_handle: AppHandle) {
    app_handle
        .get_webview_window("main")
        .unwrap()
        .open_devtools();
}

#[tauri::command]
fn drain_notifications(app_state: tauri::State<AppState>) -> Vec<Notify> {
    let mut queued_notifications = app_state.queued_notifications.lock().unwrap();
    let notifications = queued_notifications.drain(..).collect();
    notifications
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .manage(AppState {
            db: Default::default(),
            player: Default::default(),
            queued_notifications: std::sync::Mutex::new(Vec::new()),
        })
        .setup(|app| {
            let handle = app.handle();

            let app_state: State<AppState> = handle.state();
            let db = db::initialize_database(&handle).expect("Database initialize should succeed");
            *app_state.db.lock().unwrap() = Some(db);

            // Load config to get initial volume
            let initial_volume = handle
                .db(|db| db::get_config(db))
                .map(|config| config.volume)
                .unwrap_or(1.0);

            let maybe_player = Player::new(initial_volume);
            match maybe_player {
                Ok(player) => {
                    *app_state.player.lock().unwrap() = Some(player);
                }
                Err(e) => {
                    eprintln!("Failed to initialize audio player: {}", e);
                    let mut buf = app_state.queued_notifications.lock().unwrap();
                    buf.push(Notify {
                        message: format!("Failed to initialize audio player: {}", e),
                        notify_type: NotifyType::Error,
                    });
                }
            }

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

                                    let emit_player_state =
                                        handle_clone.emit("player-state", &player);

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
            uninitialize_library,
            full_scan_library,
            scan_library,
            get_tracks,
            get_track_ids,
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
            export_lyrics,
            export_track_lyrics,
            get_track_ids_with_lyrics,
            flag_lyrics,
            play_track,
            pause_track,
            resume_track,
            seek_track,
            stop_track,
            set_volume,
            open_devtools,
            drain_notifications,
            find_matching_tracks,
            get_audio_metadata,
            prepare_search_query,
            prepare_lrclib_lyricsfile,
            refresh_lrclib_lyricsfile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
