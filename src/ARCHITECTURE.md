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

| Composable           | Purpose                                                                                                                                                                         |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useGlobalState()`   | `isHotkey`, `themeMode`, `lrclibInstance`                                                                                                                                       |
| `usePlayer()`        | `playingTrack`, `status`, `duration`, `progress`, `volume`. Supports both library tracks (with `id`) and file-based tracks (with `file_path`). Listens to `player-state` events |
| `useDownloader()`    | Download queue, progress. Loop started by App.vue at boot                                                                                                                       |
| `useExporter()`      | Mass export queue, progress. Used by ExportViewer modal                                                                                                                         |
| `useSearchLibrary()` | Shared search text and track-centric filters; used by Tracks, Albums, and Artists tabs |
| `useSearchLyrics()`  | Search modal state                                                                                                                                                              |
| `useEditLyricsV2()`  | Edit lyrics modal state                                                                                                                                                         |

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

- Search via `useSearchLibrary()`; shared search text across Tracks/Albums/Artists tabs
- `MiniSearch.vue` with context-aware placeholder; filter dropdown hidden for Albums/Artists
- All tabs use virtualized lists (`@tanstack/vue-virtual`) with IDs only

**Lyrics Workflows**:

| Aspect | Details |
|--------|---------|
| **Display** | `LyricsViewer.vue` (synced) / `PlainLyricsViewer.vue`. Click to `seek()` |
| **Search** | `SearchLyrics.vue` + `Preview.vue` for LRCLIB lookup; word-level highlight when available |
| **Normalization** | `normalizeLrclibLyrics()` derives plain/synced/instrumental from `lyricsfile` when LRCLIB omits direct fields |
| **Edit/Publish** | `EditLyricsV2.vue` — CodeMirror + synced view + word timing lane. Props: `audioSource` (playback), `lyricsfile` (editing), `trackId` (save behavior). Instrumental toggle via `PlainLyricsEmptyState.vue`/`SyncedLyricsEmptyState.vue`. Publish logic in `useEditLyricsV2Publish.js`, export in `useEditLyricsV2Export.js`. Synced tab supports multi-line selection via drag and Ctrl/Cmd+click, with a floating toolbar for bulk rewind/forward/delete |
| **Keyboard Shortcuts** | `KeyboardShortcutsModal.vue` (full reference modal via `BaseModal`). Definitions in `composables/edit-lyrics-v2/keyboardShortcuts.js`. Header keyboard icon opens the modal |
| **Mass Export** | `LibraryHeader.vue` → `ExportViewer.vue` → `useExporter()` queue → `export_track_lyrics` per track |
| **My LRCLIB** | User workflows (preview, edit, publish, flag) in `my-lrclib/` |
| **Track Association** | My LRCLIB edit flow: `prepare_lrclib_lyricsfile()` → `AssociateTrackModal.vue` → `EditLyricsV2.vue` with `trackId: null` (temporary association only) |

Utils: `src/utils/` (parsing, linting), Composables: `composables/edit-lyrics-v2/`, `composables/export.js`

## Technical Details

**Styling**: Tailwind CSS + custom classes in `style.css`. Primary accent palette (`hoa`) from `tailwind.config.cjs`. Structural colors (backgrounds, text, borders) use Tailwind's default `neutral` scale. Dark mode via `html.dark`. Semantic classes: `.button`, `.input`, `.modal-content`, `.link`.

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

## Testing

**Framework**: Vitest. Run `npm test` (once) or `npm run test:watch` (watch mode).

Tests live next to the source files they exercise (e.g. `word-tokenizer.test.js` for `word-tokenizer.js`). Add tests for any utilities that involve non-trivial branching logic (e.g. parsing, tokenization, transformations). Vue component tests are not yet set up.
