# EditLyricsV2 Publish Plan

## Goal

Add a publish flow to `EditLyricsV2` that is built specifically for the V2 editor and publishes Lyricsfile content to LRCLIB.

This plan intentionally does **not** reuse UI components or publish logic from the legacy `EditLyrics` flow.

## Scope

- Add publish affordance to the `EditLyricsV2` header.
- Add a V2-specific publish dropdown using FloatingVue `lrcget-dropdown`.
- Add a V2-specific publish modal for the `Save and Publish` action.
- Update the backend publish command and LRCLIB request payload so `lyricsfile` can be sent directly.
- Keep the existing `Debug` button.
- Do not implement linting or validation rules for this feature yet.

## Non-Goals

- Do not refactor or migrate the old `EditLyrics` publish flow.
- Do not attempt to share publish UI or composables with the legacy editor.
- Do not add Lyricsfile linting.
- Do not implement actual export-to-directory behavior in this phase unless explicitly included during implementation.

## Header UI

Target header layout:

```text
[Save][⌄] [Debug] ... track name ... [Plain/Synced] [x]
```

### Header behavior

- `Save` remains the primary save action and keeps the current `Ctrl+S` behavior.
- `⌄` opens a V2-specific dropdown menu using `VDropdown` with theme `lrcget-dropdown`.
- `Debug` remains visible and unchanged.
- `Plain/Synced` tab switcher remains on the right.
- Close button remains provided by `BaseModal`.

## Dropdown UI

Clicking the `⌄` button opens a dropdown with this content:

```text
[Save and Publish]

Export to directory:

[ ] Plain lyrics (.txt)
[ ] Synced lyrics (.lrc)
[ ] Enhanced LRC lyrics (.elrc)
```

### Dropdown behavior

- `Save and Publish` should be a top-level action in the dropdown.
- The export section should be present in the UI now, even if its actions are not wired yet.
- The export options should be rendered as selectable items, most likely checkboxes or checkbox-like menu rows.
- The dropdown should follow the same `lrcget-dropdown` pattern already used elsewhere in the app.

## Publish Data Model

`EditLyricsV2` should publish the serialized Lyricsfile as the primary payload.

Source of truth:

- `plainLyrics`
- `syncedLines`
- `lyricsfileDocument`
- `editingTrack`

Derived publish payload:

- `lyricsfile = serializeLyricsfile({ track, plainLyrics, syncedLines, baseDocument })`

### Publish rule

- If `serializeLyricsfile(...)` returns content, publish that content via the `lyricsfile` parameter.
- Do not require `plainLyrics` or `syncedLyrics` when `lyricsfile` is present.
- Legacy frontend callers should remain supported by the backend.

## Frontend Implementation Plan

### 1. Update V2 header component

File:

- `src/components/library/edit-lyrics-v2/EditLyricsV2HeaderActions.vue`

Changes:

- Replace the current two-button header with a split save control:
  - left button: `Save`
  - right button: dropdown trigger `⌄`
- Keep `Debug` as a separate button.
- Emit a new event for the dropdown primary action if needed.
- Add a `VDropdown theme="lrcget-dropdown"` menu to the dropdown trigger.

### 2. Add V2-specific publish modal

Suggested new file:

- `src/components/library/edit-lyrics-v2/EditLyricsV2PublishModal.vue`

Responsibilities:

- Confirm publishing action.
- Show publish progress using the existing backend `publish-lyrics-progress` event.
- Invoke `publish_lyrics` with `lyricsfile`.
- Show success/error toast.
- Stay isolated from legacy publish modal code.

### 3. Add V2-specific publish composable

Suggested new file:

- `src/composables/edit-lyrics-v2/useEditLyricsV2Publish.js`

Responsibilities:

- Build the current Lyricsfile payload from V2 state.
- Open the V2 publish modal.
- Keep all V2 publish-specific state and modal wiring out of `EditLyricsV2.vue` where possible.

Inputs:

- `editingTrack`
- `plainLyrics`
- `syncedLines`
- `lyricsfileDocument`
- `saveLyrics`

Suggested actions:

- `openPublishModal()`
- `saveAndPublish()` or modal-driven equivalent

### 4. Wire header and publish flow in `EditLyricsV2.vue`

File:

- `src/components/library/EditLyricsV2.vue`

Changes:

- Pass the new publish/dropdown handlers into `EditLyricsV2HeaderActions`.
- Keep current save and debug behavior intact.
- Use the V2 publish composable.
- Make sure the publish flow uses the same current document state as the debug YAML view.

### 5. Save-before-publish behavior

Recommended behavior:

- `Save and Publish` should first persist the current Lyricsfile via `saveLyrics()`.
- If save succeeds, then open or continue the publish step.
- If save fails, abort publish.

Rationale:

- Keeps local storage and published content in sync.
- Matches the user-facing wording of the action.

## Backend Implementation Plan

### 6. Extend `publish_lyrics` command

File:

- `src-tauri/src/main.rs`

Changes:

- Update `publish_lyrics` to accept optional payload fields:
  - `plain_lyrics: Option<String>`
  - `synced_lyrics: Option<String>`
  - `lyricsfile: Option<String>`
- Normalize empty strings to `None`.
- Permit publish when `lyricsfile` is present even if the other two fields are absent.
- Keep backward compatibility for old callers that still send plain/synced lyrics.

### 7. Extend LRCLIB publish request body

File:

- `src-tauri/src/lrclib/publish.rs`

Changes:

- Make request fields optional.
- Add `lyricsfile: Option<String>`.
- Use serde omission for absent fields.
- Preserve current title/album/artist/duration payload.

## Export Section Plan

The dropdown includes export entries now, but the export feature should be treated as separate from publish.

Phase 1 for this plan:

- Render the export section in the dropdown UI.
- Keep state local to `EditLyricsV2` or its header component.
- Do not wire filesystem export yet unless explicitly requested during implementation.

Future export expectations:

- `.txt` should use plain lyrics text.
- `.lrc` should use line-synced LRC output.
- `.elrc` should use enhanced output derived from word timings.

## Suggested File List

Frontend:

- `src/components/library/EditLyricsV2.vue`
- `src/components/library/edit-lyrics-v2/EditLyricsV2HeaderActions.vue`
- `src/components/library/edit-lyrics-v2/EditLyricsV2PublishModal.vue` (new)
- `src/composables/edit-lyrics-v2/useEditLyricsV2Publish.js` (new)

Backend:

- `src-tauri/src/main.rs`
- `src-tauri/src/lrclib/publish.rs`

Docs:

- `src/ARCHITECTURE.md`
- `src-tauri/ARCHITECTURE.md`

## Implementation Order

1. Add backend support for optional `lyricsfile` publish payloads.
2. Add a V2-specific publish modal.
3. Add a V2-specific publish composable.
4. Update the V2 header to the new split-save dropdown design.
5. Wire `Save and Publish` through save, then publish.
6. Render the export section in the dropdown.
7. Run `cargo check` in `src-tauri/`.
8. Update architecture docs.

## Open Questions For Implementation

1. Should `Save and Publish` immediately publish after a successful save, or should it open a confirmation modal first?
2. Should the export checkboxes be persistent selections, or are they placeholders until export is implemented?
3. Should `.elrc` be emitted as a true enhanced-LRC text format from `words[]`, or should it map to Lyricsfile-derived timing output with a custom serializer?

## Recommendation

For the first implementation pass:

- `Save and Publish` should save first, then open a confirmation/progress modal.
- Export entries should be visible but not functional yet.
- V2 should publish only through `lyricsfile`, without introducing any V2 linting gate.
