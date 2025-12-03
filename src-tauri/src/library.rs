use crate::db;
use crate::fs_track::{self, FsTrack};
use crate::persistent_entities::{PersistentAlbum, PersistentArtist, PersistentTrack};
use anyhow::Result;
use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager};
use globwalk::glob;
use std::path::Path;
use std::collections::HashSet;
use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScanMetrics {
    pub files_on_disk: usize,
    pub new_files_found: usize,
    pub existing_files_found: usize,
    pub failed_files_to_retry: usize,
    pub files_processed: usize,
    pub files_skipped: usize,
    pub tracks_added: usize,
    pub tracks_modified: usize,
    pub tracks_deleted: usize,
    pub tracks_failed: usize,
}

pub fn initialize_library<R: tauri::Runtime>(conn: &mut Connection, app_handle: AppHandle<R>) -> Result<()> {
    let init = db::get_init(conn)?;
    if init {
        return Ok(());
    }

    db::clean_library(conn)?;

    let directories = db::get_directories(conn)?;
    let result = fs_track::load_tracks_from_directories(&directories, conn, app_handle);

    match result {
        Ok(()) => {
            db::set_init(true, conn)?;
            Ok(())
        }
        Err(err) => {
            let uninitialization = uninitialize_library(conn);
            if let Err(uninit_error) = uninitialization {
                println!(
                    "Uninitialization library errored. Message: {}",
                    uninit_error.to_string()
                );
            }
            Err(err)
        }
    }
}

pub fn incremental_scan<R: tauri::Runtime>(
    conn: &mut Connection, 
    app_handle: AppHandle<R>,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>
) -> Result<ScanMetrics> {
    use std::time::Instant;

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ScanProgress {
        files_changed: usize,
        files_processed: usize,
    }

    println!("Starting incremental library scan...");
    let now = Instant::now();
    
    // Initialize metrics
    let mut metrics = ScanMetrics {
        files_on_disk: 0,
        new_files_found: 0,
        existing_files_found: 0,
        failed_files_to_retry: 0,
        files_processed: 0,
        files_skipped: 0,
        tracks_added: 0,
        tracks_modified: 0,
        tracks_deleted: 0,
        tracks_failed: 0,
    };
    
    // Start a transaction for all database operations
    let tx = conn.transaction()?;
    
    let directories = db::get_directories(&tx)?;
    
    // Get current database state
    let db_tracks = db::get_track_metadata_map(&tx)?;
    let failed_files = db::get_failed_files_map(&tx)?;
    
    // First pass: collect all file paths and categorize into new/existing/failed
    #[derive(Debug)]
    struct ExistingFile {
        path: std::path::PathBuf,
        id: i64,
        db_mtime: Option<i64>,
    }

    let mut disk_files = HashSet::new();
    let mut new_files = Vec::new();
    let mut existing_files = Vec::new();
    let mut failed_files_to_retry = Vec::new();

    for directory in &directories {
        let globwalker = glob(format!(
            "{}/**/*.{{mp3,m4a,flac,ogg,opus,wav,MP3,M4A,FLAC,OGG,OPUS,WAV}}",
            directory
        ))?;

        for item in globwalker {
            // Check for cancellation periodically during file discovery
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Scan cancelled by user");
                drop(tx); 
                return Ok(metrics);
            }
            
            let entry = item?;
            let path = entry.path().to_path_buf();
            let file_path_string = path.display().to_string();
            disk_files.insert(file_path_string.clone());

            if let Some((id, db_file_mtime, _db_lrc_mtime, _db_txt_mtime)) = db_tracks.get(&file_path_string) {
                // File exists in DB - will need to check if modified
                existing_files.push(ExistingFile {
                    path,
                    id: *id,
                    db_mtime: *db_file_mtime,
                });
            } else if let Some((_failed_id, failed_mtime, _error_type)) = failed_files.get(&file_path_string) {
                // File in failed_files table - check if mtime changed to retry
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            let disk_mtime = duration.as_secs() as i64;
                            
                            if disk_mtime != *failed_mtime {
                                failed_files_to_retry.push(path);
                            }
                            // else: same mtime, still broken, skip
                        }
                    }
                }
            } else {
                // Completely new file - no stat needed, will process fully later
                new_files.push(path);
            }
        }
    }

    // Populate discovery metrics
    metrics.files_on_disk = disk_files.len();
    metrics.new_files_found = new_files.len();
    metrics.existing_files_found = existing_files.len();
    metrics.failed_files_to_retry = failed_files_to_retry.len();

    // Check for cancellation after file discovery
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(metrics);
    }

    println!("Found {} files on disk ({} new, {} existing, {} to retry), checking existing files for modifications...", 
             metrics.files_on_disk, metrics.new_files_found, metrics.existing_files_found, metrics.failed_files_to_retry);

    // Second pass: check metadata in parallel ONLY for existing files (to detect modifications)
    let modified_files: Vec<(Option<i64>, std::path::PathBuf)> = existing_files
        .par_iter()
        .filter_map(|file| {
            // Check for cancellation during parallel processing
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                return None;
            }
            
            // Get current mtime from disk
            let disk_mtime = std::fs::metadata(&file.path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            
            // Compare with DB mtime - only include if changed
            if let Some(disk_mtime) = disk_mtime {
                if file.db_mtime.map_or(true, |db_mt| db_mt != disk_mtime) {
                    return Some((Some(file.id), file.path.clone()));
                }
            }
            None
        })
        .collect();

    // Check for cancellation after metadata collection
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(metrics);
    }

    // Check for cancellation before deletion phase
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(metrics);
    }
    
    // Find and remove files that are in DB but not on disk
    for (file_path, (id, _, _, _)) in &db_tracks {
        if !disk_files.contains(file_path) {
            println!("File removed from disk, removing from DB: {}", file_path);
            db::remove_track_by_id(*id, &tx)?;
            metrics.tracks_deleted += 1;
        }
    }
    
    // Also clean up failed_files for files that no longer exist
    for (file_path, (failed_id, _, _)) in &failed_files {
        if !disk_files.contains(file_path) {
            db::remove_failed_file_by_id(*failed_id, &tx)?;
        }
    }
    
    // Combine all changed files for processing
    let mut all_to_process = Vec::new();
    
    all_to_process.extend(new_files.iter().cloned());
    all_to_process.extend(modified_files.iter().map(|(_, path)| path.clone()));
    all_to_process.extend(failed_files_to_retry.iter().cloned());
    
    let total_to_process = all_to_process.len();
    metrics.files_skipped = metrics.existing_files_found - modified_files.len(); // Existing files we didn't need to check
    
    println!("Found {} files to process (new: {}, modified: {}, retry: {}, deleted: {})", 
             total_to_process, metrics.new_files_found, modified_files.len(), metrics.failed_files_to_retry, metrics.tracks_deleted);

    if total_to_process > 0 {
        // Process files in parallel using rayon
        let processing_results: Vec<(std::path::PathBuf, Result<fs_track::FsTrack>)> = all_to_process
            .par_iter()
            .filter_map(|path| {
                // Check for cancellation before starting expensive file read
                if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return None;
                }
                
                let result = fs_track::FsTrack::new_from_path(path);
                Some((path.clone(), result))
            })
            .collect();

        // Check for cancellation after parallel file processing
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            println!("Scan cancelled by user");
            drop(tx);  
            return Ok(metrics);
        }

        // Write results to database sequentially and count successes
        for (idx, (path, track_result)) in processing_results.iter().enumerate() {
            // Check for cancellation each iteration
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Scan cancelled by user");
                drop(tx);  
                return Ok(metrics);
            }
            
            let path_string = path.display().to_string();
            
            match track_result {
                Ok(track) => {
                    if let Some((failed_id, _, _)) = failed_files.get(&path_string) {
                        // Was in failed_files but now works
                        db::remove_failed_file_by_id(*failed_id, &tx)?;
                        metrics.tracks_added += 1;
                        metrics.files_processed += 1;
                    } else if let Some((track_id, _, _, _)) = db_tracks.get(&path_string) {
                        // Update an existing track
                        db::remove_track_by_id(*track_id, &tx)?;
                        metrics.tracks_modified += 1;
                        metrics.files_processed += 1;
                    } else {
                        // New track
                        metrics.tracks_added += 1;
                        metrics.files_processed += 1;
                    }
                    db::add_track(track, &tx)?;
                    
                    app_handle.emit("scan-progress", ScanProgress {
                        files_changed: total_to_process,
                        files_processed: idx + 1,
                    }).unwrap();
                }
                Err(error) => {
                    // Check if this is an FsTrackError we should cache
                    if let Some(fs_error) = error.downcast_ref::<fs_track::FsTrackError>() {
                        // FsTrackError - cache in failed_files
                        let error_type = fs_error.variant_name();
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        
                        // Get disk mtime
                        let disk_mtime = std::fs::metadata(path)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        
                        db::add_failed_file(&path_string, file_name, disk_mtime, error_type, &tx)?;
                        metrics.tracks_failed += 1;
                        
                        eprintln!("Failed to process file ({}): {}. Error: {}", 
                                 error_type, path.display(), error);
                    } else {
                        // non-FSTrackError errors could include network, io errors that we should not skip over retrying on next scan
                        // so don't put them in our failed_files table
                        eprintln!("Error reading file: {}. Error: {}", 
                                 path.display(), error);
                    }
                    
                    // If this was a modification attempt, remove old track entry
                    if let Some((track_id, _, _, _)) = db_tracks.get(&path_string) {
                        db::remove_track_by_id(*track_id, &tx)?;
                    }
                }
            }
        }
    }

    // Check for cancellation before cleanup
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(metrics);
    }

    // Clean up orphaned artists and albums if any tracks were deleted or modified
    if metrics.tracks_deleted > 0 || total_to_process > 0 {
        db::clean_orphaned_entities(&tx)?;
    }

    // Commit transaction - if we got here, scan was not cancelled
    tx.commit()?;
    
    println!("==> Incremental scan took: {}ms", now.elapsed().as_millis());

    // Queue notification for display after reload (skip in tests)
    if let Some(app_state) = app_handle.try_state::<crate::state::AppState>() {
        use crate::state::{Notify, NotifyType};
        let mut notifications = app_state.queued_notifications.lock().unwrap();
        
        let mut parts = Vec::new();
        if metrics.tracks_added > 0 {
            parts.push(format!("{} added", metrics.tracks_added));
        }
        if metrics.tracks_modified > 0 {
            parts.push(format!("{} modified", metrics.tracks_modified));
        }
        if metrics.tracks_deleted > 0 {
            parts.push(format!("{} deleted", metrics.tracks_deleted));
        }
        if metrics.tracks_failed > 0 {
            parts.push(format!("{} failed", metrics.tracks_failed));
        }
        
        let message = if parts.is_empty() {
            "Library refreshed: no changes detected".to_string()
        } else {
            format!("Library refreshed: {}", parts.join(", "))
        };
        
        notifications.push(Notify {
            message,
            notify_type: NotifyType::Success,
        });
    }

    Ok(metrics)
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

pub fn get_track_ids(
    search_query: Option<String>,
    synced_lyrics: bool,
    plain_lyrics: bool,
    instrumental: bool,
    no_lyrics: bool,
    conn: &Connection
) -> Result<Vec<i64>> {
    match search_query {
        Some(query) => db::get_search_track_ids(&query, synced_lyrics, plain_lyrics, instrumental, no_lyrics, conn),
        None => db::get_track_ids(synced_lyrics, plain_lyrics, instrumental, no_lyrics, conn),
    }
}

pub fn get_track(id: i64, conn: &Connection) -> Result<PersistentTrack> {
    let mut track = db::get_track_by_id(id, conn)?;
    
    // Check if lyrics files on disk are newer than cached version
    // Clone the file path to avoid borrow checker issues when reassigning track
    let file_path_string = track.file_path.clone();
    let file_path = Path::new(&file_path_string);
    let parent_path = file_path.parent().unwrap();
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
    
    // Check .lrc file
    let lrc_path = parent_path.join(format!("{}.lrc", file_stem));
    if let Ok(lrc_meta) = std::fs::metadata(&lrc_path) {
        if let Ok(modified) = lrc_meta.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                let disk_lrc_mtime = duration.as_secs() as i64;
                
                // Get current DB mtime
                let query = "SELECT lrc_mtime FROM tracks WHERE id = ?";
                let db_lrc_mtime: Option<i64> = conn.prepare(query)
                    .and_then(|mut stmt| stmt.query_row([id], |row| row.get::<_, Option<i64>>(0)))
                    .ok()
                    .flatten();
                
                // Update if: no mtime in DB, OR disk is newer, OR we don't have lyrics content
                let needs_update = db_lrc_mtime.is_none() || 
                                   db_lrc_mtime.map_or(false, |db_mt| disk_lrc_mtime > db_mt) ||
                                   track.lrc_lyrics.is_none();
                
                if needs_update {
                    if let Ok(lrc_content) = std::fs::read_to_string(&lrc_path) {
                        db::update_track_synced_lyrics(id, &lrc_content, "", disk_lrc_mtime, conn)?;
                        track = db::get_track_by_id(id, conn)?;
                    }
                }
            }
        }
    } else {
        // .lrc file doesn't exist on disk - check if DB has it and remove
        if track.lrc_lyrics.is_some() {
            let txt_path = parent_path.join(format!("{}.txt", file_stem));
            if !txt_path.exists() {
                db::update_track_null_lyrics(id, conn)?;
                track = db::get_track_by_id(id, conn)?;
            }
        }
    }
    
    // Check .txt file
    let txt_path = parent_path.join(format!("{}.txt", file_stem));
    if let Ok(txt_meta) = std::fs::metadata(&txt_path) {
        if let Ok(modified) = txt_meta.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                let disk_txt_mtime = duration.as_secs() as i64;
                
                // Get current DB mtime
                let query = "SELECT txt_mtime FROM tracks WHERE id = ?";
                let db_txt_mtime: Option<i64> = conn.prepare(query)
                    .and_then(|mut stmt| stmt.query_row([id], |row| row.get::<_, Option<i64>>(0)))
                    .ok()
                    .flatten();
                
                // Update if: no mtime in DB, OR disk is newer, OR we don't have lyrics content
                let needs_update = db_txt_mtime.is_none() || 
                                   db_txt_mtime.map_or(false, |db_mt| disk_txt_mtime > db_mt) ||
                                   track.txt_lyrics.is_none();
                
                if needs_update {
                    if let Ok(txt_content) = std::fs::read_to_string(&txt_path) {
                        db::update_track_plain_lyrics(id, &txt_content, disk_txt_mtime, conn)?;
                        track = db::get_track_by_id(id, conn)?;
                    }
                }
            }
        }
    } else {
        // .txt file doesn't exist on disk - check if DB has it and remove
        if track.txt_lyrics.is_some() && track.lrc_lyrics.is_none() {
            db::update_track_null_lyrics(id, conn)?;
            track = db::get_track_by_id(id, conn)?;
        }
    }
    
    Ok(track)
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

pub fn get_album_track_ids(album_id: i64, without_plain_lyrics: bool, without_synced_lyrics: bool, conn: &Connection) -> Result<Vec<i64>> {
    db::get_album_track_ids(album_id, without_plain_lyrics, without_synced_lyrics, conn)
}

pub fn get_artist_track_ids(artist_id: i64, without_plain_lyrics: bool, without_synced_lyrics: bool, conn: &Connection) -> Result<Vec<i64>> {
    db::get_artist_track_ids(artist_id, without_plain_lyrics, without_synced_lyrics, conn)
}

pub fn get_init(conn: &Connection) -> Result<bool> {
    db::get_init(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_audio_file(path: &Path, title: &str, artist: &str, album: &str) -> Result<()> {
        use lofty::file::{TaggedFileExt, AudioFile};
        use lofty::tag::{Accessor, Tag, TagType};
        use lofty::config::WriteOptions;
        
        // Copy the minimal MP3 file from test assets
        let minimal_mp3 = include_bytes!("../tests/assets/minimal.mp3");
        fs::write(path, minimal_mp3)?;
        
        // Open and modify the tags
        let mut tagged_file = lofty::read_from_path(path)?;
        
        // Create or get ID3v2 tag
        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                tagged_file.insert_tag(Tag::new(TagType::Id3v2));
                tagged_file.primary_tag_mut().unwrap()
            }
        };
        
        // Set the metadata
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        tag.set_album(album.to_string());
        
        // Write back to file
        tagged_file.save_to_path(path, WriteOptions::default())?;
        
        Ok(())
    }

    fn setup_test_db() -> Result<(Connection, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let mut conn = Connection::open(&db_path)?;
        
        // Initialize database schema
        db::upgrade_database_if_needed(&mut conn, 0)?;
        
        Ok((conn, temp_dir))
    }

    #[test]
    fn test_mtime_tracking_in_database() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        let music_path = temp_music.path().to_str().unwrap().to_string();
        
        db::set_directories(vec![music_path.clone()], &conn)?;
        
        // Create file
        let file_path = temp_music.path().join("song.mp3");
        create_test_audio_file(&file_path, "Test", "Artist", "Album")?;
        
        // Create a track directly using FsTrack
        let track = fs_track::FsTrack::new_from_path(&file_path)?;
        
        // Verify mtime was captured
        assert!(track.file_mtime().is_some());
        
        // Add to database
        db::add_track(&track, &conn)?;
        
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 1);
        
        // Check that mtime was stored in database
        let metadata_map = db::get_track_metadata_map(&conn)?;
        let file_path_str = file_path.to_str().unwrap();
        
        assert!(metadata_map.contains_key(file_path_str));
        let (_id, file_mtime, _lrc_mtime, _txt_mtime) = metadata_map.get(file_path_str).unwrap();
        assert!(file_mtime.is_some());
        assert_eq!(*file_mtime, track.file_mtime());
        
        Ok(())
    }

    #[test]
    fn test_lyrics_mtime_tracking() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        
        // Create audio file
        let file_path = temp_music.path().join("song.mp3");
        create_test_audio_file(&file_path, "Test", "Artist", "Album")?;
        
        // Create lyrics file
        let lrc_path = temp_music.path().join("song.lrc");
        let mut lrc_file = File::create(&lrc_path)?;
        writeln!(lrc_file, "[00:00.00]Test lyrics")?;
        drop(lrc_file);
        
        // Create track
        let track = fs_track::FsTrack::new_from_path(&file_path)?;
        
        // Verify both file and lyrics mtimes were captured
        assert!(track.file_mtime().is_some());
        assert!(track.lrc_mtime().is_some());
        assert!(track.lrc_lyrics().is_some());
        
        // Add to database
        db::add_track(&track, &conn)?;
        
        // Verify mtimes stored correctly
        let metadata_map = db::get_track_metadata_map(&conn)?;
        let file_path_str = file_path.to_str().unwrap();
        let (_id, _file_mtime, lrc_mtime, _txt_mtime) = metadata_map.get(file_path_str).unwrap();
        
        assert!(lrc_mtime.is_some());
        assert_eq!(*lrc_mtime, track.lrc_mtime());
        
        Ok(())
    }

    #[test]
    fn test_lyrics_freshness_check() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        
        // Create audio file
        let file_path = temp_music.path().join("song.mp3");
        create_test_audio_file(&file_path, "Test Song", "Test Artist", "Test Album")?;
        
        // Create initial lyrics
        let lrc_path = temp_music.path().join("song.lrc");
        let mut lrc_file = File::create(&lrc_path)?;
        writeln!(lrc_file, "[00:00.00]Old lyrics")?;
        drop(lrc_file);
        
        // Add track to database
        let track = fs_track::FsTrack::new_from_path(&file_path)?;
        db::add_track(&track, &conn)?;
        
        let tracks = db::get_tracks(&conn)?;
        let track_id = tracks[0].id;
        
        // Get track - should have old lyrics
        let track = get_track(track_id, &conn)?;
        assert!(track.lrc_lyrics.as_ref().unwrap().contains("Old lyrics"));
        
        // Wait to ensure mtime changes (filesystem has 1-second resolution on many systems)
        std::thread::sleep(std::time::Duration::from_millis(1100));
        
        // Update lyrics on disk
        let mut lrc_file = File::create(&lrc_path)?;
        writeln!(lrc_file, "[00:00.00]New lyrics")?;
        drop(lrc_file);
        
        // Get track again - should auto-detect and serve fresh lyrics
        let track = get_track(track_id, &conn)?;
        assert!(track.lrc_lyrics.as_ref().unwrap().contains("New lyrics"));
        
        Ok(())
    }

    #[test]
    fn test_lyrics_deletion_removes_from_db() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        
        // Create audio file with lyrics
        let file_path = temp_music.path().join("song.mp3");
        create_test_audio_file(&file_path, "Test", "Artist", "Album")?;
        
        let lrc_path = temp_music.path().join("song.lrc");
        let mut lrc_file = File::create(&lrc_path)?;
        writeln!(lrc_file, "[00:00.00]Test lyrics")?;
        drop(lrc_file);
        
        // Add to database
        let track = fs_track::FsTrack::new_from_path(&file_path)?;
        db::add_track(&track, &conn)?;
        
        let tracks = db::get_tracks(&conn)?;
        let track_id = tracks[0].id;
        
        // Verify lyrics exist
        let track = get_track(track_id, &conn)?;
        assert!(track.lrc_lyrics.is_some());
        
        // Delete lyrics file from disk
        fs::remove_file(&lrc_path)?;
        
        // Get track again - should detect deletion and clear from DB
        let track = get_track(track_id, &conn)?;
        assert!(track.lrc_lyrics.is_none());
        
        Ok(())
    }

    #[test]
    fn test_file_removal_detection() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        
        // Create two files
        let file1_path = temp_music.path().join("song1.mp3");
        let file2_path = temp_music.path().join("song2.mp3");
        create_test_audio_file(&file1_path, "Song 1", "Artist", "Album")?;
        create_test_audio_file(&file2_path, "Song 2", "Artist", "Album")?;
        
        // Add both to database
        let track1 = fs_track::FsTrack::new_from_path(&file1_path)?;
        let track2 = fs_track::FsTrack::new_from_path(&file2_path)?;
        db::add_track(&track1, &conn)?;
        db::add_track(&track2, &conn)?;
        
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 2);
        
        // Get track ID before deletion
        let track1_id = tracks.iter().find(|t| t.title == "Song 1").unwrap().id;
        
        // Delete file1 from disk
        fs::remove_file(&file1_path)?;
        
        // Manually check the detection logic (without full scan)
        let metadata_map = db::get_track_metadata_map(&conn)?;
        let file1_str = file1_path.to_str().unwrap();
        
        // Verify file1 is in DB
        assert!(metadata_map.contains_key(file1_str));
        
        // Verify file1 doesn't exist on disk
        assert!(!file1_path.exists());
        
        // Remove it from DB as incremental_scan would
        db::remove_track_by_id(track1_id, &conn)?;
        
        // Verify removal
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Song 2");
        
        Ok(())
    }

    #[test]
    fn test_orphaned_artists_albums_cleaned_up() -> Result<()> {
        let (conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        
        // Create files from different artists and albums
        let file1_path = temp_music.path().join("artist1_song.mp3");
        let file2_path = temp_music.path().join("artist2_song.mp3");
        create_test_audio_file(&file1_path, "Song 1", "Artist 1", "Album 1")?;
        create_test_audio_file(&file2_path, "Song 2", "Artist 2", "Album 2")?;
        
        // Add to database
        let track1 = fs_track::FsTrack::new_from_path(&file1_path)?;
        let track2 = fs_track::FsTrack::new_from_path(&file2_path)?;
        db::add_track(&track1, &conn)?;
        db::add_track(&track2, &conn)?;
        
        // Verify we have 2 artists and 2 albums
        let artists_before = db::get_artists(&conn)?;
        let albums_before = db::get_albums(&conn)?;
        assert_eq!(artists_before.len(), 2, "Should have 2 artists initially");
        assert_eq!(albums_before.len(), 2, "Should have 2 albums initially");
        
        // Verify artist/album IDs include both
        let artist_ids_before = db::get_artist_ids(&conn)?;
        let album_ids_before = db::get_album_ids(&conn)?;
        assert_eq!(artist_ids_before.len(), 2, "Should have 2 artist IDs initially");
        assert_eq!(album_ids_before.len(), 2, "Should have 2 album IDs initially");
        
        // Remove track from Artist 1
        let tracks = db::get_tracks(&conn)?;
        let track1_id = tracks.iter().find(|t| t.title == "Song 1").unwrap().id;
        db::remove_track_by_id(track1_id, &conn)?;
        
        // Before cleanup, orphaned entities still exist in raw tables
        // (get_artists/get_albums use JOIN so they won't show, but get_artist_ids doesn't)
        let artist_ids_before_cleanup = db::get_artist_ids(&conn)?;
        let album_ids_before_cleanup = db::get_album_ids(&conn)?;
        assert_eq!(artist_ids_before_cleanup.len(), 2, "Orphans still in database before cleanup");
        assert_eq!(album_ids_before_cleanup.len(), 2, "Orphans still in database before cleanup");
        
        // Clean up orphaned entities
        db::clean_orphaned_entities(&conn)?;
        
        // Verify Artist 1 and Album 1 are completely removed
        let artists_after = db::get_artists(&conn)?;
        let albums_after = db::get_albums(&conn)?;
        assert_eq!(artists_after.len(), 1, "Should have 1 artist after cleanup");
        assert_eq!(albums_after.len(), 1, "Should have 1 album after cleanup");
        assert_eq!(artists_after[0].name, "Artist 2");
        assert_eq!(albums_after[0].name, "Album 2");
        
        // Verify Artist 1 and Album 1 don't appear in ID lists either
        let artist_ids_after = db::get_artist_ids(&conn)?;
        let album_ids_after = db::get_album_ids(&conn)?;
        assert_eq!(artist_ids_after.len(), 1, "Should have 1 artist ID after cleanup");
        assert_eq!(album_ids_after.len(), 1, "Should have 1 album ID after cleanup");
        
        Ok(())
    }

    fn create_incomplete_audio_file(path: &Path, title: &str, artist: &str, album: Option<&str>) -> Result<()> {
        use lofty::file::{TaggedFileExt, AudioFile};
        use lofty::tag::{Accessor, Tag, TagType};
        use lofty::config::WriteOptions;
        
        // Copy the minimal MP3 file from test assets
        let minimal_mp3 = include_bytes!("../tests/assets/minimal.mp3");
        fs::write(path, minimal_mp3)?;
        
        // Open and modify the tags
        let mut tagged_file = lofty::read_from_path(path)?;
        
        // Create or get ID3v2 tag
        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                tagged_file.insert_tag(Tag::new(TagType::Id3v2));
                tagged_file.primary_tag_mut().unwrap()
            }
        };
        
        // Set the metadata
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        
        if let Some(album_name) = album {
            tag.set_album(album_name.to_string());
        }
        // If album is None, don't set it - will cause AlbumNotFound error
        
        // Write back to file
        tagged_file.save_to_path(path, WriteOptions::default())?;
        
        Ok(())
    }

    #[test]
    fn test_failed_file_retry_on_mtime_change() -> Result<()> {
        let (mut conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        let music_path = temp_music.path().to_str().unwrap().to_string();
        
        db::set_directories(vec![music_path.clone()], &conn)?;
        
        // Create file WITHOUT album tag (will fail AlbumNotFound)
        let file_path = temp_music.path().join("song.mp3");
        create_incomplete_audio_file(&file_path, "Test Title", "Test Artist", None)?;
        
        // Run initial full scan - should detect the file fails and add to failed_files
        db::set_init(false, &conn)?;
        let app = tauri::test::mock_app();
        initialize_library(&mut conn, app.handle().clone())?;
        db::set_init(true, &conn)?;
        
        // Verify file is in failed_files
        let failed = db::get_failed_files_map(&conn)?;
        let file_path_str = file_path.to_str().unwrap();
        assert!(failed.contains_key(file_path_str), "File should be in failed_files after initial scan");
        let (_failed_id, failed_mtime, error_type) = failed.get(file_path_str).unwrap();
        assert_eq!(error_type, "AlbumNotFound");
        
        // Verify NOT in tracks
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 0, "Failed file should not be in tracks");
        
        // Wait for mtime to change
        std::thread::sleep(std::time::Duration::from_millis(1100));
        
        // Fix the file - add album tag
        create_test_audio_file(&file_path, "Test Title", "Test Artist", "Test Album")?;
        
        // Verify mtime changed
        let new_mtime = std::fs::metadata(&file_path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        assert_ne!(new_mtime, *failed_mtime, "mtime should have changed");
        
        // Run incremental scan - should detect mtime change and retry
        let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let metrics = incremental_scan(&mut conn, app.handle().clone(), cancel_flag)?;
        
        // VERIFY: Scan detected and processed the retry
        assert_eq!(metrics.failed_files_to_retry, 1, "Should have found 1 file to retry");
        assert_eq!(metrics.files_processed, 1, "Should have processed 1 file");
        assert_eq!(metrics.tracks_added, 1, "Should have added 1 track (recovered)");
        assert_eq!(metrics.tracks_failed, 0, "Should have 0 failures (file was fixed)");
        
        // Verify NOW in tracks
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 1, "Recovered file should be in tracks");
        assert_eq!(tracks[0].title, "Test Title");
        assert_eq!(tracks[0].album_name, "Test Album");
        
        // Verify REMOVED from failed_files
        let failed = db::get_failed_files_map(&conn)?;
        assert!(!failed.contains_key(file_path_str), "File should be removed from failed_files after recovery");
        
        Ok(())
    }

    #[test]
    fn test_failed_file_not_retried_if_mtime_unchanged() -> Result<()> {
        let (mut conn, _temp_db) = setup_test_db()?;
        let temp_music = TempDir::new()?;
        let music_path = temp_music.path().to_str().unwrap().to_string();
        
        db::set_directories(vec![music_path.clone()], &conn)?;
        
        // Create file WITHOUT album tag
        let file_path = temp_music.path().join("song.mp3");
        create_incomplete_audio_file(&file_path, "Test Title", "Test Artist", None)?;
        
        // Run initial full scan - should fail and add to failed_files
        db::set_init(false, &conn)?;
        let app = tauri::test::mock_app();
        initialize_library(&mut conn, app.handle().clone())?;
        db::set_init(true, &conn)?;
        
        // Verify in failed_files
        let failed_before = db::get_failed_files_map(&conn)?;
        let file_path_str = file_path.to_str().unwrap();
        assert!(failed_before.contains_key(file_path_str));
        let (_, mtime_before, _) = failed_before.get(file_path_str).unwrap();
        
        // check that the current mtime for the file on disk has remained the same
        let current_mtime = std::fs::metadata(&file_path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        assert_eq!(current_mtime, *mtime_before, "mtime should be unchanged");
        
        // Run incremental scan - should skip retry since mtime unchanged
        let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let metrics = incremental_scan(&mut conn, app.handle().clone(), cancel_flag)?;
        
        // VERIFY: Scan actually skipped the file (didn't process it)
        assert_eq!(metrics.files_processed, 0, "Scan should not have processed any files");
        assert_eq!(metrics.failed_files_to_retry, 0, "Should have found 0 files to retry");
        
        // Verify still in failed_files (scan skipped it)
        let failed_after = db::get_failed_files_map(&conn)?;
        assert!(failed_after.contains_key(file_path_str), "File should still be in failed_files");
        let (_, mtime_after, _) = failed_after.get(file_path_str).unwrap();
        assert_eq!(*mtime_after, *mtime_before, "mtime in failed_files should be unchanged");
        
        // Verify still NOT in tracks
        let tracks = db::get_tracks(&conn)?;
        assert_eq!(tracks.len(), 0, "Failed file should still not be in tracks");
        
        Ok(())
    }
}
