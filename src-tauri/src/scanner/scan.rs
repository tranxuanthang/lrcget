use crate::db;
use crate::db::ScanTrackInfo;
use crate::lyricsfile::{build_lyricsfile, LyricsfileTrackMetadata, INSTRUMENTAL_LRC};
use crate::parser::lrc::parse_lrc;
use crate::scanner::hasher::compute_quick_hash;
use crate::scanner::metadata::extract_track_info;
use crate::scanner::models::{ScanProgress, ScanResult};
use anyhow::Result;
use globwalk::glob;
use rusqlite::{Connection, OptionalExtension};
use std::time::{Instant, SystemTime};

const BATCH_SIZE: usize = 100;

/// Method to detect file changes during scan
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionMethod {
    /// Use content hash (default) - slower but handles all edge cases including:
    /// - File moves with modified metadata
    /// - Cross-platform timestamp differences
    /// - Filesystems with different timestamp precision
    Hash,
    /// Use file metadata (mtime + size) - faster but with limitations:
    /// - May create duplicates if files are moved with different metadata
    /// - Best for: Large libraries on single filesystem that rarely move
    Metadata,
}

impl Default for DetectionMethod {
    fn default() -> Self {
        DetectionMethod::Hash
    }
}

/// Single-pass streaming scan - discovers and processes files simultaneously
///
/// This approach eliminates the double-traversal problem by processing files
/// in a single pass and emitting progress updates showing processed count.
pub fn scan_library(
    directories: &[String],
    conn: &mut Connection,
    progress_callback: &dyn Fn(ScanProgress),
    detection_method: DetectionMethod,
) -> Result<ScanResult> {
    let start_time = Instant::now();
    let is_initial_scan = !db::get_init(conn)?;

    // Phase 1: Mark all tracks as pending
    db::mark_all_tracks_pending(conn)?;

    let mut total_files = 0;
    let mut processed_files = 0;
    let mut added = 0;
    let mut moved = 0;
    let mut unchanged = 0;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    // Phase 2: Stream through files with globwalk - discover AND process in single pass
    for directory in directories {
        let pattern = format!(
            "{}/**/*.{{mp3,m4a,flac,ogg,opus,wav,MP3,M4A,FLAC,OGG,OPUS,WAV}}",
            directory
        );

        for item in glob(&pattern)? {
            match item {
                Ok(entry) => {
                    batch.push(entry);
                    total_files += 1;

                    if batch.len() >= BATCH_SIZE {
                        let batch_result = process_batch(&batch, conn, detection_method)?;
                        added += batch_result.added;
                        moved += batch_result.moved;
                        unchanged += batch_result.unchanged;
                        processed_files += batch.len();

                        // Emit progress after processing each batch
                        progress_callback(ScanProgress::processing(processed_files, total_files));

                        batch.clear();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading entry: {}", e);
                }
            }
        }
    }

    // Process remaining files in batch
    if !batch.is_empty() {
        let batch_result = process_batch(&batch, conn, detection_method)?;
        added += batch_result.added;
        moved += batch_result.moved;
        unchanged += batch_result.unchanged;
        processed_files += batch.len();

        // Emit final progress
        progress_callback(ScanProgress::processing(processed_files, total_files));
    }

    // Phase 3: Delete tracks that weren't processed (deleted files)
    progress_callback(ScanProgress::updating());
    let deleted = db::delete_unprocessed_tracks(conn)?;

    // Mark as initialized after first successful scan
    if is_initial_scan {
        db::set_init(true, conn)?;
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(ScanResult {
        total_files,
        added,
        modified: 0,
        deleted,
        moved,
        unchanged,
        is_initial_scan,
        duration_ms,
    })
}

#[derive(Default)]
struct BatchResult {
    added: usize,
    moved: usize,
    unchanged: usize,
}

fn process_batch(
    batch: &[globwalk::DirEntry],
    conn: &mut Connection,
    detection_method: DetectionMethod,
) -> Result<BatchResult> {
    let mut result = BatchResult::default();
    let tx = conn.transaction()?;

    for entry in batch {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error getting metadata for {:?}: {}", path, e);
                continue;
            }
        };

        let file_size = metadata.len() as i64;
        let modified_time = metadata
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        let path_str = path.to_string_lossy().to_string();

        match detection_method {
            DetectionMethod::Hash => {
                // Hash-based detection (default)
                let hash = match compute_quick_hash(path) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("Error hashing {:?}: {}", path, e);
                        continue;
                    }
                };

                match db::find_track_by_hash_tx(&hash, &tx)? {
                    Some(ScanTrackInfo { id, file_path }) => {
                        if file_path == path_str {
                            // Same path, same hash - unchanged
                            db::mark_track_processed_tx(id, &tx)?;
                            result.unchanged += 1;
                        } else {
                            // Different path, same hash - moved!
                            db::update_track_path_and_fingerprint_tx(
                                id,
                                &path_str,
                                file_size,
                                modified_time,
                                &hash,
                                &tx,
                            )?;
                            result.moved += 1;
                        }
                    }
                    None => {
                        // No match found - new file
                        match insert_new_track(path, file_size, modified_time, &hash, &tx) {
                            Ok(_) => result.added += 1,
                            Err(e) => {
                                eprintln!("Error inserting track {:?}: {}", path, e);
                                continue;
                            }
                        }
                    }
                }
            }
            DetectionMethod::Metadata => {
                // Metadata-based detection (mtime + size) - FAST but less accurate
                match db::find_track_by_fingerprint_tx(modified_time, file_size, &tx)? {
                    Some(ScanTrackInfo { id, file_path }) => {
                        if file_path == path_str {
                            // Same path, same fingerprint - unchanged
                            db::mark_track_processed_tx(id, &tx)?;
                            result.unchanged += 1;
                        } else {
                            // Different path, same fingerprint - moved!
                            db::update_track_path_tx(id, &path_str, &tx)?;
                            result.moved += 1;
                        }
                    }
                    None => {
                        // No fingerprint match - treat as new file
                        let hash = match compute_quick_hash(path) {
                            Ok(h) => h,
                            Err(e) => {
                                eprintln!("Error hashing {:?}: {}", path, e);
                                continue;
                            }
                        };
                        match insert_new_track(path, file_size, modified_time, &hash, &tx) {
                            Ok(_) => result.added += 1,
                            Err(e) => {
                                eprintln!("Error inserting track {:?}: {}", path, e);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        import_matching_sidecar(path, &tx)?;
    }

    tx.commit()?;
    Ok(result)
}

/// Refresh external lyrics even when the audio fingerprint has not changed.
/// Saved synced/instrumental lyrics and existing plain edits remain protected,
/// except that a valid matching LRC can upgrade a plain record to synced.
fn import_matching_sidecar(path: &std::path::Path, tx: &rusqlite::Transaction) -> Result<()> {
    let state: Option<(i64, bool, bool)> = tx
        .query_row(
            "SELECT tracks.id,
         COALESCE(lyricsfiles.has_synced_lyrics, 0) OR COALESCE(lyricsfiles.instrumental, 0),
         COALESCE(lyricsfiles.has_plain_lyrics, 0)
         FROM tracks LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
         WHERE tracks.file_path = ? LIMIT 1",
            [path.to_string_lossy().as_ref()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((track_id, protected, has_plain)) = state else {
        return Ok(());
    };
    if protected {
        return Ok(());
    }
    let lrc = std::fs::read_to_string(path.with_extension("lrc"))
        .ok()
        .filter(|text| valid_lrc(text));
    // TXT is a fallback for missing lyrics, never an overwrite of saved plain edits.
    let txt = if lrc.is_none() && !has_plain {
        std::fs::read_to_string(path.with_extension("txt")).ok()
    } else {
        None
    };
    if lrc.is_none() && txt.is_none() {
        return Ok(());
    }
    let track = db::get_track_by_id(track_id, tx)?;
    if let Some(content) = build_lyricsfile(
        &LyricsfileTrackMetadata::from_persistent_track(&track),
        txt.as_deref(),
        lrc.as_deref(),
    ) {
        db::upsert_lyricsfile_for_track_tx(
            track.id,
            &track.title,
            &track.album_name,
            &track.artist_name,
            track.duration,
            &content,
            tx,
        )?;
    }
    Ok(())
}

fn valid_lrc(text: &str) -> bool {
    text.trim() == INSTRUMENTAL_LRC
        || parse_lrc(text)
            .timed_lines
            .iter()
            .any(|line| !line.text.trim().is_empty())
}

/// Helper to insert a new track with metadata extraction
fn insert_new_track(
    path: &std::path::Path,
    file_size: i64,
    modified_time: i64,
    content_hash: &str,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    // Extract metadata and lyrics
    let (metadata, lyrics) = extract_track_info(path)?;

    // Get or create artist
    let artist_id = match db::find_artist_tx(&metadata.artist, tx) {
        Ok(id) => id,
        Err(_) => db::add_artist_tx(&metadata.artist, tx)?,
    };

    // Get or create album
    let album_id = match db::find_album_tx(&metadata.album, &metadata.album_artist, tx) {
        Ok(id) => id,
        Err(_) => db::add_album_tx(&metadata.album, &metadata.album_artist, tx)?,
    };

    // Insert track
    let track_id = db::insert_track_from_metadata_tx(
        &metadata,
        &lyrics,
        file_size,
        modified_time,
        content_hash,
        artist_id,
        album_id,
        tx,
    )?;

    // Sync FTS index
    db::insert_track_fts_tx(
        track_id,
        &crate::utils::prepare_input(&metadata.title),
        &crate::utils::prepare_input(&metadata.artist),
        &crate::utils::prepare_input(&metadata.album),
        tx,
    )?;

    // Check for orphaned lyricsfile before importing embedded lyrics
    let orphaned_lyricsfile = db::find_orphaned_lyricsfile_tx(
        &metadata.title,
        &metadata.artist,
        &metadata.album,
        metadata.duration,
        tx,
    )?;

    if let Some(lyricsfile_id) = orphaned_lyricsfile {
        // Reattach orphaned lyricsfile to this track
        db::reattach_lyricsfile_to_track_tx(lyricsfile_id, track_id, tx)?;
    }

    Ok(())
}

#[cfg(test)]
mod sidecar_tests {
    use super::*;
    use include_dir::{include_dir, Dir};
    use lofty::config::WriteOptions;
    use lofty::prelude::{Accessor, TagExt, TaggedFileExt};
    use rusqlite_migration::Migrations;

    fn add_metadata(path: &std::path::Path, title: &str) {
        let mut file = lofty::read_from_path(path).unwrap();
        let tag = file.primary_tag_mut().unwrap();
        tag.set_title(title.into());
        tag.set_artist("test artist".into());
        tag.set_album("test album".into());
        tag.save_to_path(path, WriteOptions::default()).unwrap();
    }

    fn test_database() -> Connection {
        static MIGRATIONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");
        let mut conn = Connection::open_in_memory().unwrap();
        Migrations::from_directory(&MIGRATIONS)
            .unwrap()
            .to_latest(&mut conn)
            .unwrap();
        conn
    }

    #[test]
    fn matching_lrc_upgrades_plain_after_restart_and_preserves_synced_edits() {
        for method in [DetectionMethod::Hash, DetectionMethod::Metadata] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("song.mp3");
            std::fs::write(&path, include_bytes!("../../tests/fixtures/embedding.mp3")).unwrap();
            add_metadata(&path, "song");
            let audio = std::fs::read(&path).unwrap();
            std::fs::write(path.with_extension("txt"), "original plain words").unwrap();
            let directories = vec![directory.path().to_string_lossy().into_owned()];
            let mut conn = test_database();
            scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
            let original = db::get_tracks(&conn).unwrap().remove(0);
            let parsed =
                crate::lyricsfile::parse_lyricsfile(original.lyricsfile.as_deref().unwrap())
                    .unwrap();
            assert!(parsed.plain_lyrics.is_some());
            assert!(parsed.synced_lyrics.is_none());

            // An external app creates an LRC while LRCGET is closed.
            let database_path = directory.path().join("restart.sqlite3");
            conn.execute("VACUUM INTO ?", [database_path.to_str().unwrap()])
                .unwrap();
            drop(conn);
            let timed = "[00:01.00]external synced words\n[00:02.00]second line\n";
            std::fs::write(path.with_extension("lrc"), timed).unwrap();
            let mut conn = Connection::open(&database_path).unwrap();
            let result = scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
            assert_eq!(result.unchanged, 1);
            let upgraded = db::get_track_by_id(original.id, &conn).unwrap();
            let parsed =
                crate::lyricsfile::parse_lyricsfile(upgraded.lyricsfile.as_deref().unwrap())
                    .unwrap();
            assert!(parsed
                .synced_lyrics
                .as_deref()
                .unwrap()
                .contains("external synced words"));
            assert!(!parsed
                .plain_lyrics
                .as_deref()
                .unwrap()
                .contains("original plain words"));
            assert_eq!(std::fs::read(&path).unwrap(), audio);
            assert_eq!(
                std::fs::read_to_string(path.with_extension("txt")).unwrap(),
                "original plain words"
            );
            assert_eq!(
                std::fs::read_to_string(path.with_extension("lrc")).unwrap(),
                timed
            );

            let saved = build_lyricsfile(
                &LyricsfileTrackMetadata::from_persistent_track(&upgraded),
                None,
                Some("[00:03.00]protected editor words\n"),
            )
            .unwrap();
            let tx = conn.transaction().unwrap();
            db::upsert_lyricsfile_for_track_tx(
                upgraded.id,
                &upgraded.title,
                &upgraded.album_name,
                &upgraded.artist_name,
                upgraded.duration,
                &saved,
                &tx,
            )
            .unwrap();
            tx.commit().unwrap();
            std::fs::write(
                path.with_extension("lrc"),
                "[00:04.00]conflicting external words\n",
            )
            .unwrap();
            scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
            let protected = db::get_track_by_id(original.id, &conn).unwrap();
            assert_eq!(protected.lyricsfile.as_deref(), Some(saved.as_str()));
        }
    }

    #[test]
    fn missing_lyrics_use_txt_fallback_until_valid_lrc_appears() {
        for method in [DetectionMethod::Hash, DetectionMethod::Metadata] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("song.mp3");
            std::fs::write(&path, include_bytes!("../../tests/fixtures/embedding.mp3")).unwrap();
            add_metadata(&path, "song");
            let directories = vec![directory.path().to_string_lossy().into_owned()];
            let mut conn = test_database();
            scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
            let id = db::get_tracks(&conn).unwrap().remove(0).id;
            assert!(db::get_track_by_id(id, &conn).unwrap().lyricsfile.is_none());
            std::fs::write(path.with_extension("txt"), "fallback words").unwrap();
            for invalid in ["untimed words", "[ti:metadata only]", "[00:01.00]\n"] {
                std::fs::write(path.with_extension("lrc"), invalid).unwrap();
                scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
                let plain = db::get_track_by_id(id, &conn).unwrap();
                let parsed =
                    crate::lyricsfile::parse_lyricsfile(plain.lyricsfile.as_deref().unwrap())
                        .unwrap();
                assert_eq!(parsed.plain_lyrics.as_deref(), Some("fallback words"));
                assert!(parsed.synced_lyrics.is_none());
            }
            std::fs::write(
                path.with_extension("lrc"),
                "[00:01.00]preferred LRC words\n",
            )
            .unwrap();
            scan_library(&directories, &mut conn, &|_| {}, method).unwrap();
            let upgraded = db::get_track_by_id(id, &conn).unwrap();
            let parsed =
                crate::lyricsfile::parse_lyricsfile(upgraded.lyricsfile.as_deref().unwrap())
                    .unwrap();
            assert!(parsed
                .synced_lyrics
                .unwrap()
                .contains("preferred LRC words"));
        }
    }
}
