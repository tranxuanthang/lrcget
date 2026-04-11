# LRCLIB `lyricsfile` Support Plan

## Goal

Add first-class support for LRCLIB responses that include `lyricsfile` (from `/api/get` and `/api/search`) so that:

1. When LRCLIB returns `lyricsfile`, we persist that exact payload into `lyricsfiles`.
2. When LRCLIB returns only `plainLyrics`/`syncedLyrics` (or instrumental marker), we keep the current fallback behavior: generate a Lyricsfile and persist it.
3. Keep `tracks.instrumental` as-is, and move lyrics filtering logic to explicit boolean flags derived from Lyricsfile content.

## Current State (review)

- `lyricsfiles` persistence already exists via `db::upsert_lyricsfile_for_track` and `db::upsert_lyricsfile_for_track_tx` in `src-tauri/src/db.rs`.
- Fallback Lyricsfile generation already exists via `persist_lyricsfile_for_track` in `src-tauri/src/main.rs`, which calls `lyricsfile::build_lyricsfile`.
- Library scan/import path already generates Lyricsfile from local `.txt/.lrc` when available (`src-tauri/src/db.rs:444`, `src-tauri/src/scanner/scan.rs:280`).
- `save_lyrics` already supports a provided Lyricsfile payload and persists it as-is (`src-tauri/src/main.rs:630`).
- LRCLIB models currently ignore `lyricsfile`:
  - `src-tauri/src/lrclib/get.rs` `RawResponse` has only `plain_lyrics` / `synced_lyrics` + metadata.
  - `src-tauri/src/lrclib/search.rs` `SearchItem` has only `plain_lyrics` / `synced_lyrics` + metadata.
  - `src-tauri/src/lrclib/get_by_id.rs` `RawResponse` also lacks `lyricsfile`.
- Download/apply flows currently decide behavior only from `Response::from_raw_response(...)` (`SyncedLyrics` / `UnsyncedLyrics` / `IsInstrumental` / `None`), so Lyricsfile-only LRCLIB payloads are effectively treated as missing.
- Library filtering currently depends on legacy `tracks.txt_lyrics` / `tracks.lrc_lyrics` checks in:
  - `src-tauri/src/db.rs:get_track_ids`
  - `src-tauri/src/db.rs:get_search_track_ids`
  - `src-tauri/src/db.rs:get_album_track_ids`
  - `src-tauri/src/db.rs:get_artist_track_ids`
  This will become incorrect once Lyricsfile-only records are common.

## Implementation Plan

### 1) Extend LRCLIB DTOs to carry `lyricsfile`

- Update response structs:
  - `src-tauri/src/lrclib/get.rs::RawResponse`
  - `src-tauri/src/lrclib/search.rs::SearchItem`
  - `src-tauri/src/lrclib/get_by_id.rs::RawResponse`
- Add `lyricsfile: Option<String>` with serde mapping compatible with LRCLIB JSON field name (`lyricsfile`).
- Keep existing fields (`plain_lyrics`, `synced_lyrics`) untouched for backward compatibility.

### 2) Introduce a single resolver for apply/download payloads

- Add a helper in backend (likely in `main.rs` or a small helper module) that resolves an incoming LRCLIB payload into:
  - `plain_lyrics` (String)
  - `synced_lyrics` (String)
  - `is_instrumental` (bool)
  - `lyricsfile_to_store` (Option<String>)
- Resolution precedence:
  1. If `lyricsfile` exists and is non-empty:
     - Parse with `lyricsfile::parse_lyricsfile`.
     - Use parsed lyrics to drive file write + `tracks` updates.
     - Store the original `lyricsfile` payload in `lyricsfiles` (as-is) via upsert.
  2. Else fallback to existing plain/synced/instrumental behavior.
     - Use `persist_lyricsfile_for_track` to generate/store Lyricsfile.

Rationale: one resolver avoids drift between `download_lyrics` and `apply_lyrics` and ensures identical precedence rules.

### 3) Wire resolver into mass download flow (`download_lyrics`)

- Refactor command path in `src-tauri/src/main.rs:384` to support LRCLIB `lyricsfile`:
  - Keep writing sidecar/embed behavior through existing `lyrics` functions.
  - Keep updating `tracks.txt_lyrics` / `tracks.lrc_lyrics` / `instrumental` exactly as today.
  - Change Lyricsfile persistence branch:
    - If LRCLIB provided `lyricsfile`, upsert that exact payload.
    - Otherwise call `persist_lyricsfile_for_track` (current behavior).

### 4) Wire resolver into apply-from-search flow (`apply_lyrics`)

- Refactor command path in `src-tauri/src/main.rs:435`:
  - Accept search item payloads that may contain only `lyricsfile`.
  - Reuse the same resolver from step 2.
  - Persist provided Lyricsfile when present; fallback-generate otherwise.

### 5) Ensure search results transport `lyricsfile` end-to-end

- With updated `SearchItem`, `search_lyrics` response will include `lyricsfile` to frontend.
- Frontend `SearchLyrics.vue` already forwards the selected item as `lrclibResponse` to `apply_lyrics`; no protocol redesign needed.

### 6) Frontend UI updates for `lyricsfile`-first LRCLIB results

- Update `src/components/library/SearchLyrics.vue`:
  - Render status/line-count badges correctly when a result has only `lyricsfile` (no direct `plainLyrics`/`syncedLyrics`).
  - Derive previewable plain/synced state from `lyricsfile` via `parseLyricsfile` when needed.
- Update `src/components/library/search-lyrics/Preview.vue`:
  - Support preview when LRCLIB item has only `lyricsfile` by parsing into plain/synced before deciding synced-vs-plain view.
  - Keep existing behavior unchanged when `plainLyrics`/`syncedLyrics` are present.
- Update `src/components/library/my-lrclib/SearchResult.vue`:
  - Show correct badges/line counts for `lyricsfile`-only items.
  - Keep list semantics consistent with `SearchLyrics.vue`.
- Update `src/components/library/my-lrclib/PreviewLyrics.vue` and `src/components/library/my-lrclib/EditLyrics.vue`:
  - When fetched LRCLIB-by-id payload includes `lyricsfile`, derive editable/preview text from parsed Lyricsfile instead of assuming plain/synced fields exist.
- Optional shared helper (recommended): add a small frontend utility normalizer (e.g. in `src/utils/lyricsfile.js` or a nearby helper) that converts an LRCLIB item into a unified shape (`plainLyrics`, `syncedLyrics`, `isInstrumental`, `hasLyricsfile`) to avoid duplicating parsing logic across components.

### 7) Add derived lyrics-presence flags on `tracks` and use them for filtering

- Add new boolean columns on `tracks` (via new migration):
  - `has_plain_lyrics`
  - `has_synced_lyrics`
  - `has_word_synced_lyrics`
- Add indexes for these columns to keep filtering fast (individual indexes are fine; composite can be evaluated after profiling).
- Keep `tracks.instrumental` behavior unchanged.
- Keep `tracks.txt_lyrics` / `tracks.lrc_lyrics` for compatibility, but stop using them as source-of-truth for filtering.
- Derive these booleans from persisted Lyricsfile content whenever lyrics state changes:
  - LRCLIB download/apply
  - Manual save/edit (`save_lyrics`)
  - Scanner/import path (when Lyricsfile is generated from sidecar/local metadata)
- Define derivation rules from Lyricsfile parse result:
  - `has_plain_lyrics = true` when non-empty plain lyrics exist
  - `has_synced_lyrics = true` when synced lyrics exist (including word-synced)
  - `has_word_synced_lyrics = true` when at least one synced line contains timing for words
  - `instrumental` remains independent and unchanged

### 8) Switch backend filter SQL to new flags

- Update these DB query builders to use the new booleans (+ `instrumental`) instead of `txt_lyrics`/`lrc_lyrics` null checks:
  - `get_track_ids`
  - `get_search_track_ids`
  - `get_album_track_ids`
  - `get_artist_track_ids`
- Preserve existing UI behavior/meaning of current filters (synced/plain/instrumental/no-lyrics), but backed by explicit booleans.
- Ensure `no_lyrics` semantics exclude instrumental tracks, same as current behavior.

## Edge Cases to Handle Explicitly

- `lyricsfile` present but invalid YAML: return a clear error (do not silently discard).
- `lyricsfile` present and valid but yields empty plain/synced while non-instrumental: treat as no usable lyrics and return not found-style error.
- `lyricsfile` + plain/synced both present but conflicting: trust `lyricsfile` as source of truth for persistence and applied content.
- Instrumental encoded in Lyricsfile metadata: ensure `tracks.instrumental = true` and sidecar logic remains consistent.

## Verification Checklist

1. `/api/get` returns `lyricsfile` only -> `download_lyrics` stores exact payload in `lyricsfiles`, updates track lyrics/instrumental derived from parsed Lyricsfile.
2. `/api/get` returns plain/synced only -> fallback-generated Lyricsfile is stored (existing behavior preserved).
3. `/api/search` item contains `lyricsfile` only -> selecting "Apply" works and stores payload in `lyricsfiles`.
4. `/api/search` item contains plain/synced only -> existing apply flow remains unchanged.
5. Search result list badges/line-counts are correct for `lyricsfile`-only items.
6. Lyrics preview UI works for `lyricsfile`-only items (both search modal and MyLRCLIB flows).
7. Migration creates `has_plain_lyrics`, `has_synced_lyrics`, `has_word_synced_lyrics` with indexes.
8. Existing rows are backfilled correctly from available Lyricsfile data (and safe fallback for missing/invalid payloads).
9. Track/album/artist/library filters return correct results when lyrics exist only in Lyricsfile.
10. Existing scanner/import behavior remains unchanged.
11. Run `cargo check` in `src-tauri/`.

## Suggested Execution Order (small PRs)

1. DTO updates (`get.rs`, `search.rs`, `get_by_id.rs`) + compile.
2. Shared resolver helper + unit-level parsing branches.
3. Integrate `download_lyrics`.
4. Integrate `apply_lyrics`.
5. Add migration + backfill for lyrics-presence booleans on `tracks`.
6. Update all filter SQL paths to use new booleans.
7. Frontend normalization + UI updates for search/preview/edit components.
8. `cargo check` and manual verification.
