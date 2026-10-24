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
| `useGlobalState()`   | `isHotkey`, `themeMode`, `lrclibInstance`, `spectrogramVisible` (+ `toggleSpectrogramVisible`/`showSpectrogram` actions persisting via `set_spectrogram_visible`), `spectrogramTheme` (+ `selectSpectrogramTheme` action persisting via `set_spectrogram_theme`)                       |
| `usePlayer()`        | `playingTrack`, `status`, `duration`, `progress`, `volume`. Supports both library tracks (with `id`) and file-based tracks (with `file_path`). Listens to `player-state` events |
| `useDownloader()`    | Download queue, progress, 500 ms + jitter request spacing, and LRCLIB cooldown pause. Loop started by App.vue at boot                                                             |
| `useExporter()`      | Mass export queue, progress; each queued track retains submitted formats. Used by ExportViewer modal                                                                                                                         |
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

### Download and Export preferences

`Library.vue` mounts one `DownloadOptionsPopup.vue` for the library-wide button and the album/artist row and detail Download actions. `useDownloadOptions()` owns the shared draft, hydration, submission, and queue preparation. `useDownloadTrigger()` captures an immutable target ID/name and the clicked anchor; row triggers hold no separate popup or preference state. Scrolling outside the popup, recycling/unmounting its trigger, Escape, or dismissal clears the draft. Late hydration or hide events from an older target cannot change the new session. Only confirmation saves preferences and queues tracks for the captured target with the confirmed filters. The manual Export popup stays separate. `useExportPreferences()` (`composables/export-preferences.js`) provides one reactive source backed by `get_config` / scoped `set_export_preferences` commands. Both popups load drafts on opening and save only on submission; dismissing discards drafts. No browser storage is used. Popup styling is scoped to `DownloadOptionsPopup.vue` and manual Export in `LibraryHeader.vue`: theme-aware error text, readable disabled actions, darker accent action backgrounds for text contrast, and viewport-bounded scrolling.

The Download popup also drafts the three existing Configuration “Download lyrics for” choices above a separator and the auto-export controls. `utils/download-filter.js` matches Configuration’s skip-flag mapping (plain-flag precedence), and submission saves those existing flags with export preferences in one scoped update. Queue filters use the captured submission, not returned/re-read config. Configuration remains available and refreshes from `get_config` before opening. The popup contains controls and actionable errors without explanatory export copy.

Shared fields are `export_txt`, `export_lrc`, and `export_embedded`. `auto_export_enabled` belongs only to Download. Download updates applicable format choices when enabled; turning it off preserves all formats. Manual Export does not update the toggle. Both popups show the embedded choice under the experimental `try_embed_lyrics` gate and preserve its remembered value when disabled. Defaults: auto-export off, LRC selected, TXT/embedded off. Automatic export accepts embed-only batches when the gate is enabled; at least one effective format is required.

`downloader.js` queues `{ trackId, autoExport }` entries; `autoExport` is an immutable TXT/LRC/embedded snapshot or null. The album/artist row actions (`AlbumItem.vue`, `ArtistItem.vue`) and detail actions (`AlbumTrackList.vue`, `ArtistTrackList.vue`) use the same popup and snapshots as Download All. Scope queries use `get_album_track_ids` / `get_artist_track_ids` with the submitted skip flags; library-wide queries retain their existing presence filters. Single-track Download, Apply, and editor saves are unchanged. `export.js` queues `{ trackId, formats }` with immutable ordered TXT/LRC/embedded selections. Subsequent submissions cannot mutate queued batches.

`download_lyrics` returns a string on success, with post-save export failures appended and kept on the existing green success/FOUND log path in `DownloadViewer.vue`. On failure it rejects with a structured `DownloadLyricsError`. Batch downloads use the retry policy and the queue pauses on rate-limit errors; see **Download Queue** below. The shared writers still overwrite targets and delete the opposite sidecar; format-operation success does not imply both files remain.

### Download Queue

- `useDownloader()` processes one track at a time.
- Requests are spaced by **500 ms + 0–250 ms jitter** to stay under LRCLIB's per-IP rate limit.
- On failure, `download_lyrics` rejects with a structured error object (`kind`, `message`, `retryAfterSecs`, `scope`).
- On a rate-limit/server-overload error with `retryAfterSecs`, the queue pauses before the next track and `DownloadViewer.vue` shows a countdown. The Rust client also enforces its own cooldown, so the frontend pause is a UI/queue-level complement rather than the sole guard.

### Edit/Publish Details

`EditLyricsV2.vue` combines CodeMirror plain editing and synced editing with a word timing lane.

- Props/context: `audioSource` (playback source), `lyricsfile` (editing target), `trackId` (save behavior)
- Instrumental mode: toggle via `PlainLyricsEmptyState.vue` / `SyncedLyricsEmptyState.vue`
- Publish/export: handled by `useEditLyricsV2Publish.js` and `useEditLyricsV2Export.js`
- Publish/flag write retries: on explicit 429/503, Rust retries up to 3 times with a fresh challenge/PoW token. The progress table shows `Retrying...` during the wait; the payload is never replayed with the same one-time token.
- Synced lines: multi-line selection via drag and Ctrl/Cmd+click, with floating bulk rewind/forward/delete toolbar
- Synced line nudge shortcuts: `Left`/`Right` adjust selected line start by `-/+100ms`; `Shift+Left`/`Shift+Right` adjust selected line end by `-/+100ms`
- End timestamp visibility: in synced rows, the end timestamp pill stays visible even without hover when it differs from the next line's start timestamp (helps surface gaps/overlaps), and color-codes direction (`before` = gap, `after` = overlap)
- Player bar: playback speed control (`0.5x`-`2.0x`)
- Line status: each synced line row shows a tiny word-sync status dot
- Word timing: multi-separator selection (Ctrl/Cmd+click + Shift+click), merge separators (`Delete`/`Backspace`), hover split preview snapped to grapheme boundaries, double-click split at cursor, and `Z` syncs selected separator then advances (last-word sync advances to next line)
- Narrow segment hint: when a segment is too narrow to show text, a visible hint is rendered beneath it with the next word text
- Boundary sync: can cascade adjacent boundaries so sync is not blocked by intervening separators, while staying within line bounds
- Word boundary chain enforcement: dragging the start of word N (or `Z`-syncing it) also writes `end_ms` on word N-1 in the same update, so the shared boundary stays consistent in the persisted data — not just visually. The dragged word's own `end_ms` is deliberately left alone, so an overshoot that produces `start_ms > end_ms` on a word is observable rather than silently clamped
- Word start-after-end warning: `SyncedWordTimingSegment.vue` renders a small amber `mdi/alert` above any segment whose `word.start_ms > word.end_ms`, with a `word start is after end` title tooltip. Triggers reactively during drag because `displayedWords` propagates the live `start_ms` while `word.end_ms` stays at its stored value
- Reset behavior: clears persisted word timings and reloads default (non-persisted) segmentation
- Line-start mutations (nudge `±100ms`, sync to progress, bulk shift): `setLineStartMs` locks `word[0].start_ms` to the new line start, then forward-cascades successors so each word starts at least `MIN_WORD_DURATION_MS` (10 ms) after its predecessor.
- Line-end mutations (nudge `±100ms`, sync to progress, sync-to-next): `setLineEndMs` mirrors the start logic for the last word only — if `word[lastIdx].end_ms` is explicit (finite), it's locked to the new line end. Implicit ends (the common case, derived by the lane from the next word's `start_ms` or `laneEndMs`) are left untouched.
- Word-lane boundary arrows: `SyncedWordTimingLane.vue` exposes the first word's start (left edge) and last word's end (right edge) as thin primary-color chevron buttons that both nudge and trim. A click expands the line outward and retriggers the spectrogram like the line-row nudges — left emits `rewind-line` (line + `word[0].start` −100 ms, replays), right emits `forward-end` (line + last word end +100 ms). A drag trims the window inward with the boundary + word edge moving together: the left arrow reuses the existing boundary-`0` drag (moves `line.start` + `word[0].start` within `[lineStart, nextWordStart)`); the right arrow uses `useEditLyricsV2LineEndDrag` → `setLineEndMsTo` to move `line.end` (+ explicit last-word end) within `[lastWordStart+1, lineEnd]`. Click vs drag is disambiguated by the shared `DRAG_THRESHOLD`, and the boundary-`0` handle is replaced by the left arrow (`interiorBoundaryIndexes` drops index 0).
- Selection behavior: selecting a synced line starts at the second boundary by default
- Synced line operations: each row has a `sync-end-to-next` button (`arrow-collapse-right` icon) that snaps the current line's `end_ms` to the next line's `start_ms`. `addSyncedLineAt` pre-fills `start_ms` from the previous line's `end_ms` and `end_ms` from the next line's `start_ms`, so inserts drop in pre-timed
- Overlap highlight: `SyncedLyricsEditor.vue` computes `overlappingLineIndexes` — every line that's part of an overlapping pair anywhere in the song gets an amber tint, computed in a single-pass sweep that relies on the sorted-by-`start_ms` invariant
- Stable line identity & reorder: each in-memory synced line carries an integer `id` (generated by `nextLineId()` in `utils/lyricsfile.js`, attached by `normalizeSyncedLine`, stripped before YAML output). The v-for keys by `line.id`, so a `start_ms` mutation that crosses a neighbor re-sorts the array while preserving inline-edit state on the moved row. Selection follows the moved line by id; saves re-attach existing ids by array index so v-for keys don't churn across round-trips
- Spectrogram: when audio is available and the toggle is on, `SpectrogramPanel.vue` renders a colour-mapped log-frequency spectrogram (50 Hz–8 kHz) of the selected line's audio slice above the timeline. Backed by Rust `get_audio_slice` + a frontend STFT (`fft.js` real-input transform). Colour comes from `utils/spectrogram-themes.js`, which samples any d3 `t => colour` interpolator into a 256-entry RGB LUT via one shared builder: `inferno` uses `d3-scale-chromatic`'s accurate matplotlib map, while `blue`/`green` are black-anchored ramps fed through `interpolateRgbBasis` — the same builder d3 uses for its own named ramps — so every theme shares a single code path. d3's stock `Blues`/`Greens` are deliberately unused: they run light→dark with no black anchor, inverting the loudness polarity. Results memoized by `(file_path, start_ms, end_ms)` in a bounded module-level cache; switching theme recolours the cached frames instead of recomputing the transform. The lane grows from `h-[5rem]` to `h-[13rem]` when the spectrogram is shown. The lane header hosts a split pill button: the left half toggles visibility (waveform/eye-off), the right half opens a dropdown of three unlabeled dots (one per theme, each filled with the theme's declared `swatchColor` — both ends of every ramp are near-black/near-white, so a swatch derived from the ramp itself reads as black-to-white and identifies nothing) and reveals the panel on pick. Both preferences persist globally via `useGlobalState().spectrogramVisible`/`.spectrogramTheme` and `config_data.spectrogram_visible`/`.spectrogram_theme`
- Click-to-seek: clicking on the spectrogram or the word-timing timeline emits `seek` and jumps playback to that horizontal position. Clicks are NOT consumed by boundary handles or by word segments whose split preview is currently being rendered. 
- Playback recovery: clicking play on a line after the track has finished now reloads the track with full metadata before seeking, instead of racing a bare-`{trackId}` `play_track` invocation against `seek_track`

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

**Styling**: Tailwind CSS + custom classes in `style.css`. Primary accent palette (`hoa`) from `tailwind.config.cjs`. Structural colors (backgrounds, text, borders) use Tailwind's default `neutral` scale. Dark mode via `html.dark`. Semantic classes: `.button`, `.input`, `.select`, `.modal-content`, `.link`.

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

Tests live next to the source files they exercise (e.g. `word-tokenizer.test.js` for `word-tokenizer.js`). Add tests for any utilities that involve non-trivial branching logic (e.g. parsing, tokenization, transformations). `export-workflows.test.js` verifies shared preferences and batch snapshots with mocked commands. `download-options.test.js` uses a minimal Vue renderer to exercise all four album/artist trigger setups, recycling/unmount lifecycle, target isolation, and scope queries without a DOM or real downloads. DOM component tests are not yet set up.
