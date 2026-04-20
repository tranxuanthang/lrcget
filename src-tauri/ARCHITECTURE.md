# LRCGET Tauri Architecture

## Architecture Overview

**Stack:** Tauri v2 (Rust) + SQLite + Kira audio + LRCLIB API

**Patterns:**
| Pattern | Implementation |
|---------|---------------|
| State | Global `AppState` with `Mutex<Connection>` and `Mutex<Player>` |
| DB Access | `ServiceAccess` trait on `AppHandle` for read/write operations |
| Events | Async backend→frontend via `app.emit()` |
| Commands | All FFI in `main.rs`, organized by domain |
| Scanning | Single-pass streaming with batch processing (100 files) |
| Export | Manual sidecar (.txt/.lrc) and embedded metadata export |

**Key Dependencies:** `tauri`, `rusqlite`+`rusqlite_migration`, `lofty`, `kira`, `reqwest`, `rayon`, `xxhash-rust`, `regex` (for LRC parsing)

## Project Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Entry point, Tauri commands
│   ├── state.rs             # AppState, ServiceAccess trait
│   ├── db.rs                # SQLite operations, migrations
│   ├── library.rs           # High-level library API
│   ├── scanner/             # Incremental file scanning (NEW)
│   │   ├── scan.rs          # Single-pass streaming scanner
│   │   ├── hasher.rs        # xxhash3 content hashing
│   │   ├── metadata.rs      # Audio metadata extraction
│   │   └── models.rs        # ScanResult, ScanProgress
│   ├── parser/              # File format parsers
│   │   └── lrc.rs           # LRC lyrics parser (replaces lrc crate)
│   ├── export.rs            # Manual sidecar/embed export helpers
│   ├── lyricsfile.rs        # YAML lyricsfile helpers
│   ├── player.rs            # Kira audio playback
│   ├── persistent_entities.rs # Track/Album/Artist structs
│   ├── utils.rs             # Text normalization
│   └── lrclib/              # LRCLIB API client
│       ├── search.rs        # GET /api/search
│       ├── get.rs           # GET /api/get
│       ├── get_by_id.rs     # GET /api/get/{id}
│       ├── publish.rs       # POST /api/publish (with PoW)
│       ├── flag.rs          # POST /api/flag (with PoW)
│       └── challenge_solver.rs # SHA256 proof-of-work
├── migrations/              # SQL files (v1-v9), rusqlite_migration
└── Cargo.toml / tauri.conf.json
```

## Core Components

### State Management (`state.rs`)

```rust
struct AppState {
    db: Mutex<Option<Connection>>,           // SQLite
    player: Mutex<Option<Player>>,           // Kira audio
    queued_notifications: Mutex<Vec<Notify>>,
}

trait ServiceAccess {
    fn db<F, T>(&self, f: F) -> T;      // Read-only
    fn db_mut<F, T>(&self, f: F) -> T;  // Mutable
}
```

### Database Schema (v10)

**Tables:**
| Table | Purpose |
|-------|---------|
| `directories` | Watched music paths |
| `library_data` | Init flag (single row) |
| `config_data` | Settings (embed, skip flags, theme, LRCLIB instance) |
| `artists` | name, name_lower (search) |
| `albums` | name, album_artist_name, image_path |
| `tracks` | file_path, title, duration, lrc_lyrics, txt_lyrics |
| `lyricsfiles` | Persisted YAML lyrics (decoupled from tracks). Contains track metadata (title, album, artist, duration), presence fields (`has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics`, `instrumental`), and optional LRCLIB source fields (`lrclib_instance`, `lrclib_id`). `track_id` is NULL for standalone LRCLIB lyrics without local track association. |

**Migration v8 (scanning):** Added `file_size`, `modified_time`, `content_hash`, `scan_status`

**Migration v9:** Added `lyricsfiles` table with denormalized track metadata

**Migration v10:** Added indexed lyrics-presence booleans on `tracks` (`has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics`) used for filtering

**Migration v11:** Added `volume` column to `config_data` for persisting player volume level

**Migration v12:** Added `lrclib_instance` and `lrclib_id` columns to `lyricsfiles` table for tracking remote LRCLIB lyrics sources. Enables editing lyrics from LRCLIB without requiring a local track association.

**Migration v13:** Added `has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics`, `instrumental` columns to `lyricsfiles` table. This moves the source of truth for lyrics presence to the `lyricsfiles` table, enabling filtering based on lyricsfile content for both library tracks and standalone LRCLIB lyrics.

**Migration v14:** Recreates the `tracks` table without deprecated lyrics columns (`txt_lyrics`, `lrc_lyrics`, `instrumental`, `has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics`). All lyrics data is now stored exclusively in the `lyricsfiles` table. Forces a library reset.

**Indexes:** All `*_lower` columns + `content_hash`, `scan_status`, `modified_time+file_size` (fingerprint) + lyrics-presence indexes + LRCLIB composite index (`lrclib_instance`, `lrclib_id`)

### File Scanning (`scanner/`)

**ScanResult:** `{ total_files, added, modified, deleted, moved, unchanged, is_initial_scan, duration_ms }`

**ScanProgress:** `{ phase: "discovering"|"updating", progress: f64, files_processed, files_total, message }`

**Process:**
1. Mark existing tracks as "pending" (scan_status=0)
2. Single-pass streaming: discover + process simultaneously
3. Batch size: 100 files
4. Detection modes:
   - **Hash** (default): xxhash3 of first 64KB - detects moves, 100% accurate
   - **Metadata**: mtime+size only - faster, may duplicate on metadata changes
5. Delete remaining "pending" tracks

**Performance:** 100K files HDD ~30-90s faster, SSD ~5-10s faster; Memory ~10MB vs ~200MB old

### Orphaned Lyricsfile Reattachment

When tracks are deleted from the library, their associated `lyricsfiles` records are retained (via `ON DELETE SET NULL`). When the same track is re-added during a library scan, the system automatically reattaches these orphaned lyricsfiles instead of requiring re-import from embedded metadata or sidecar files.

**Matching Criteria:**
- Exact match on normalized `track_artist_name_lower` + `track_title_lower` + `track_album_name_lower`
- Duration must be within ±2 seconds
- Only considers lyricsfiles where `track_id IS NULL` (orphaned)

**Process during `insert_new_track()`:**
1. Extract metadata from audio file
2. Query for matching orphaned lyricsfile via `db::find_orphaned_lyricsfile_tx()`
3. If found: reattach via `db::reattach_lyricsfile_to_track_tx()`, skip embedded lyrics import
4. If not found: proceed with normal embedded/sidecar lyrics import

**Benefits:**
- Preserves lyrics edits when tracks are temporarily removed and re-added
- Avoids redundant re-import from sidecar files
- Works for both previously-deleted tracks and standalone LRCLIB lyrics

### Audio Player (`player.rs`)

```rust
struct Player {
    manager: AudioManager,              // Kira
    sound_handle: Option<StreamingSoundHandle>,
    track: Option<PlayableTrack>,      // Unified type for DB and file-based tracks
    status: PlayerStatus,               // Playing/Paused/Stopped
    progress: f64, duration: f64, volume: f64,
}
```

The player uses `PlayableTrack` to support both library tracks (from database) and arbitrary tracks (from file picker). This enables playback of tracks not present in the library database.

Background loop (40ms) in `main.rs` emits `player-state` event.

Volume persistence:
- Player initializes with volume from `config_data` on startup
- `set_volume()` command updates both the player and persists to config
- Frontend receives volume updates via `player-state` events

### Export Module (`export.rs`)

Manual lyrics export to sidecar files (`.txt`, `.lrc`) and embedded metadata.

```rust
pub enum ExportFormat {
    Txt,        // Plain text sidecar file
    Lrc,        // Synced LRC sidecar file
    Embedded,   // Embedded in audio metadata (MP3/FLAC)
}

pub struct ExportResult {
    pub format: ExportFormat,
    pub path: Option<PathBuf>,
    pub success: bool,
    pub message: String,
}
```

**Key Functions:**
- `export_track()` - Export a single track to multiple formats
- `export_track_format()` - Export to a specific format
- `embed_lyrics()` - Embed lyrics into MP3 (ID3v2 USLT/SYLT) or FLAC (Vorbis comments)

**Note:** Sidecar exports overwrite existing files silently. Embedded exports use `lofty` for tag writing.

### LRC Parser (`parser/lrc.rs`)

Lightweight LRC (LyRiCs) format parser that replaces the external `lrc` crate. Supports timestamp tags with 1-3 digit precision for milliseconds.

**Problem with `lrc` crate:** The original crate only supported 1-2 digit fractional seconds (centiseconds) and silently ignored lines with 3-digit milliseconds, causing data loss when parsing LRC files like `[01:35.492] Line text`.

**Solution:** Custom parser with flexible timestamp parsing:
- `[mm:ss.x]` - 1 digit = deciseconds (×100ms)
- `[mm:ss.xx]` - 2 digits = centiseconds (×10ms)
- `[mm:ss.xxx]` - 3 digits = milliseconds (×1ms)

```rust
pub struct TimedLine {
    pub timestamp_ms: i64,
    pub text: String,
}

pub struct ParsedLrc {
    pub timed_lines: Vec<TimedLine>,
    pub id_tags: Vec<(String, String)>,
}

pub fn parse_lrc(input: &str) -> ParsedLrc;
pub fn is_instrumental_lrc(input: &str) -> bool;
pub fn format_timestamp(timestamp_ms: i64) -> String;
```

**Features:**
- Multiple timestamps per line: `[00:01.00][00:03.00] Repeated text`
- ID tags: `[ti:Title]`, `[ar:Artist]`, `[au:instrumental]`
- Automatic sorting by timestamp
- Instrumental detection via `[au:instrumental]` marker

**Used by:**
- `lyricsfile.rs` - Parsing embedded LRC lyrics during import
- `export.rs` - Converting LRC to SYLT format for MP3 embedding

### LRCLIB API (`lrclib/`)

**Endpoints:** search, get, get_by_id, publish, flag, request_challenge

**Challenge-Response (publish/flag):**
1. Request challenge → prefix + target hash
2. Solve PoW (SHA256): find nonce where hash(prefix+nonce) < target
3. Submit with token

### Data Entities (`persistent_entities.rs`)

**PersistentTrack:** id, file_path, file_name, title, album_name, artist_name, album_id, artist_id, image_path, track_number, txt_lyrics, lrc_lyrics, lyricsfile, duration, instrumental

**PlayableTrack:** A unified type for playback that works with both database tracks and arbitrary file-based tracks. Used by the `Player` to support:
- Library tracks (from database): `id` is `Some(track_id)`
- File picker tracks: `id` is `None`, metadata provided directly

Implements `From<PersistentTrack>` for seamless conversion from database entities.

**PersistentAlbum:** id, name, artist_name, tracks_count

**PersistentArtist:** id, name, tracks_count

**PersistentConfig:** skip_synced, skip_plain, show_line_count, try_embed, theme_mode, lrclib_instance, volume

## Commands

### Library
| Command | Description |
|---------|-------------|
| `scan_library(use_hash?)` | Incremental scan. Emits `scan-progress`, `scan-complete` |
| `uninitialize_library()` | Clear all library data |

### Data Queries
| Command | Returns |
|---------|---------|
| `get_track(s/ids)()` | `PersistentTrack` (includes joined `lyricsfile`) |
| `get_album(s/ids)()` | `PersistentAlbum` |
| `get_artist(s/ids)()` | `PersistentArtist` |
| `get_album_tracks/ids()` | Filtered tracks |
| `get_artist_tracks/ids()` | Filtered tracks |

### Lyrics
| Command | Purpose |
|---------|---------|
| `download_lyrics()` | Auto-download from LRCLIB |
| `retrieve_lyrics/by_id()` | Get raw LRCLIB response |
| `search_lyrics()` | Search LRCLIB database |
| `apply_lyrics()` | Save a selected LRCLIB result into database-backed lyrics storage |
| `prepare_lrclib_lyricsfile(lrclib_id)` | Get or create lyricsfile from LRCLIB. Checks local cache first, fetches from API if needed. Saves to `lyricsfiles` table with `lrclib_instance` + `lrclib_id`. Returns `lyricsfile_id` + content + `exists_in_db` flag. |
| `refresh_lrclib_lyricsfile(lrclib_id)` | Force re-download lyrics from LRCLIB API. Updates existing record in `lyricsfiles` table. Returns refreshed `lyricsfile_id` + content. |
| `save_lyrics(track_id?, lyricsfile_id?, plain?, synced?, lyricsfile?)` | Save lyrics edits. For library tracks: provide `track_id`. For standalone LRCLIB lyrics: provide `lyricsfile_id`. Prefers `lyricsfile` format. |
| `publish_lyrics(title, album, artist, duration, plain?, synced?, lyricsfile?)` | Upload to LRCLIB (with PoW; accepts Lyricsfile-only payloads) |
| `export_lyrics(track_id, formats, lyricsfile?)` | Manual export to `.txt`, `.lrc`, or embedded tags |
| `export_track_lyrics(track_id, formats)` | Export single track, returns summary for mass export |
| `get_track_ids_with_lyrics()` | Get all track IDs that have lyrics for mass export |
| `flag_lyrics()` | Report to LRCLIB (with PoW) |
| `find_matching_tracks(title, album, artist, duration?)` | Find local tracks matching LRCLIB metadata (for My LRCLIB edit flow) |
| `get_audio_metadata(filePath)` | Extract metadata from audio file (for file picker) |
| `prepare_search_query(title)` | Prepare search query by removing brackets and normalizing |

### Playback & Config
- `play_track(track_id?, file_path?, title?, album_name?, artist_name?, album_artist_name?, duration?)` - Unified playback for both library tracks (via `track_id`) and file-based tracks (via `file_path` with metadata)
- `pause/resume_track()`, `seek_track()`, `stop_track()`, `set_volume()` (persists volume to config)
- `get/set_directories()`, `get/set_config()`, `get_init()`
- Volume is loaded from config on startup and auto-saved when changed via `set_volume()`
- `open_devtools()`, `drain_notifications()`

## Events (Backend → Frontend)

| Event | Payload | Purpose |
|-------|---------|---------|
| `scan-progress` | `{ phase, progress, filesProcessed, filesTotal, message }` | Real-time scan updates |
| `scan-complete` | `{ totalFiles, added, modified, deleted, moved, unchanged, isInitialScan, durationMs }` | Scan finished |
| `player-state` | Player status | Playback updates (40ms loop) |
| `reload-track-id` | track_id | Request refresh |
| `publish-lyrics-progress` | Status | Publishing updates |
| `flag-lyrics-progress` | Status | Flagging updates |
| `export-progress` | `{ trackId, status, message }` | Export progress for single track |
| `export-complete` | `{ exported, skipped, errors }` | Mass export complete |

## Configuration

**tauri.conf.json:** Window 1024x768 min, CSP for asset protocol + media

**Cargo.toml:** Rust 2021, Tauri v2 (dialog, shell, global-shortcut, os plugins)

## Track Matching (`find_matching_tracks`)

Backend command for finding local tracks that match LRCLIB metadata. Uses `prepare_input()` normalization for case-insensitive matching.

**Algorithm:**
1. **Strong Match**: Title + Artist + Album match (via `prepare_input()` normalization) AND duration within ±2 seconds (if provided)
2. **Partial Match**: Title matches, returned if no strong matches found

**Types:**
```rust
enum MatchQuality { Strong, Partial }
struct MatchingTrack {
    #[serde(flatten)]
    track: PersistentTrack,
    match_quality: MatchQuality,
}
```

**DB Function:** `db::find_tracks_by_metadata()` searches using normalized `*_lower` columns for case-insensitive matching.

**Note:** The frontend currently uses a simpler approach with `get_tracks` + client-side filtering in `AssociateTrackModal.vue`.

## Audio Metadata Extraction (`get_audio_metadata`)

Extracts metadata from an audio file path using the existing scanner logic. Used by the file picker in the track association flow.

**Returns:**
```rust
struct AudioMetadataResponse {
    file_path: String,
    file_name: String,
    title: String,
    album: String,
    artist: String,
    album_artist: String,
    duration: f64,
    track_number: Option<u32>,
}
```

**Implementation:** Reuses `scanner::metadata::TrackMetadata::from_path()` from the scanning module.

## Search Query Preparation (`prepare_search_query`)

Prepares a search query from track title by:
1. Removing content inside `()` and `[]` brackets (including the brackets)
2. Applying `prepare_input()` normalization (lowercase, secular normalization, special char removal, whitespace collapsing)

**Example:**
- Input: `title="Love The Way You Lie (Remix) [Explicit]"`
- Output: `"love the way you lie"`

Used by `AssociateTrackModal.vue` to prefill the search input when associating LRCLIB tracks with local tracks.

## Notes

- **Lyrics Storage:** `lyricsfiles` table is the sole persistence source of truth; sidecar files and embedded tags are manual exports. The `tracks` table no longer contains `txt_lyrics` or `lrc_lyrics` columns as of migration v14.
- **Filtering Source of Truth:** Lyrics filters use `lyricsfiles.has_*_lyrics` booleans (via LEFT JOIN with COALESCE)
- **Instrumental:** `[au: instrumental]` marker in lrc_lyrics, stored as `instrumental = true` in lyricsfiles table
- **Security:** PoW for LRCLIB writes, user-agent in requests, DB in app_data_dir

## Deprecated Code Cleanup

The following deprecated functions were removed from `db.rs` (no longer used):
- `set_track_lyrics_presence()` / `set_track_lyrics_presence_tx()` - No-ops after presence fields moved to `lyricsfiles` table
- `update_track_synced_lyrics()` - Replaced by `upsert_lyricsfile_for_track()`
- `update_track_plain_lyrics()` - Replaced by `upsert_lyricsfile_for_track()`
- `update_track_null_lyrics()` - Replaced by `delete_lyricsfile_by_track_id()`
- `update_track_instrumental()` - Replaced by `upsert_lyricsfile_for_track()` with instrumental lyricsfile content
- `derive_lyrics_presence_from_legacy()` - No longer needed; presence is calculated from lyricsfile content via `lyrics_presence_from_lyricsfile()`

**Scanning Changes:**
- `insert_track_from_metadata_tx()` no longer stores `txt_lyrics`, `lrc_lyrics`, `instrumental`, or presence fields in the `tracks` table
- Embedded lyrics are now exclusively stored in the `lyricsfiles` table via `upsert_lyricsfile_for_track_tx()`
- The tracks table schema for new databases (post-reset) will not include these deprecated columns

**Migration v14:** Uses `ALTER TABLE DROP COLUMN` (SQLite 3.35.0+) to remove deprecated columns: `txt_lyrics`, `lrc_lyrics`, `instrumental`, `has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics`. Drops related indexes and forces a library reset.
