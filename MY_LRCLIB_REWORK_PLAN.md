# My LRCLIB Edit Lyrics Rework Plan

## Overview

Rework the edit lyrics flow from the My LRCLIB tab to provide a unified editing experience that works both with and without local audio files.

### Current State Analysis

**My LRCLIB Tab:**

- Uses legacy `EditLyrics.vue` (CodeMirror-based, no playback)
- Tracks from LRCLIB API have: `id` (LRCLIB ID), `name`, `artistName`, `albumName`, `duration`, `plainLyrics`, `syncedLyrics`, `lyricsfile`
- **Missing:** Local DB `id`, `file_path` → cannot play audio or use V2 editor features

**Local Library Flow (V2):**

- Uses modern `EditLyricsV2.vue` with full playback, sync, word timing
- Requires `PersistentTrack` with local `id` and `file_path`

---

## Implementation Phases

### Phase 1: Create Track Matching Backend (Rust)

**New Command: `find_matching_tracks`**

```rust
#[tauri::command]
async fn find_matching_tracks(
    title: String,
    album_name: String,
    artist_name: String,
    duration: Option<f64>,
    app_handle: AppHandle,
) -> Result<Vec<PersistentTrack>, String>
```

**Matching Strategy:**

**Strong/Exact Match Criteria:**

- Title + Artist + Album all normalized through `prepare_input()` (from `src-tauri/src/utils.rs`)
- AND duration is within ±2 seconds (if duration is provided)

**Match Results Priority:**

1. **Strong matches** - Pass all criteria above
2. **Partial matches** - Title matches but artist/album differ
3. **All results returned** - Frontend decides how to present them

**New DB Functions** (`src-tauri/src/db.rs`):

```rust
pub fn find_tracks_by_metadata(
    title: &str,
    artist_name: Option<&str>,
    album_name: Option<&str>,
    duration: Option<f64>,
    db: &Connection,
) -> Result<Vec<PersistentTrack>>
```

---

### Phase 2: Create "Associate Track" Modal (Vue)

**New Component: `src/components/library/my-lrclib/AssociateTrackModal.vue`**

Flow when user clicks "Edit Lyrics" on LRCLIB result:

1. **Check for existing match** (automatic)
   - Call `find_matching_tracks` with LRCLIB track metadata
   - If **exactly 1 strong match** → Auto-open V2 editor with that track
   - If **multiple matches** or **no match** → Show association modal

2. **Association Modal UI:**
   ```
   ┌─────────────────────────────────────────┐
   │  Associate Track for Lyrics Editing     │
   ├─────────────────────────────────────────┤
   │  LRCLIB: "Song Name" by Artist          │
   │  Duration: 3:45                         │
   ├─────────────────────────────────────────┤
   │  [MATCHING TRACKS IN LIBRARY]          │
   │  ○ Track 1 (Exact match)                │
   │  ○ Track 2 (Title matches, diff album) │
   │                                         │
   │  [SELECT FILE ON COMPUTER]             │
   │  ○ Choose audio file...                 │
   │                                         │
   │  ─────────── OR ───────────            │
   │                                         │
   │  [Edit Without Audio]                  │
   │  ⚠️ Limited features: no playback,     │
   │     no sync button                      │
   ├─────────────────────────────────────────┤
   │  [Cancel]  [Edit Lyrics →]              │
   └─────────────────────────────────────────┘
   ```

---

### Phase 3: File Picker Integration

**New Command: `select_audio_file`**

```rust
#[tauri::command]
async fn select_audio_file(app_handle: AppHandle) -> Result<Option<String>, String> {
    // Use tauri-plugin-dialog to pick audio file
    // Return file path or None if cancelled
}
```

**New Command: `get_audio_metadata`**

```rust
#[tauri::command]
async fn get_audio_metadata(file_path: String) -> Result<TrackMetadata, String> {
    // Reuse existing scanner/metadata.rs logic
    // Extract full metadata: title, artist, album, duration, etc.
}
```

**File Picker Flow:**

1. User clicks "Choose audio file..."
2. Open Tauri dialog with audio file filters (`.mp3`, `.flac`, `.ogg`, `.m4a`, etc.)
3. On selection, extract **full metadata** via `get_audio_metadata` (reusing scanner logic)
4. Show preview: "Selected: filename (Duration: X:XX)"
5. **Metadata Mismatch Warning:** If extracted metadata doesn't match the LRCLIB track:
   - Show warning dialog: "The selected file's metadata doesn't match the lyrics. Continue anyway?"
   - Options: "Continue" / "Pick Different File" / "Cancel"
6. User confirms → Create temporary track object for editing

---

### Phase 4: V2 Editor Adaptations

**Modify `src/components/library/EditLyricsV2.vue` for "Edit Without Audio" mode:**

When no local track/file is associated:

```javascript
// New props or composable state
const hasAudioSource = computed(() => !!editingTrack.value?.file_path)

// Disable playback-dependent features
const isPlaybackDisabled = computed(() => !hasAudioSource.value)
```

**UI Changes for "Edit Without Audio":**

- Show warning banner: "Editing without audio - playback features disabled"
- Disable: Play button, sync line button, seek functionality
- Keep enabled: Plain/synced tabs, timestamp editing (manual), word timing, save, publish
- Auto-mark as dirty on any change (since there's no original file to compare)

**New Composable: `useEditLyricsV2Lite.js`**

- Variant for LRCLIB-only editing
- Skips playback-related composables
- Initializes from LRCLIB `lyricsfile` or `plainLyrics`/`syncedLyrics`

---

### Phase 5: Reuse Strategy - Unified Editor Entry Point

**Option A: Refactor `EditLyricsV2.vue`** (Recommended)

- Add `mode` prop: `'local'` | `'lrclib-associated'` | `'lrclib-lite'`
- Conditional playback integration based on mode
- Same UI, different capabilities

**Option B: Create `EditLyricsV2Lite.vue`**

- Fork of V2 without playback
- Simpler but duplicates code

**Recommendation: Option A** with these changes to `EditLyricsV2.vue`:

```javascript
const props = defineProps({
  track: Object,
  mode: {
    type: String,
    default: 'local', // 'local', 'lrclib-associated', 'lrclib-lite'
    validator: v => ['local', 'lrclib-associated', 'lrclib-lite'].includes(v),
  },
})

const hasPlayback = computed(() => props.mode !== 'lrclib-lite')
const isAssociatedTrack = computed(() => props.mode === 'lrclib-associated')
```

---

### Phase 6: Update My LRCLIB Flow

**Modify `src/components/library/my-lrclib/SearchResult.vue`:**

```javascript
const setEditingTrack = async lrclibTrack => {
  isOpeningTrack.value = true
  try {
    // 1. Fetch full lyrics from LRCLIB
    const refreshedTrack = await invoke('retrieve_lyrics_by_id', { id: lrclibTrack.id })

    // 2. Find matching local tracks
    const matches = await invoke('find_matching_tracks', {
      title: refreshedTrack.name,
      album_name: refreshedTrack.albumName,
      artist_name: refreshedTrack.artistName,
      duration: refreshedTrack.duration,
    })

    if (matches.length === 1 && isStrongMatch(matches[0], refreshedTrack)) {
      // Auto-associate and open V2
      openEditLyricsV2(matches[0], refreshedTrack.lyricsfile)
    } else {
      // Show association modal
      openAssociateTrackModal({
        lrclibTrack: refreshedTrack,
        matchingTracks: matches,
        onAssociate: associatedTrack => {
          openEditLyricsV2(associatedTrack, refreshedTrack.lyricsfile)
        },
        onSelectFile: async () => {
          const filePath = await invoke('select_audio_file')
          if (filePath) {
            const metadata = await invoke('get_audio_metadata', { filePath })
            const tempTrack = createTempTrack(metadata, filePath)
            openEditLyricsV2(tempTrack, refreshedTrack.lyricsfile)
          }
        },
        onEditLite: () => {
          openEditLyricsV2Lite(refreshedTrack)
        },
      })
    }
  } catch (error) {
    toast.error('Error opening lyrics editor')
  } finally {
    isOpeningTrack.value = false
  }
}
```

---

### Phase 7: Lyrics Saving (No Track Association)

**Lyricsfiles Table:**

The `lyricsfiles` table is designed to not require a track association. When editing in "lrclib-lite" mode (no audio file):

- **Save to `lyricsfiles` table:** YES - fully supported
- **Save to sidecar file:** NO - no associated track file
- **Save to embedded metadata:** NO - no audio file to embed into
- **Publish to LRCLIB:** YES - primary use case for lite mode

---

## File Structure Changes

```
src/
├── components/
│   └── library/
│       ├── my-lrclib/
│       │   ├── SearchResult.vue          # Modified
│       │   ├── AssociateTrackModal.vue   # NEW
│       │   └── EditLyrics.vue            # REMOVE (migrate to V2)
│       └── EditLyricsV2.vue              # Modified for mode support
├── composables/
│   └── edit-lyrics-v2/
│       ├── useEditLyricsV2Document.js    # Modified for LRCLIB init
│       └── useEditLyricsV2Lite.js        # NEW (or extend existing)
src-tauri/
└── src/
    ├── db.rs                             # Add find_tracks_by_metadata
    ├── scanner/
    │   └── metadata.rs                   # Reuse for get_audio_metadata
    └── main.rs                           # Add new commands
```

---

## Implementation Priority

1. **High:** Backend track matching (`find_matching_tracks`)
2. **High:** Association modal UI
3. **Medium:** V2 editor mode adaptations
4. **Medium:** File picker integration
5. **Low:** Metadata mismatch warning dialog
6. **Low:** Remove legacy `EditLyrics.vue`

---

## Design Decisions

### 1. Matching Confidence Threshold

**Strong Match Definition:**

- Title + Artist + Album all normalized through `prepare_input()` (case-insensitive, special chars removed)
- AND duration is within ±2 seconds
- Uses exact matching on normalized strings

### 2. File Picker Scope

Users can pick:

- Any audio file on their computer (via file picker)
- OR choose from already-scanned library tracks
- Both options presented in the association modal

### 3. Metadata Extraction for New Files

**Full metadata extraction** using existing scanner logic:

- Title, artist, album, duration, track number, etc.
- If metadata doesn't match LRCLIB track → show warning dialog
- User can choose to continue anyway or pick a different file

### 4. "Edit Without Audio" Features

| Feature                  | Keep? | Notes                         |
| ------------------------ | ----- | ----------------------------- |
| Timestamp manual editing | Yes   | User can type timestamps      |
| Word timing editing      | Yes   | But no playback verification  |
| Publish to LRCLIB        | Yes   | Primary use case              |
| Save to `lyricsfiles`    | Yes   | Table supports no track assoc |
| Save to sidecar file     | No    | No associated track file      |
| Plain/synced tabs        | Yes   | Full editing capability       |
| Import from plain        | Yes   | Useful for creating synced    |

### 5. Backward Compatibility

**Legacy `EditLyrics.vue`:**

- Remove entirely after V2 migration is complete
- No fallback needed - V2 with lite mode covers all use cases

### 6. Multiple Matches UI

**Edge case handling:**

- Multiple strong matches should be rare
- For simplicity: choose the first result
- Can be improved later if needed

### 7. Audio File Metadata Command

**Reuse existing scanner logic:**

- `scanner/metadata.rs` already handles metadata extraction
- Create `get_audio_metadata` command that calls existing logic
- No new metadata parsing code needed

---

## Notes

- Tauri dialog plugin is already available (`tauri-plugin-dialog = "2"` in Cargo.toml)
- Dialog plugin is already initialized in `main.rs`
- File picker reference implementation in `src/components/ChooseDirectory.vue`
- LRCLIB track metadata structure: `name`, `artistName`, `albumName`, `duration`
- Local track metadata structure: `title`, `artist_name`, `album_name`, `duration`, `file_path`
- The `prepare_input()` function in `utils.rs` handles: lowercase, secular normalization, special char removal, whitespace collapsing

---

## TODO

- [x] Add TODO and history sections to plan
- [x] Phase 1: Create track matching backend (`find_matching_tracks` command)
  - [x] Add `find_tracks_by_metadata` function in `src-tauri/src/db.rs`
  - [x] Add `find_matching_tracks` command in `src-tauri/src/main.rs`
  - [x] Run `cargo check` to verify compilation
- [x] Phase 2: Create "Associate Track" modal (Vue)
  - [x] Create simplified `AssociateTrackModal.vue` component
    - Search input with client-side filtering
    - Results list (max 10)
    - Footer with "Choose file...", "Edit without audio", "Edit with audio" buttons
  - [x] Add `get_audio_metadata` backend command for file picker
  - [x] Update `SearchResult.vue` to show modal on edit click
  - [x] Add `prepare_search_query` backend command
  - [x] Prefill search with normalized title (brackets removed)
- [ ] Phase 3: File picker integration
- [ ] Phase 4: V2 Editor adaptations
- [ ] Phase 5: Unified editor entry point
- [ ] Phase 6: Update My LRCLIB flow
- [ ] Phase 7: Lyrics saving without track association
- [ ] Remove legacy `EditLyrics.vue`

## History

**2026-04-13:** Plan created, Phase 1 implementation completed

- Added `find_tracks_by_metadata()` DB function for metadata-based track search
- Added `find_matching_tracks` Tauri command with strong/partial match quality
- Added `MatchingTrack` and `MatchQuality` types for structured results
- Added `Clone` derive to `PersistentTrack` for frontend use
- All code compiles successfully with `cargo check`

**2026-04-13:** Phase 2 implementation completed

- Created `AssociateTrackModal.vue` with track matching UI
- Added `get_audio_metadata` backend command for file metadata extraction
- Updated `SearchResult.vue` with new editing flow:
  - Auto-opens V2 editor on single strong match (placeholder for Phase 4)
  - Shows association modal for multiple matches or no matches
  - Supports "Edit Without Audio" fallback to legacy editor
- Added `AudioMetadataResponse` struct for file picker integration

---

_Created: April 2026_
_Status: In Progress - Phase 3 Ready (File Picker & V2 Integration)_
