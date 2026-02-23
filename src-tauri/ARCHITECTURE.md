# LRCGET Tauri Architecture

## Overview

LRCGET is a desktop application for mass-downloading LRC (LRC format) synced lyrics for offline music libraries. It's built with Tauri v2 (Rust backend + Web frontend).

**Key Features:**
- Scan local music directories (MP3, FLAC, M4A, OGG, OPUS, WAV)
- Download lyrics from LRCLIB API
- Embed lyrics into audio files
- Play tracks with built-in audio player
- Publish/flag lyrics to LRCLIB

## Directory Structure

```
src-tauri/
├── src/
│   ├── main.rs                 # Entry point, Tauri command handlers
│   ├── state.rs                # Global application state management
│   ├── db.rs                   # SQLite database operations
│   ├── library.rs              # High-level library management
│   ├── scanner/                # File system scanning (NEW)
│   │   ├── mod.rs              # Module exports
│   │   ├── scan.rs             # Main scan implementation with incremental updates
│   │   ├── hasher.rs           # Content hashing (xxhash3)
│   │   ├── metadata.rs         # Audio metadata extraction
│   │   └── models.rs           # ScanResult, ScanProgress types
│   ├── fs_track.rs             # DEPRECATED: Old file system scanning
│   ├── lyrics.rs               # Lyrics download, save, and embedding
│   ├── player.rs               # Audio playback using Kira
│   ├── persistent_entities.rs  # Data structures for DB entities
│   ├── utils.rs                # Utility functions (text processing)
│   └── lrclib/                 # LRCLIB API client module
│       ├── mod.rs              # Module exports
│       ├── search.rs           # Search lyrics by metadata
│       ├── get.rs              # Get lyrics by track metadata
│       ├── get_by_id.rs        # Get lyrics by track ID
│       ├── publish.rs          # Publish lyrics to LRCLIB
│       ├── flag.rs             # Flag incorrect lyrics
│       ├── request_challenge.rs # Request proof-of-work challenge
│       └── challenge_solver.rs # Proof-of-work challenge solver
├── Cargo.toml                  # Rust dependencies
├── tauri.conf.json            # Tauri configuration
├── build.rs                   # Build script
└── capabilities/              # Tauri capabilities
```

## Core Components

### 1. Application State (`state.rs`)

**`AppState`** - Thread-safe global state containing:
- `db: Mutex<Option<Connection>>` - SQLite connection
- `player: Mutex<Option<Player>>` - Audio player instance
- `queued_notifications: Mutex<Vec<Notify>>` - Notification queue

**`ServiceAccess` trait** - Provides database access from `AppHandle`:
```rust
fn db<F, TResult>(&self, operation: F) -> TResult  // Read-only access
fn db_mut<F, TResult>(&self, operation: F) -> TResult  // Mutable access
```

### 2. Database Layer (`db.rs`)

SQLite database with automatic migrations (current version: 7).

**Key Tables:**
- `directories` - Watched music directories
- `library_data` - Library initialization status
- `config_data` - Application settings
- `artists` - Music artists
- `albums` - Music albums
- `tracks` - Individual music tracks with lyrics

**Migration History:**
- v1: Initial schema (artists, albums, tracks)
- v2: Added txt_lyrics, indexes
- v3: Added instrumental flag
- v4: Added lowercase columns for searching
- v5: Added track_number, album_artist_name, theme_mode, lrclib_instance
- v6: Split skip_not_needed_tracks into synced/plain variants
- v7: Added show_line_count setting
- v8: Added incremental scanning columns (file_size, modified_time, content_hash, scan_status)

**Key Functions:**
- `initialize_database()` - Creates/opens DB file in app_data_dir
- `upgrade_database_if_needed()` - Runs migrations
- CRUD operations for all entities

### 3. File System Scanning (NEW: `scanner/` module)

**Migration from `fs_track.rs` to `scanner/`:**
- `fs_track.rs` is **DEPRECATED** and will be removed
- New `scanner/` module provides incremental/partial scanning
- Single-pass streaming (no double traversal)
- Move detection via content hashing
- Two detection methods: Hash (default) or Metadata

**`scanner/models.rs`**:
```rust
struct ScanResult {
    total_files: usize,
    added: usize,
    modified: usize,
    deleted: usize,
    moved: usize,
    unchanged: usize,
    is_initial_scan: bool,
    duration_ms: u64,
}

struct ScanProgress {
    phase: String,        // "discovering" | "updating"
    progress: f64,        // 0.0 to 1.0
    files_processed: usize,
    files_total: Option<usize>,  // None during discovery phase
    message: String,      // Human-readable status
}
```

**Scanning Process (`scanner/scan.rs`):**
1. Mark all existing tracks as "pending" in DB
2. **Single-pass streaming**: Discover and process simultaneously
3. Processes in batches of 100 (hash detection by default)
4. Emits `scan-progress` after each batch showing processed count
5. Shows "Processing files: X/Y" as progress
6. Detects moves via content hash matching
7. Deletes tracks still marked "pending" (removed files)

**Detection Methods:**
- **Hash (default)**: Computes xxhash3 of first 64KB
  - Detects all file moves/renames
  - Slower but 100% accurate
- **Metadata**: Uses mtime + size only
  - Very fast (just stat() call)
  - May create duplicates if metadata changes during moves

**Database Columns for Incremental Scanning (v8):**
```sql
ALTER TABLE tracks ADD COLUMN file_size INTEGER;
ALTER TABLE tracks ADD COLUMN modified_time INTEGER;
ALTER TABLE tracks ADD COLUMN content_hash TEXT;
ALTER TABLE tracks ADD COLUMN scan_status INTEGER DEFAULT 1;
```

**Performance:**
- 100K files on HDD: ~30-90 seconds faster (no double traversal)
- 100K files on SSD: ~5-10 seconds faster
- Memory usage: ~10MB (batch processing) vs ~200MB (old approach)

### 4. Legacy File System Scanning (`fs_track.rs`) - DEPRECATED

**Status**: Deprecated, kept for backward compatibility only

**`FsTrack`** struct - Represents a file-system track:
```rust
struct FsTrack {
    file_path: String,
    file_name: String,
    title: String,
    album: String,
    artist: String,
    album_artist: String,
    duration: f64,
    txt_lyrics: Option<String>,
    lrc_lyrics: Option<String>,
    track_number: Option<u32>,
}
```

**Issues with old approach:**
- Double directory traversal (count + process)
- No move detection (creates duplicates on renames)
- Full memory load of all paths
- Emits `initialize-progress` with old format

**Migration Path:**
- Frontend now uses `scan_library` command
- Listens to `scan-progress` and `scan-complete` events
- Old `initialize_library` and `refresh_library` commands still exist but unused

### 5. Library Management (`library.rs`)

High-level API for music library operations:
- `initialize_library()` - Scan directories and populate DB
- `uninitialize_library()` - Clear all library data
- `get_tracks()`, `get_albums()`, `get_artists()` - Retrieve entities
- Various filter/search functions for tracks

### 5. Lyrics Operations (`lyrics.rs`)

**Lyrics Sources:**
- Download from LRCLIB API
- Load from local .txt/.lrc files
- Manual editing

**Lyrics Storage Options:**
1. **Sidecar files** (default): Creates `trackname.txt` and `trackname.lrc`
2. **Embedded** (optional): Embeds in MP3 (ID3v2 SYLT/USLT) or FLAC (Vorbis comments)

**Lyrics Types:**
- Plain text (.txt files)
- Synchronized LRC format (.lrc files with timestamps)
- Instrumental marker: `[au: instrumental]`

### 6. Audio Player (`player.rs`)

Uses **Kira** audio library for playback.

**`Player`** struct:
```rust
struct Player {
    manager: AudioManager,           // Kira audio manager
    sound_handle: Option<StreamingSoundHandle>,
    track: Option<PersistentTrack>,  // Currently playing track
    status: PlayerStatus,            // Playing/Paused/Stopped
    progress: f64,                   // Current position in seconds
    duration: f64,                   // Track duration
    volume: f64,                     // 0.0 - 1.0
}
```

**Background Loop:** (40ms interval in `main.rs`)
- Updates player state
- Emits `player-state` event to frontend

### 7. LRCLIB API Client (`lrclib/`)

Client for the LRCLIB lyrics database API.

**API Endpoints:**

| Module | Endpoint | Purpose |
|--------|----------|---------|
| `search.rs` | `GET /api/search` | Search lyrics by metadata |
| `get.rs` | `GET /api/get` | Get lyrics by track info |
| `get_by_id.rs` | `GET /api/get/{id}` | Get lyrics by track ID |
| `publish.rs` | `POST /api/publish` | Upload new lyrics |
| `flag.rs` | `POST /api/flag` | Report incorrect lyrics |
| `request_challenge.rs` | `POST /api/request-challenge` | Get PoW challenge |

**Challenge-Response Flow** (for publish/flag):
1. Request challenge (gets prefix + target hash)
2. Solve proof-of-work (find nonce where SHA256(prefix+nonce) < target)
3. Include token in publish/flag request

**`challenge_solver.rs`** - Brute-force PoW solver using SHA256

### 8. Data Entities (`persistent_entities.rs`)

**`PersistentTrack`**:
```rust
struct PersistentTrack {
    id: i64,
    file_path: String,
    file_name: String,
    title: String,
    album_name: String,
    artist_name: String,
    album_id: i64,
    artist_id: i64,
    image_path: Option<String>,
    track_number: Option<i64>,
    txt_lyrics: Option<String>,
    lrc_lyrics: Option<String>,
    duration: f64,
    instrumental: bool,
}
```

**`PersistentAlbum`**: id, name, artist_name, tracks_count

**`PersistentArtist`**: id, name, tracks_count

**`PersistentConfig`**: skip_tracks_with_synced_lyrics, skip_tracks_with_plain_lyrics, show_line_count, try_embed_lyrics, theme_mode, lrclib_instance

### 9. Utilities (`utils.rs`)

- `prepare_input()` - Normalize strings for searching (lowercase, remove accents, collapse spaces)
- `strip_timestamp()` - Remove LRC timestamps to get plain lyrics

## Tauri Commands

All commands defined in `main.rs` and exposed to frontend:

### Configuration
- `get_directories()` / `set_directories()` - Music directories
- `get_config()` / `set_config()` - Application settings
- `get_init()` - Check if library is initialized

### Library Management
- `scan_library(use_hash_detection?: boolean)` - **NEW**: Incremental scan with real-time progress
  - Single-pass streaming (no double traversal)
  - Detects file moves via content hashing
  - Emits `scan-progress` and `scan-complete` events
  - Default: Hash detection for accuracy, pass `false` for Metadata mode (faster)
- `initialize_library()` - **DEPRECATED**: Use `scan_library` instead
- `uninitialize_library()` - Clear library
- `refresh_library()` - **DEPRECATED**: Use `scan_library` instead

### Data Queries
- `get_tracks()`, `get_track_ids()`, `get_track()` - Track retrieval
- `get_albums()`, `get_album_ids()`, `get_album()` - Album retrieval
- `get_artists()`, `get_artist_ids()`, `get_artist()` - Artist retrieval
- `get_album_tracks()`, `get_artist_tracks()` - Track lists
- `get_album_track_ids()`, `get_artist_track_ids()` - Track IDs with filters

### Lyrics Operations
- `download_lyrics()` - Auto-download from LRCLIB
- `retrieve_lyrics()` - Get raw response from LRCLIB
- `retrieve_lyrics_by_id()` - Get by track ID
- `search_lyrics()` - Search LRCLIB database
- `apply_lyrics()` - Apply specific LRCLIB result
- `save_lyrics()` - Save manually edited lyrics
- `publish_lyrics()` - Upload to LRCLIB (with PoW)
- `flag_lyrics()` - Report to LRCLIB (with PoW)

### Audio Playback
- `play_track()`, `pause_track()`, `resume_track()` - Playback control
- `seek_track()` - Jump to position
- `stop_track()` - Stop playback
- `set_volume()` - Adjust volume

### Debug
- `open_devtools()` - Open browser devtools
- `drain_notifications()` - Get queued notifications

## Events (Backend → Frontend)

Emitted by backend, listened by frontend:

### Scan Events (NEW)
- `scan-progress` - Real-time scan progress
  ```typescript
  {
    phase: 'processing' | 'updating';
    progress: number;           // 0.0 to 1.0
    filesProcessed: number;
    filesTotal: number;
    message: string;            // e.g., "Processing files: 1234/5000"
  }
  ```
- `scan-complete` - Scan finished
  ```typescript
  {
    totalFiles: number;
    added: number;
    modified: number;
    deleted: number;
    moved: number;
    unchanged: number;
    isInitialScan: boolean;
    durationMs: number;
  }
  ```

### Legacy Events (DEPRECATED)
- `initialize-progress` - Old scan progress format (DEPRECATED, use `scan-progress`)

### Other Events
- `player-state` - Current playback state
- `reload-track-id` - Request to refresh track data
- `publish-lyrics-progress` - Publishing status updates
- `flag-lyrics-progress` - Flagging status updates

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop framework |
| `rusqlite` | SQLite database |
| `lofty` | Audio metadata reading/writing |
| `kira` | Audio playback |
| `reqwest` | HTTP client for LRCLIB API |
| `rayon` | Parallel processing for scanning |
| `globwalk` | File glob patterns |
| `xxhash-rust` | Fast content hashing for move detection |
| `serde` | Serialization |
| `anyhow` / `thiserror` | Error handling |
| `ring` / `data-encoding` | Cryptography for PoW |
| `lrc` | LRC lyrics parsing |
| `secular` | Unicode normalization |

## Architecture Patterns

1. **Command Pattern**: All frontend-backend communication via Tauri commands
2. **State Management**: Global `AppState` with mutex-protected resources
3. **Event-Driven**: Async events for progress updates and state changes
4. **Repository Pattern**: Database operations abstracted in `db.rs`
5. **Service Layer**: High-level operations in `library.rs` and `lyrics.rs`
6. **Modular API Client**: LRCLIB operations organized by endpoint
7. **Batch Processing**: File scanning uses parallel batches for performance

## Security Considerations

- Database stored in app's data directory
- Asset protocol enabled for reading music files
- CSP configured for media playback
- Proof-of-work for write operations to LRCLIB (rate limiting)
- User-agent identification in API requests

## Configuration

**tauri.conf.json:**
- Window size: 1024x768 (min)
- CSP allows asset protocol and media playback
- macOS/Linux app metadata

**Cargo.toml:**
- Rust edition 2021
- Tauri v2 with plugins: dialog, shell, global-shortcut, os
- Profile optimizations for audio crates in dev mode
