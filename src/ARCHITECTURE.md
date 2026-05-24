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
| `useLibraryNavigation()` | Cross-tab navigation: clicking an album/artist name in `TrackItem.vue` or `NowPlaying.vue` switches to the Albums/Artists tab and opens the corresponding entity via `AlbumList`/`ArtistList` exposed methods |

**Boot Flow**: `main.js` → Vue app init → `App.vue` checks `get_init()` → shows `ChooseDirectory.vue` (setup) or `Library.vue` (main). Loads config, applies theme, starts downloader loop.

## UI Architecture

**Main Shells**:

- `ChooseDirectory.vue` - Setup: folder picker, persists via `set_directories`, emits to trigger library view
- `Library.vue` - Header + tabbed panes (Tracks/Albums/Artists/MyLrclib) + `NowPlaying.vue`. Manages scan lifecycle (`scan-progress`, `scan-complete`, `scan_library`)
- `NowPlaying.vue` - Persistent bottom panel. Track metadata, seek/play/volume/speed, lyrics. Keyboard shortcuts (space/enter/arrows) disabled when typing or via `isHotkey` state

**Modals**: `Config.vue`, `About.vue`, `DownloadViewer.vue`

**Common Components**:

- `BaseModal.vue` — Reusable modal wrapper using Vue Final Modal with consistent styling
- `ConfirmModal.vue` — Simple confirmation modal for user actions (e.g., confirming close on unsaved changes)

## Library & Lyrics System

**Library Browsing**:

- Search via `useSearchLibrary()`; shared search text across Tracks/Albums/Artists tabs
- `MiniSearch.vue` with context-aware placeholder; filter dropdown hidden for Albums/Artists
- All tabs use virtualized lists (`@tanstack/vue-virtual`) with IDs only
- Clickable album/artist names in track rows and the NowPlaying panel navigate to the respective tab and open the entity detail view (`AlbumTrackList`/`ArtistTrackList`)

**Lyrics Workflows**:

| Aspect | Details |
|--------|---------|
| **Display** | `LyricsViewer.vue` (synced) / `PlainLyricsViewer.vue`. Click to `seek()` |
| **Search** | `SearchLyrics.vue` + `Preview.vue` for LRCLIB lookup; word-level highlight when available |
| **Normalization** | `normalizeLrclibLyrics()` derives plain/synced/instrumental from `lyricsfile` when LRCLIB omits direct fields |
| **Edit/Publish** | `EditLyricsV2.vue` + `useEditLyricsV2Publish.js` + `useEditLyricsV2Export.js`. For detailed editor behavior, see **Edit/Publish Details** below. |
| **Keyboard Shortcuts** | `KeyboardShortcutsModal.vue` + shared registry in `composables/edit-lyrics-v2/shortcutRegistry.js`. See **Keyboard Shortcuts Details** below. |
| **Mass Export** | `LibraryHeader.vue` → `ExportViewer.vue` → `useExporter()` queue → `export_track_lyrics` per track |
| **My LRCLIB** | User workflows (preview, edit, publish, flag) in `my-lrclib/` |
| **Track Association** | My LRCLIB edit flow: `prepare_lrclib_lyricsfile()` → `AssociateTrackModal.vue` → `EditLyricsV2.vue` with `trackId: null` (temporary association only) |

### Edit/Publish Details

`EditLyricsV2.vue` combines CodeMirror plain editing and synced editing with a word timing lane.

- Props/context: `audioSource` (playback source), `lyricsfile` (editing target), `trackId` (save behavior)
- Instrumental mode: toggle via `PlainLyricsEmptyState.vue` / `SyncedLyricsEmptyState.vue`
- Publish/export: handled by `useEditLyricsV2Publish.js` and `useEditLyricsV2Export.js`
- Synced lines: multi-line selection via drag and Ctrl/Cmd+click, with floating bulk rewind/forward/delete toolbar
- Synced line nudge shortcuts: `Left`/`Right` adjust selected line start by `-/+100ms`; `Shift+Left`/`Shift+Right` adjust selected line end by `-/+100ms`
- End timestamp visibility: in synced rows, the end timestamp pill stays visible even without hover when it differs from the next line's start timestamp (helps surface gaps/overlaps), and color-codes direction (`before` = gap, `after` = overlap)
- Player bar: playback speed control (`0.5x`-`2.0x`)
- Line status: each synced line row shows a tiny word-sync status dot
- Word timing: multi-separator selection (Ctrl/Cmd+click + Shift+click), merge separators (`Delete`/`Backspace`), hover split preview snapped to grapheme boundaries, double-click split at cursor, and `Z` syncs selected separator then advances (last-word sync advances to next line)
- Narrow segment hint: when a segment is too narrow to show text, a visible hint is rendered beneath it with the next word text
- Boundary sync: can cascade adjacent boundaries so sync is not blocked by intervening separators, while staying within line bounds
- Reset behavior: clears persisted word timings and reloads default (non-persisted) segmentation
- Line-start sync behavior: syncing line start shifts existing word boundaries by the same offset
- Selection behavior: selecting a synced line starts at the second boundary by default

### Keyboard Shortcuts Details

Shortcut behavior and shortcut-menu content share one canonical registry:

- Definitions live in `composables/edit-lyrics-v2/shortcutRegistry.js`
- Runtime handlers consume that registry: `useEditLyricsV2Hotkeys.js`, `useEditLyricsV2SyncedHotkeys.js`, `useEditLyricsV2WordTimingHotkeys.js`
- Menu data (`keyboardShortcuts.js`) is derived from the same registry
- Result: shortcut behavior and displayed shortcut menu stay in sync
- Shortcut-aware button tooltips are also derived from the same registry (e.g., `Sync line to current playback (Space)`)
- Shortcuts are configurable via registry override APIs (`setShortcutOverride`, `resetShortcutOverride`, `resetAllShortcutOverrides`) and persisted in browser `localStorage`
- `KeyboardShortcutsModal.vue` includes a Configure mode window to remap shortcuts by key capture and reset per-shortcut or all shortcuts
- Configure mode detects duplicate shortcut assignments and shows warnings both globally and per conflicting shortcut
- Configure mode shows capture/result feedback through button color states (listening, assigned, canceled, reset)
- Modified shortcuts (customized from defaults) are highlighted via key chip and reset-button colors
- Access: header keyboard icon and `Ctrl+/` open `KeyboardShortcutsModal.vue`

Utils: `src/utils/` (parsing, linting), Composables: `composables/edit-lyrics-v2/`, `composables/export.js`. Default word timing tokenization uses backend `segment_words` (Charabia), with frontend tokenizer fallback.

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
