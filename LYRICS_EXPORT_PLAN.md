# Lyrics Export Feature Implementation Plan

## Overview

Change LRCGET's lyrics storage model from "auto-export to sidecar files" to "database-first with optional/manual export".

### Current Behavior

- Lyrics are automatically imported from `.txt`/`.lrc` files during library scan
- Lyrics are automatically saved to `.txt`/`.lrc` files when downloaded, searched, or edited
- Lyrics are stored in both `tracks` table (`txt_lyrics`, `lrc_lyrics`) and `lyricsfiles` table

### Target Behavior

- Lyrics continue to be imported from `.txt`/`.lrc` files during scan (into `lyricsfiles` table)
- Lyrics are **only** saved to database (no automatic file writes)
- Users manually export lyrics when desired via UI actions
- Export supports: `.lrc`, `.txt`, and embedded metadata

## Phase 1: Backend Changes

### 1.1 Database & Configuration

**Migration:** Add export configuration columns to `config_data` table

- `export_format_lrc: boolean` (default: false)
- `export_format_txt: boolean` (default: false)
- `export_embed: boolean` (default: false)

**Update:**

- `PersistentConfig` struct to include new fields
- `get_config()` and `set_config()` functions
- Add functions to query tracks with lyrics for mass export

### 1.2 Export Module

**Create:** `src-tauri/src/export.rs`

Core functionality:

- `export_lyrics_for_track()` - Export single track to specified formats
- `export_all_lyrics()` - Mass export with progress callbacks
- Format generators for `.lrc`, `.txt`
- Embedding support for MP3/FLAC metadata

### 1.3 Remove Auto-Export

**Modify:** `src-tauri/src/lyrics.rs`

- Remove automatic file-writing functions
- Move path-building utilities to export module
- Keep embed functions for export module use

**Update Commands:**

- `download_lyrics()` - Only save to DB + lyricsfiles table
- `apply_lyrics()` - Only save to DB + lyricsfiles table
- `save_lyrics()` - Only save to DB + lyricsfiles table

### 1.4 New Commands

**Add to `src-tauri/src/main.rs`:**

- `export_lyrics(track_id, formats)` - Export single track
- `export_all_lyrics(formats)` - Mass export all tracks with lyrics
- `get_export_config()` - Get current export settings
- `set_export_config(...)` - Update export settings

**Add Events:**

- `export-progress` - Real-time progress updates
- `export-complete` - Export finished notification

## Phase 2: Frontend Changes

### 2.1 Config UI

**Update:** `src/components/Config.vue`

Add "Lyrics Export Settings" section with:

- Format checkboxes: `.lrc`, `.txt`, embedded
- Help text explaining each format

### 2.2 Export Composable

**Create:** `src/composables/export.js`

Exports:

- `exportTrack()` - Single track export
- `exportAllTracks()` - Mass export with progress
- `getExportConfig()` / `setExportConfig()`
- `useExport()` composable

### 2.3 Export Viewer Modal

**Create:** `src/components/ExportViewer.vue`

Features:

- Format selection UI
- Progress display with progress bar
- Current track name display
- Summary view (exported, skipped, errors)

### 2.4 Track-Level Export

**Update:** Track row component

Add export icon button:

- Visible only for tracks with lyrics
- Uses configured formats or prompts for selection
- Shows success/error notification

### 2.5 Mass Export

**Update:** `src/components/library/Library.vue`

Add "Export All Lyrics" button to toolbar:

- Opens ExportViewer modal
- Exports all tracks that have lyrics in database
- Shows progress and summary

## Phase 3: Documentation

Update architecture docs:

- `src/ARCHITECTURE.md` - Frontend export flow
- `src-tauri/ARCHITECTURE.md` - Backend export module, commands, events

Note deprecated `txt_lyrics`/`lrc_lyrics` columns (no longer written to).

## Implementation Sprints

### Sprint 1: Core Infrastructure

- Database migration
- Config schema updates
- Create export module
- Add new commands

### Sprint 2: Remove Auto-Export

- Remove file-writing from lyrics module
- Update download/apply/save commands
- Test: Only DB writes, no file writes

### Sprint 3: Frontend UI

- Config settings UI
- Export composable
- ExportViewer modal
- Track export button
- Mass export button

### Sprint 4: Integration & Polish

- Event listeners
- Documentation updates
- Testing

## Design Decisions

### Export Location

- Individual track export: Same directory as track file
- Mass export: Each track's own directory (custom directory for future)

### Conflict Handling

- Overwrite existing files silently
- No backup creation

### Backward Compatibility

- Existing `.lrc`/`.txt` files left as-is
- Import from files continues to work
- No automatic deletion of existing files
- Deprecated columns remain for read-only legacy support

## Future Enhancements (Out of Scope)

- Custom export directories
- Filename templates/patterns
- Export filters (e.g., only tracks without existing .lrc)
- "Unexport" (remove sidecar files)
- Export to cloud storage

---

## TODO

- [ ] Sprint 1: Database migration (11-add-export-config)
- [ ] Sprint 1: Update PersistentConfig struct
- [ ] Sprint 1: Update get_config/set_config functions
- [ ] Sprint 1: Create export.rs module
- [ ] Sprint 1: Add export commands to main.rs
- [ ] Sprint 2: Remove file-writing from lyrics.rs
- [ ] Sprint 2: Update download_lyrics command
- [ ] Sprint 2: Update apply_lyrics command
- [ ] Sprint 2: Update save_lyrics command
- [ ] Sprint 3: Create export.js composable
- [ ] Sprint 3: Update Config.vue with export settings
- [ ] Sprint 3: Create ExportViewer.vue modal
- [ ] Sprint 3: Add export button to track rows
- [ ] Sprint 3: Add mass export button to Library
- [ ] Sprint 4: Add event listeners for export progress
- [ ] Sprint 4: Update ARCHITECTURE.md files
- [ ] Sprint 4: Testing and bug fixes

## Implementation History

| Date       | Sprint | Description                 |
| ---------- | ------ | --------------------------- |
| 2026-04-12 | -      | Created implementation plan |
