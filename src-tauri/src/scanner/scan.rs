use crate::db;
use crate::db::ScanTrackInfo;
use crate::lyricsfile::{build_lyricsfile, LyricsfileTrackMetadata};
use crate::scanner::hasher::compute_quick_hash;
use crate::scanner::metadata::extract_track_info;
use crate::scanner::models::{ScanProgress, ScanResult};
use anyhow::Result;
use globwalk::glob;
use rusqlite::Connection;
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
                                eprintln!("Error inserting track {:?}: {}", path, e)
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
                                eprintln!("Error inserting track {:?}: {}", path, e)
                            }
                        }
                    }
                }
            }
        }
    }

    tx.commit()?;
    Ok(result)
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

    let lyricsfile_track_metadata = LyricsfileTrackMetadata::new(
        &metadata.title,
        &metadata.album,
        &metadata.artist,
        metadata.duration,
    );

    if let Some(lyricsfile) = build_lyricsfile(
        &lyricsfile_track_metadata,
        lyrics.txt_lyrics.as_deref(),
        lyrics.lrc_lyrics.as_deref(),
    ) {
        db::upsert_lyricsfile_for_track_tx(
            track_id,
            &metadata.title,
            &metadata.album,
            &metadata.artist,
            metadata.duration,
            &lyricsfile,
            tx,
        )?;
    }

    Ok(())
}

/// DEPRECATED: Use single-pass scan_library instead
///
/// This function performs a full directory traversal just to count files,
/// which is inefficient for large libraries. The new scan_library() uses
/// a single-pass approach that discovers and processes files simultaneously,
/// emitting "Found N files so far..." progress updates instead of requiring
/// a pre-scan count.
///
/// Kept for backward compatibility but should not be used for new code.
#[deprecated(
    since = "0.9.0",
    note = "Use scan_library() instead - it performs single-pass streaming with real-time progress"
)]
pub fn estimate_file_count(directories: &[String]) -> Result<usize> {
    let mut count = 0;
    for directory in directories {
        let pattern = format!(
            "{}/**/*.{{mp3,m4a,flac,ogg,opus,wav,wma,aac,aiff,ape,mpc,wv}}",
            directory
        );
        count += glob(&pattern)?.count();
    }
    Ok(count)
}
