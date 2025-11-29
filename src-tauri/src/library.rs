use crate::db;
use crate::fs_track::{self, FsTrack};
use crate::persistent_entities::{PersistentAlbum, PersistentArtist, PersistentTrack};
use anyhow::Result;
use rusqlite::Connection;
use tauri::{AppHandle, Emitter, State, Manager};
use globwalk::glob;
use std::path::Path;
use std::collections::HashSet;
use rayon::prelude::*;

pub fn initialize_library(conn: &mut Connection, app_handle: AppHandle) -> Result<()> {
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

pub fn incremental_scan(
    conn: &mut Connection, 
    app_handle: AppHandle,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>
) -> Result<()> {
    use serde::Serialize;
    use std::time::Instant;

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ScanProgress {
        files_changed: usize,
        files_processed: usize,
    }

    println!("Starting incremental library scan...");
    let now = Instant::now();
    
    // Start a transaction for all database operations
    let tx = conn.transaction()?;
    
    let directories = db::get_directories(&tx)?;
    
    // Get current database state
    let db_tracks = db::get_track_metadata_map(&tx)?;
    
    // First pass: collect all file paths and categorize into new vs existing
    #[derive(Debug)]
    struct ExistingFile {
        path: std::path::PathBuf,
        id: i64,
        db_mtime: Option<i64>,
    }

    let mut disk_files = HashSet::new();
    let mut new_files = Vec::new();
    let mut existing_files = Vec::new();

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
                return Ok(());
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
            } else {
                // New file - no stat needed, will process fully later
                new_files.push((None, path));
            }
        }
    }

    // Check for cancellation after file discovery
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(());
    }

    println!("Found {} files on disk ({} new, {} existing), checking existing files for modifications...", 
             disk_files.len(), new_files.len(), existing_files.len());

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
        return Ok(());
    }

    // Check for cancellation before deletion phase
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(());
    }
    
    // Find and remove files that are in DB but not on disk
    let mut deleted_count = 0;
    for (file_path, (id, _, _, _)) in &db_tracks {
        if !disk_files.contains(file_path) {
            println!("File removed from disk, removing from DB: {}", file_path);
            db::remove_track_by_id(*id, &tx)?;
            deleted_count += 1;
        }
    }
    
    // Combine all changed files for processing
    let mut all_changed = Vec::new();
    all_changed.extend(new_files.clone());
    all_changed.extend(modified_files.clone());
    
    let total_changed = all_changed.len();
    let potential_added = new_files.len();
    let potential_modified = modified_files.len();
    
    println!("Found {} changed files (potential added: {}, potential modified: {}, deleted: {})", 
             total_changed, potential_added, potential_modified, deleted_count);

    // Track actual successes
    let mut actual_added = 0;
    let mut actual_modified = 0;

    if total_changed > 0 {
        // Process files in parallel using rayon
        let track_results: Vec<(Option<i64>, Result<FsTrack>)> = all_changed
            .par_iter()
            .filter_map(|(maybe_id, path)| {
                // Check for cancellation before starting expensive file read
                if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    return None;
                }
                
                let result = fs_track::FsTrack::new_from_path(path);
                Some((*maybe_id, result))
            })
            .collect();

        // Check for cancellation after parallel file processing
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            println!("Scan cancelled by user");
            drop(tx);  
            return Ok(());
        }

        // Write results to database sequentially and count successes
        for (idx, (maybe_id, track_result)) in track_results.iter().enumerate() {
            // Check for cancellation each iteration
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Scan cancelled by user");
                drop(tx);  
                return Ok(());
            }
            
            match track_result {
                Ok(track) => {
                    if let Some(id) = maybe_id {
                        // Update existing track by removing old entry
                        db::remove_track_by_id(*id, &tx)?;
                        actual_modified += 1;
                    } else {
                        // New track
                        actual_added += 1;
                    }
                    db::add_track(track, &tx)?;
                    
                    app_handle.emit("scan-progress", ScanProgress {
                        files_changed: total_changed,
                        files_processed: idx + 1,
                    }).unwrap();
                }
                Err(error) => {
                    let path = &all_changed[idx].1;
                    eprintln!("Failed to read file during incremental scan: {}. Error: {}", 
                             path.display(), error);
                    if let Some(id) = maybe_id {
                        db::remove_track_by_id(*id, &tx)?;
                    }
                }
            }
        }
    }

    // Check for cancellation before cleanup
    if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
        println!("Scan cancelled by user");
        drop(tx);  
        return Ok(());
    }

    // Clean up orphaned artists and albums if any tracks were deleted or modified
    if deleted_count > 0 || total_changed > 0 {
        db::clean_orphaned_entities(&tx)?;
    }

    // Commit transaction - if we got here, scan was not cancelled
    tx.commit()?;
    
    println!("==> Incremental scan took: {}ms", now.elapsed().as_millis());

    // Queue notification for display after reload
    use crate::state::{AppState, Notify, NotifyType};
    let app_state: State<AppState> = app_handle.state();
    let mut notifications = app_state.queued_notifications.lock().unwrap();
    
    let mut parts = Vec::new();
    if actual_added > 0 {
        parts.push(format!("{} added", actual_added));
    }
    if actual_modified > 0 {
        parts.push(format!("{} modified", actual_modified));
    }
    if deleted_count > 0 {
        parts.push(format!("{} deleted", deleted_count));
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

    Ok(())
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
}
