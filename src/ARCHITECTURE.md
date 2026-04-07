# LRCGET Frontend Architecture

## Overview

The `src/` directory contains the Vue 3 frontend for LRCGET. It runs inside a Tauri webview and is responsible for:

- bootstrapping the Vue application and shared UI plugins
- switching between library setup and the main library workspace
- rendering tracks, albums, artists, lyrics, and playback controls
- coordinating long-running flows such as library scans, lyric downloads, and manual lyric editing
- calling Tauri commands and listening to backend events from the Rust side

The frontend is intentionally thin on persistence. Most durable state lives in the Rust backend and SQLite database, while the Vue layer keeps reactive session state for the current screen, modal, playback view, and download queue.

## Directory Structure

```text
src/
├── App.vue                          # Root app shell and top-level boot logic
├── main.js                          # Vue app entry point and plugin registration
├── style.css                        # Shared Tailwind component classes and global styles
├── assets/                          # Static images used by the UI
├── components/
│   ├── About.vue                    # About/update modal
│   ├── ChooseDirectory.vue          # Initial directory selection flow
│   ├── CopyablePre.vue              # Copy-friendly preformatted text block
│   ├── Library.vue                  # Main workspace shell after initialization
│   ├── NowPlaying.vue               # Bottom playback area and lyrics display
│   ├── SelectStrategy.vue           # Strategy selection UI used by lyrics flows
│   ├── common/                      # Reusable controls and modal shell
│   ├── icons/                       # Small custom icon components
│   ├── library/                     # Library browsing, search, config, and lyric workflows
│   │   ├── album-list/              # Album-specific list and item components
│   │   ├── artist-list/             # Artist-specific list and item components
│   │   ├── edit-lyrics/             # Legacy edit modal subcomponents, publish flows, and CodeMirror helpers
│   │   ├── edit-lyrics-v2/          # V2 edit modal pieces (plain CodeMirror tab + synced row/insert/empty-state components)
│   │   ├── my-lrclib/               # User-contributed LRCLIB views and actions
│   │   ├── search-lyrics/           # Preview UI for remote lyric search results
│   │   └── track-list/              # Virtualized track rows and row actions
│   └── now-playing/                 # Seek bar, volume, synced/plain lyric viewers
├── composables/
│   ├── downloader.js                # Shared download queue and progress state
│   ├── edit-lyrics.js               # Opens the legacy edit-lyrics modal flow
│   ├── edit-lyrics-v2.js            # Opens the EditLyricsV2 modal flow
│   ├── edit-lyrics-v2/              # V2 lyricsfile document, playback, synced hotkey, insert-hover, and inline-edit composables
│   ├── edit-lyrics/                 # Edit modal document, hotkey, publish, and playback-sync composables
│   ├── global-state.js              # Shared theme/hotkey/LRCLIB instance state
│   ├── player.js                    # Shared playback state backed by Tauri events
│   ├── search-library.js            # Shared library search text and filters
│   └── search-lyrics.js             # Opens the search-lyrics modal flow
└── utils/                           # Small helpers for lyrics, linting, durations, line counts
```

## Application Boot Flow

### Entry Point (`main.js`)

`main.js` creates the Vue app, registers global plugins, and mounts the root component.

Registered plugins/services:

- `vue-toastification` for non-blocking notifications
- `vue-final-modal` for modal-driven workflows
- `floating-vue` for tooltips and dropdowns
- global `BaseModal` and `VueFinalModal` components

It also imports `style.css`, which defines the shared design tokens and utility classes built on top of Tailwind.

### Root Shell (`App.vue`)

`App.vue` owns the first layer of application state:

- `loading` - whether the app is still fetching startup data
- `init` - whether the backend reports that the library is initialized
- `shouldScan` - one-shot signal telling `Library.vue` to trigger a scan after directory changes

Startup sequence:

1. call `get_init` to decide whether to show setup or the library workspace
2. call `get_config` and push config-derived values into shared frontend state
3. apply light/dark mode handling based on `theme_mode` and OS theme events
4. start the background lyrics download loop from `useDownloader()`
5. poll `drain_notifications` and surface backend notifications as toasts

Rendered branches:

- `ChooseDirectory.vue` when the user still needs to pick scan folders
- `Library.vue` once the library is ready to browse

## High-Level UI Composition

### Setup Flow (`ChooseDirectory.vue`)

This screen is shown before the main library becomes available.

Responsibilities:

- load existing directories via `get_directories`
- offer a native folder picker through `@tauri-apps/plugin-dialog`
- persist changes through `set_directories`
- emit `directoriesChanged` only when the selected directory set actually changed
- emit `progressStep` so the root app can switch into the main library UI

The component does not scan directly. It delegates that decision upward to `App.vue`, which passes the scan trigger into `Library.vue`.

### Main Workspace (`Library.vue`)

`Library.vue` is the main shell after setup. It composes:

- `LibraryHeader.vue` for top-level navigation and actions
- one of the tabbed content panes:
  - `TrackList.vue`
  - `AlbumList.vue`
  - `ArtistList.vue`
  - `MyLrclib.vue`
- `NowPlaying.vue` as the persistent bottom playback panel

It also owns the scan lifecycle for the frontend:

- subscribes to `scan-progress`
- subscribes to `scan-complete`
- invokes `scan_library`
- shows a loading/scanning placeholder until the library is ready

Modal workflows opened from this shell:

- `Config.vue`
- `About.vue`
- `DownloadViewer.vue`

## Shared Frontend State

The frontend uses composables with module-level refs instead of a formal store library.

### `useGlobalState()`

Shared values:

- `isHotkey` - enables or disables keyboard playback shortcuts
- `themeMode` - `auto`, `light`, or `dark`
- `lrclibInstance` - backend endpoint override for LRCLIB-compatible servers

This composable is mainly used by `App.vue`, `NowPlaying.vue`, and configuration-related UI.

### `usePlayer()`

Playback state is shared globally through module refs:

- `playingTrack`
- `status`
- `duration`
- `progress`
- `volume`

It bridges the UI and backend by:

- listening to `player-state`
- refreshing the current track on `reload-track-id`
- invoking `play_track`, `pause_track`, `resume_track`, `seek_track`, `stop_track`, and `set_volume`

This keeps player controls and lyric viewers synchronized without passing state through many component levels.

### `useDownloader()`

This composable implements a single shared queue for background lyric downloads.

State includes:

- queued track ids
- current item being processed
- download log
- success and failure counters
- progress over the active batch

`downloadNext()` runs as a perpetual async loop started by `App.vue`. UI components only enqueue work and inspect shared progress.

### Search and Modal Composables

- `useSearchLibrary()` stores the active search text and track filters used by library panes
- `useSearchLyrics()` opens `SearchLyrics.vue` as a modal for a selected track
- `useEditLyrics()` opens `EditLyrics.vue` as a modal for a selected track
- `useEditLyricsV2()` opens `EditLyricsV2.vue` (tabbed plain/synced editor scaffold)

## Library Browsing Architecture

### Search and Filtering

The search bar updates shared query state through `useSearchLibrary()`. Tab content components react to that shared state and fetch new ids from the backend.

This architecture keeps filtering centralized while letting each tab decide how it loads its own data.

### Tracks (`components/library/TrackList.vue`)

The tracks view is optimized for large libraries:

- only track ids are fetched initially via `get_track_ids`
- row rendering uses `@tanstack/vue-virtual`
- each visible row is delegated to `track-list/TrackItem.vue`
- row-level actions trigger playback, lyric download, lyric search, and lyric editing flows

This avoids loading the full track dataset into the DOM at once.

### Albums and Artists

The album and artist tabs follow the same backend-driven pattern:

- fetch ids or grouped entities from Tauri commands
- render lightweight list items
- lazily show track subsets inside album/artist detail views

The folder split under `album-list/` and `artist-list/` keeps entity-specific row rendering separate from the main shell.

### My LRCLIB

`MyLrclib.vue` and the `my-lrclib/` subfolder cover user-generated LRCLIB workflows such as previewing, editing, publishing, and flagging lyrics. These flows sit beside local library browsing but still rely on the same track entities and backend command layer.

## Lyrics Workflows

The frontend supports three major lyric flows:

### 1. Passive Display

`NowPlaying.vue` chooses between:

- `LyricsViewer.vue` for synchronized LRC lyrics
- `PlainLyricsViewer.vue` for plain text lyrics
- a minimal divider when no lyrics are available or the track is instrumental

The synced viewer can emit `lyrics-clicked`, which maps directly to `seek()` so users can jump within the track by clicking a lyric line.

### 2. Search and Apply

`SearchLyrics.vue` and `search-lyrics/Preview.vue` drive remote LRCLIB lookup for a selected track. The modal-oriented architecture lets track rows open search UI without adding more nested routing.

### 3. Edit, Save, and Publish

`EditLyrics.vue` acts as the current full-featured editor modal. Most of the feature logic is split between `components/library/edit-lyrics/` child components and `composables/edit-lyrics/` composables:

- `useLyricsDocument()` owns the editable lyric text, dirty tracking, lint state, and save flow
- `useLyricsEditorCommands()` owns sync/repeat/timestamp-shift/editor mutation commands
- `useLyricsPlaybackSync()` keeps the runner and current highlighted lyric line in sync with player progress
- `useLyricsPublish()` decides whether to open the synced or plain-text publish modal and exposes publish UI state
- `useLyricsEditorHotkeys()` binds the modal-specific keyboard shortcuts

This split keeps `EditLyrics.vue` focused on wiring together the modal, child toolbar/editor components, and shared player state.

`EditLyricsV2.vue` is a parallel editing flow that keeps the playback control bar, removes the legacy sync-command toolbar, and introduces tabbed editing with:

- a plain-lyrics CodeMirror tab
- a synced-lyrics interactive line list UI with hover/selection controls (play, sync, +/-100ms, delete), floating between-line insert actions, and inline single-line editing
- keyboard-driven synced-line workflows (arrow-key navigation, space/enter sync, left/right +/-100ms) that are paused while a line is actively being edited
- visual now-playing feedback in synced mode by bolding the active line timestamp and lyric text as playback progresses
- an empty-state synced editor panel that offers importing lines from plain lyrics or creating the first line manually

The V2 modal keeps layout wiring in `EditLyricsV2.vue` while delegating mutable document state to `useEditLyricsV2Document()`, playback actions to `useEditLyricsV2Playback()`, modal-level save/plain-zoom shortcuts to `useEditLyricsV2Hotkeys()`, and synced editor key handling to `useEditLyricsV2SyncedHotkeys()` under `composables/edit-lyrics-v2/`.

`SyncedLyricsEditor.vue` is further split into focused pieces:

- `SyncedLyricsEmptyState.vue` for the zero-lines onboarding panel
- `SyncedLyricsLineRow.vue` for each interactive synced row (controls, timestamp, inline edit, delete)
- `SyncedInsertButton.vue` for floating between-row insert affordances
- `SyncedWordTimingLane.vue` - Fixed timeline showing word timing controls for the selected line
- `SyncedWordTimingSegment.vue` - Visual word segment component sized by adjacent word boundaries
- `useEditLyricsV2SyncedInsertHover()` for insert-hover geometry and opacity behavior
- `useEditLyricsV2SyncedInlineEditing()` for inline row text edit lifecycle and emit wiring

**Word Timing Lane**

The `SyncedWordTimingLane.vue` component provides a fixed horizontal timeline at the top of the synced lyrics editor for word-level timing adjustments:

- Displays word segments for the currently selected line only
- Auto-generates word tokens from line text using `word-tokenizer.js` utilities
- Supports both Latin (space-delimited) and CJK (character-based) tokenization
- Renders words as contiguous timeline segments whose widths are derived from the next word boundary
- Uses dedicated boundary handles between words so the first word stays fixed to the line start
- Previews boundary movement during drag and commits the new right-word `start_ms` on pointer release
- Enforces monotonic timing constraints (no overlaps, must stay within line bounds)
- "Distribute evenly" button to reset word timings to equal spacing across the line window
- Shows playhead indicator synced to current playback position
- Grid lines for visual time reference (every 500ms)

The `word-tokenizer.js` utility provides:

- `tokenizeText()` - Splits text into tokens (CJK per-character, Latin space-delimited)
- `generateWordsFromLine()` - Creates word array from line text
- `distributeWordTimings()` - Evenly distributes word start times across line duration
- `hasValidWords()` - Validates existing word array matches line text
- `ensureLineWords()` - Ensures line has valid words with distributed timings

## Playback Architecture

`NowPlaying.vue` is always mounted inside the library workspace so playback controls remain available while switching tabs.

Key responsibilities:

- render current track metadata
- expose seek, play/pause/resume/replay, and volume controls
- render either synced or plain lyrics for the active track
- register keyboard shortcuts for space/enter and left/right arrows

The keyboard shortcut behavior is intentionally guarded:

- hotkeys can be disabled through shared state
- shortcuts are ignored while typing in inputs, textareas, or CodeMirror editors

## Styling System

The frontend uses Tailwind CSS with a small custom design system layered in `src/style.css`.

Key traits:

- custom `brave` and `hoa` color palettes from `tailwind.config.cjs`
- reusable semantic classes such as `.button`, `.input`, `.textarea`, `.modal-content`, and `.link`
- class-based dark mode controlled by the root `html.dark` class
- styling hooks for Vue Toastification, Floating Vue, and CodeMirror

Most component templates stay utility-first, while repeated visual patterns are promoted into named classes in `style.css`.

## Backend Integration Pattern

The Vue frontend talks to Rust through two mechanisms:

### Tauri Commands

Most data mutations and queries are done with `invoke(...)`, including:

- configuration and directory management
- library scans and data queries
- playback control
- lyric download, save, search, publish, and flag actions

### Tauri Events

Long-running or externally updated state comes from backend events:

- `scan-progress`
- `scan-complete`
- `player-state`
- `reload-track-id`

This split keeps request/response operations simple while allowing the backend to push progress and playback updates asynchronously.

## Utilities

The `src/utils/` folder contains presentation and lyric-specific helpers, including:

- duration formatting for the now-playing bar
- line-count helpers used by search results
- linting helpers for plain text and LRC lyrics
- lyric parsing/normalization helpers reused by editing and preview components
- Lyricsfile YAML parsing/serialization helpers for the V2 editing flow

## Architectural Patterns

1. **Shell + modal composition**: the app keeps one main workspace and opens focused tasks in modals instead of introducing a router.
2. **Composable shared state**: lightweight module-level refs replace a heavier store solution.
3. **Backend-owned persistence**: the frontend derives most durable state from Rust commands rather than owning a separate client-side data layer.
4. **Event-driven live updates**: scanning and playback rely on backend events to keep the UI current.
5. **Large-list optimization**: track rendering uses virtualization and id-based fetching to stay responsive on big music libraries.

## Important Constraints and Tradeoffs

- There is no Vue Router; navigation is tab- and modal-based.
- Shared composables behave like singletons, so they are simple but global by design.
- The app prefers backend truth over local caching, which reduces frontend complexity but increases the number of Tauri calls.
- The downloader uses a perpetual loop, which is straightforward operationally but means app startup is responsible for launching the queue worker exactly once.

## Frontend Technology Summary

- Vue 3 with `<script setup>` single-file components
- Vite for bundling and dev server integration with Tauri
- Tailwind CSS for styling
- Vue Final Modal for modal workflows
- Floating Vue for tooltips/dropdowns
- Vue Toastification for notifications
- TanStack Vue Virtual for scalable track rendering
- CodeMirror-based editors inside lyric editing flows
