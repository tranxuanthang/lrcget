# LRCGET Frontend Architecture

## Architecture Overview

Vue 3 frontend in Tauri webview. Handles UI, playback, library browsing, lyric editing, and backend communication. Session state only; persistence in Rust/SQLite.

**Tech Stack**: Vue 3 (`<script setup>`), Vite, Tailwind CSS, Vue Final Modal, Floating Vue, Vue Toastification, TanStack Vue Virtual, CodeMirror, `unplugin-icons` + Iconify Material Design Icons (`@iconify-json/mdi`).

**Code Quality**: ESLint with Vue plugin + Prettier for formatting. Run `npm run lint` to check, `npm run format` to format.

**Core Patterns**:
| Pattern | Implementation |
|---------|---------------|
| Shell + modals | Main workspace + modal tasks (no router) |
| Composable state | Module-level refs, no store library |
| Backend-owned persistence | State from Rust commands, minimal client caching |
| Event-driven updates | Backend pushes scan/playback events |
| Virtualized lists | `@tanstack/vue-virtual` for large libraries |

## Project Structure

```
src/
├── App.vue                 # Root shell
├── main.js                 # Entry: plugins, Tailwind import
├── style.css               # Tailwind + custom classes
├── assets/                 # Static images
├── components/
│   ├── common/             # Reusable controls
│   ├── icons/              # Custom icons
│   ├── library/            # Library views + lyric edit/search
│   └── now-playing/        # Playback controls
├── composables/            # Shared state (player, downloader, search, edit)
└── utils/                  # Helpers (lyrics, durations, linting)
```

## State Management

Module-level ref composables (singletons by design):

| Composable                              | Purpose                                                                                                                                                                         |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useGlobalState()`                      | `isHotkey`, `themeMode`, `lrclibInstance`                                                                                                                                       |
| `usePlayer()`                           | `playingTrack`, `status`, `duration`, `progress`, `volume`. Supports both library tracks (with `id`) and file-based tracks (with `file_path`). Listens to `player-state` events |
| `useDownloader()`                       | Download queue, progress. Loop started by App.vue at boot                                                                                                                       |
| `useExporter()`                         | Mass export queue, progress. Used by ExportViewer modal                                                                                                                         |
| `useSearchLibrary()`                    | Search text, filters                                                                                                                                                            |
| `useSearchLyrics()`                     | Search modal state                                                                                                                                                              |
| `useEditLyrics()` / `useEditLyricsV2()` | Edit modal state                                                                                                                                                                |

**Boot Flow**: `main.js` → Vue app init → `App.vue` checks `get_init()` → shows `ChooseDirectory.vue` (setup) or `Library.vue` (main). Loads config, applies theme, starts downloader loop.

## UI Architecture

**Main Shells**:

- `ChooseDirectory.vue` - Setup: folder picker, persists via `set_directories`, emits to trigger library view
- `Library.vue` - Header + tabbed panes (Tracks/Albums/Artists/MyLrclib) + `NowPlaying.vue`. Manages scan lifecycle (`scan-progress`, `scan-complete`, `scan_library`)
- `NowPlaying.vue` - Persistent bottom panel. Track metadata, seek/play/volume, lyrics. Keyboard shortcuts (space/enter/arrows) disabled when typing or via `isHotkey` state

**Modals**: `Config.vue`, `About.vue`, `DownloadViewer.vue`

**Common Components**:
- `BaseModal.vue` — Reusable modal wrapper using Vue Final Modal with consistent styling
- `ConfirmModal.vue` — Simple confirmation modal for user actions (e.g., confirming close on unsaved changes)

## Library & Lyrics System

**Library Browsing**:

- Search via `useSearchLibrary()`; tabs fetch IDs from backend
- Tracks: Virtualized (`@tanstack/vue-virtual`), IDs only; `TrackItem.vue` for actions
- Albums/Artists: Backend-driven, lazy-load track subsets

**Lyrics Workflows**:

- **Display**: `LyricsViewer.vue` (synced) / `PlainLyricsViewer.vue`. Click to `seek()`
- **Search**: `SearchLyrics.vue` + `Preview.vue` for LRCLIB lookup, including word-level highlight in preview when Lyricsfile word timings are available
- **LRCLIB normalization**: `normalizeLrclibLyrics()` in `utils/lyricsfile.js` derives plain/synced/instrumental state from `lyricsfile` when LRCLIB responses omit direct `plainLyrics`/`syncedLyrics`
- **Edit/Publish**:
  - _Legacy_: `EditLyrics.vue` — CodeMirror + sync toolbar
  - _V2_: `EditLyricsV2.vue` — CodeMirror + interactive synced view + word timing lane. Accepts three props: `audioSource` (for playback only), `lyricsfile` (for editing operations), and `trackId` (for determining save behavior). The `audioSource` format is `{ type: 'library'|'file', id?, file_path?, duration?, title?, artist_name?, album_name? }`. The `lyricsfile` format is `{ id?, content, metadata?: { title, artist, album, duration_ms } }`. The `trackId` is `null` for temporary associations (LRCLIB Browser flow) or a track ID for library tracks. This design separates audio playback from lyrics editing, allowing the editor to work with both library tracks (via track ID) and standalone lyricsfiles (via lyricsfile ID) without flow-specific conditional logic. The separate `trackId` prop handles the case where a library track is temporarily associated with a standalone lyricsfile for playback - in this case the lyricsfile should NOT be associated with the track on save. Synced line selection keeps the active row in view while navigating or syncing, and the word timing lane supports dragging the first word boundary to update the line start while keeping the current lane window stable until selection changes; line timestamp edits do not automatically shift existing word timings. Header actions use a split save dropdown (`EditLyricsV2HeaderActions.vue`) with `Save`, `Save and Publish`, and manual export for `.txt`, `.lrc`, and embedded lyrics. V2 publish is isolated in `useEditLyricsV2Publish.js` + `EditLyricsV2PublishModal.vue`, and V2 export is isolated in `useEditLyricsV2Export.js`; both send serialized Lyricsfile payloads. When closing the editor with unsaved changes (dirty state), a confirmation modal (`ConfirmModal.vue`) is shown to warn the user before discarding changes.
  - _V2 Instrumental Support_: Tracks can be marked as instrumental via `PlainLyricsEmptyState.vue` or `SyncedLyricsEmptyState.vue`. When marked, the plain/synced tab switcher is disabled and a centered popup appears with an "Unmark as instrumental" button. The instrumental state is stored in the `lyricsfile` metadata and managed by `useEditLyricsV2Document.js`.
- **Export (Mass)**: `LibraryHeader.vue` has an export button (with dropdown) that emits `exportAllLyrics` → `Library.vue` opens `ExportViewer.vue` modal → `useExporter()` composable manages queue → invokes `export_track_lyrics` command per track. Exports to `.txt`, `.lrc`, and/or embedded metadata.
- **My LRCLIB**: User workflows (preview, edit, publish, flag) in `my-lrclib/`
- **Track Association (My LRCLIB Edit Flow)**: When editing lyrics from My LRCLIB search results, the flow is: (1) Call `prepare_lrclib_lyricsfile(lrclibId)` to fetch/create the lyricsfile in the database (stored with `lrclib_instance` and `lrclib_id`). If lyrics already exist locally, show `LyricsfileConflictModal.vue` to ask if user wants to redownload from LRCLIB or continue with local version. (2) Show `AssociateTrackModal.vue` for selecting either a library track or a file from the computer. (3) Open `EditLyricsV2.vue` with the selected audio source (as `audioSource` prop), the standalone lyricsfile (as `lyricsfile` prop), and `trackId: null` to indicate this is a temporary association. The `get_audio_metadata` backend command extracts metadata from selected files using the existing scanner logic. Supports playback for both library tracks and arbitrary file-based tracks. Note: Even when a library track is selected for playback, `trackId` is `null` to ensure the standalone lyricsfile is not associated with the track after editing.

Utils: `src/utils/` (parsing, linting), Composables: `composables/edit-lyrics/`, `composables/edit-lyrics-v2/`, `composables/export.js`

## Technical Details

**Styling**: Tailwind CSS + custom classes in `style.css`. Palettes (`brave`, `hoa`) from `tailwind.config.cjs`. Dark mode via `html.dark`. Semantic classes: `.button`, `.input`, `.modal-content`, `.link`.

**Icons**: Icons are imported directly per-file from `~icons/mdi/*` (powered by `unplugin-icons` in `vite.config.js`). Avoid adding `mdue`; use MDI icon imports instead.

**Utilities**: `src/utils/` — duration formatting, line counts, lyric parsing/linting, Lyricsfile YAML helpers (including LRCLIB payload normalization for lyricsfile-first responses).

### Playback System

The player supports two types of tracks via a unified `PlayableTrack` type:

| Track Source       | Required Fields | Backend Command                               |
| ------------------ | --------------- | --------------------------------------------- |
| Library (Database) | `id`            | `play_track({ trackId: id, ...metadata })`    |
| File Picker        | `file_path`     | `play_track({ filePath: path, ...metadata })` |

**`usePlayer().playTrack(track)`** automatically detects the track source:

- If `track.id` is present → Database track, fetches full data from SQLite
- If only `track.file_path` is present → File-based track, metadata extracted from file or provided directly

This enables the V2 lyrics editor to support playback for:

- Scanned library tracks (full features)
- Arbitrary files from file picker (full features)
- Tracks without audio (disabled playback, manual timestamp editing only)

## Constraints

- No Vue Router (tab/modal navigation only)
- Global composables are singletons by design
- Prefer Tauri calls over frontend caching
- Downloader perpetual loop starts once at boot
