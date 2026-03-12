# EditLyrics Refactoring Plan

## Goals

- Reduce the size and responsibility count of `src/components/library/EditLyrics.vue`.
- Separate presentational UI from editor, playback, and persistence logic.
- Remove duplicated timestamp and lint/update logic.
- Make the modal easier to test and safer to extend.

## Current Problems

`src/components/library/EditLyrics.vue` currently owns all of these concerns at once:

- modal orchestration
- toolbar and header rendering
- CodeMirror setup and resize handling
- editor commands for syncing and timestamp shifting
- playback control and seek integration
- lyrics runner state and highlighted line tracking
- lint execution and publish-state UI
- save flow and publish modal setup
- keyboard shortcut registration

This makes the component harder to read because behavior is spread across one large file and several pieces of logic are tightly coupled.

## Target Structure

Keep `src/components/library/EditLyrics.vue` as the modal-level container only.

Move UI into focused child components:

- `src/components/library/edit-lyrics/EditLyricsHeaderActions.vue`
  - Save button
  - Publish button
  - lint/publish status icon + tooltip
- `src/components/library/edit-lyrics/EditLyricsSyncToolbar.vue`
  - sync line actions
  - rewind/forward/repeat controls
  - mark instrumental action
- `src/components/library/edit-lyrics/EditLyricsPlayerBar.vue`
  - play/pause button
  - elapsed/duration labels
  - `Seek` integration
- `src/components/library/edit-lyrics/LyricsCodeEditor.vue`
  - async CodeMirror loading
  - editor ready handling
  - wheel zoom handling
  - font-size controls
  - container resize logic

Move behavior into composables/helpers:

- `src/composables/edit-lyrics/useLyricsDocument.js`
  - `unifiedLyrics`
  - dirty state
  - lint state
  - initial lyrics loading
  - save flow
- `src/composables/edit-lyrics/useLyricsEditorCommands.js`
  - `syncLine`
  - `repeatLine`
  - `markAsInstrumental`
  - shared timestamp shifting helper
- `src/composables/edit-lyrics/useLyricsPlaybackSync.js`
  - `Runner` lifecycle
  - progress watcher
  - current line index
  - editor highlight updates
- `src/composables/edit-lyrics/useLyricsPublish.js`
  - publish mode selection
  - modal setup for synced/plain text publish
- `src/composables/edit-lyrics/useLyricsEditorHotkeys.js`
  - hotkey config
  - document keydown binding/unbinding
- `src/components/library/edit-lyrics/codemirror-line-highlight.js`
  - CodeMirror `StateEffect`
  - CodeMirror `StateField`
  - exported editor extensions

## Refactoring Phases

### Phase 1: Extract pure helpers first

Start with the lowest-risk pieces that do not change template structure.

1. Create a shared helper for recalculating derived lyrics state:
   - parse lyrics
   - rebuild runner input when needed
   - execute both lint passes
2. Create a shared helper for shifting selected timestamps by a delta.
3. Move CodeMirror line highlight field/effect into its own module.
4. Move hotkey registration into `useLyricsEditorHotkeys.js`.

Expected result: smaller script section with no UI changes yet.

### Phase 2: Extract editor/document state

1. Introduce `useLyricsDocument.js`.
2. Move these responsibilities there:
   - initialize `unifiedLyrics` from `editingTrack`
   - maintain `isDirty`
   - maintain `lyricsLintResult`
   - maintain `plainTextLintResult`
   - expose `saveLyrics`
3. Replace direct mutations in `EditLyrics.vue` with composable methods.

Expected result: all document-related state lives behind one API.

### Phase 3: Extract editor commands

1. Introduce `useLyricsEditorCommands.js`.
2. Move command functions there:
   - `syncLine`
   - `repeatLine`
   - `rewind100`
   - `fastForward100`
   - `markAsInstrumental`
3. Replace `rewind100` and `fastForward100` with one helper such as `shiftSelectedTimestamps(delta)`.
4. Keep the composable API view-focused so the modal only wires dependencies.

Expected result: editing actions become isolated and easier to unit test.

### Phase 4: Extract playback sync

1. Introduce `useLyricsPlaybackSync.js`.
2. Move these responsibilities there:
   - runner creation and teardown
   - progress watcher
   - current line index state
   - line highlight dispatches
3. Expose a small API such as:
   - `currentIndex`
   - `refreshRunner`
   - `handleProgressUpdate`

Expected result: the modal no longer mixes player synchronization with editing concerns.

### Phase 5: Extract presentational components

1. Extract `EditLyricsHeaderActions.vue`.
2. Extract `EditLyricsSyncToolbar.vue`.
3. Extract `EditLyricsPlayerBar.vue`.
4. Extract `LyricsCodeEditor.vue`.

Guidelines for child components:

- pass plain props and callbacks instead of large shared objects
- keep business logic in composables, not in templates
- emit narrow events such as `save`, `publish`, `sync-line`, `zoom-in`

Expected result: the parent template becomes a readable composition of 3-4 blocks.

### Phase 6: Simplify publish flow

1. Introduce `useLyricsPublish.js`.
2. Hide the two `useModal` setups behind one function.
3. Expose a single `publishLyrics()` method that chooses synced vs plain text mode.
4. Add computed values for publish UI state:
   - `publishMode`
   - `publishTooltip`
   - `publishStatusIcon`
   - `canPublish`

Expected result: publish behavior is easier to follow and template branching is reduced.

## Proposed End State For `EditLyrics.vue`

The parent component should mostly do the following:

1. get `editingTrack`
2. initialize document/publish/playback composables
3. hold the shared editor `view` reference
4. connect child component events to composable methods
5. mount/unmount modal-specific side effects

If this refactor goes well, `EditLyrics.vue` should drop from 600+ LOC to roughly 120-200 LOC.

## Suggested Order Of Implementation

Use this order to minimize risk and keep the component working after each step:

1. extract CodeMirror highlight module
2. extract hotkey composable
3. extract timestamp shift helper
4. extract document composable
5. extract editor commands composable
6. extract playback sync composable
7. extract header/toolbar/player/editor child components
8. extract publish composable and polish computed UI state

## Validation Checklist

After each phase, verify:

- the modal still opens and closes correctly
- initial lyrics load correctly for both synced and plain text tracks
- play/pause/seek still work
- sync line and repeat line still work
- rewind/forward timestamp actions still work on single and multiple selected lines
- current lyric line highlighting still updates during playback
- save still persists lyrics correctly
- publish still opens the correct modal for synced vs plain text lyrics
- all hotkeys still work
- zoom and resize behavior still work

## Nice-To-Have Follow-Ups

- add unit tests for pure helpers such as timestamp shifting and publish-mode detection
- add component tests around child component events
- consider TypeScript for the new composables if this area keeps growing
- centralize lyric formatting helpers if similar logic exists elsewhere
