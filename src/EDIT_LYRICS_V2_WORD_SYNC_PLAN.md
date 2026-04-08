# EditLyricsV2 Word Sync Plan

## Overview

Add a fixed horizontal word timing lane at the top of the synced lyrics lines container. This enables drag-based word timing adjustments for the currently selected line, with CJK support and even distribution.

---

## UX Design

### Fixed Lane Behavior

- The word timing lane is fixed at the **top** of the synced lyrics lines container (just below the player progress bar)
- The lane displays word timing for the **currently selected line** only
- When no line is selected, the lane shows an empty/placeholder state
- Selecting a different line updates the lane content to show that line's words
- The lane position is stable - no layout shift occurs when changing selections
- The lane shows a short placeholder reason when word sync is unavailable for the selected line

### Visual Layout

```
[Player Progress Bar]

[Word Timing Lane]     ← fixed position, always here
  [Line info header: timestamps, text preview, Play/Sync word/Redistribute buttons]
  [Timeline with grid lines every 500ms]
────────────────────
[Previous lines]

[Selected Line Row]  ← currently selected line

[Next lines]
```

### Lane Header

When word sync is available, the lane header displays:
- **Timestamp range**: `line.start_ms - next_line.start_ms` (e.g., `00:23.500 - 00:26.800`)
- **Line text preview**: truncated text of the selected line
- **Play button**: replays the current line from its start time
- **Sync word button**: syncs the selected boundary to current playback position (keyboard: `z`)
- **Redistribute button**: distributes word timings evenly across the line window

---

## Core Features

### 1. Drag Behavior

- Words render as draggable horizontal segments in the timeline that fill the space until the next word
- Timeline range: `[line.start_ms, next_line.start_ms]`
- **Last line fallback**: If no next line exists, the timeline extends to `line.start_ms + 2000ms`
- Word sync is only enabled when **next line is also synced** (`next_line.start_ms` exists)
- **First word**: `words[0].start_ms` always equals `line.start_ms` (fixed, not draggable)
- **Last word**: implicitly extends to `next_line.start_ms`
- **Middle words**: Drag the handle between words to adjust the `start_ms` of the word to the right
  - The handle is positioned at the boundary between two consecutive words
  - Dragging moves the boundary, effectively changing the `start_ms` of the right word
  - The left word automatically extends/shrinks to fill the available space
- Constraints:
  - Boundary positions must be monotonically non-decreasing
  - First word cannot be moved (fixed to line start)
  - Last word's end is implicitly determined by the line end
  - No handle before the first word or after the last word
  - Minimum 1ms gap between consecutive word start times
- Visual feedback during drag with ghost/overlay showing current position and timestamp badge
- Changes apply immediately on pointer release

### 1.1 Boundary Selection

- Each boundary handle (between words) can be individually selected
- Selected boundary shows a highlighted visual state (cyan)
- Clicking a boundary selects it without dragging
- Only one boundary can be selected at a time
- Default selection: first boundary (between word 0 and word 1) when line changes
- Selected boundary is used for **Sync word** action

### 2. Tokens

- If `line.words[]` exists and is valid (words array matches line text) → use existing
- If missing or invalid (doesn't match line text) → auto-regenerate from `line.text`:
  - **Latin scripts**: space-delimited, preserve trailing spaces in word tokens
  - **CJK (Chinese/Japanese/Korean)**: treat each character as a token (no spaces needed)
- Initial timing: evenly distributed across the line time window

### 2.1 Word Validation

Words are considered valid only if:
1. `line.words` exists and is a non-empty array
2. The reconstructed text (`words.map(w => w.text).join('')`) exactly matches `line.text`

If words exist but are missing `start_ms` values, timings are redistributed evenly.

### 2.2 Availability Rules

Word sync is available only when all conditions are true:

- The selected line has non-empty text content
- The selected line has a valid `start_ms`
- The next line exists and has a valid `start_ms`

When unavailable, the lane stays visible and shows a short placeholder reason (no content, missing current sync, or missing next-line sync).

### 2.3 Data Consistency Rules

- If line text changes, clear `line.words` for that line
- If line `start_ms` changes, offset every `words[].start_ms` by the same actual delta
- If next line `start_ms` changes, previous line words do **not** need mutation (their implicit end updates from timeline context)

### 3. Distribute Evenly

- Single-click action to reset all word timings to equal spacing across the line window
- Useful for quick prototyping or resetting after manual edits
- Preserves word tokens, only adjusts `start_ms` values
- After redistributing, selected boundary resets to first boundary
- Triggers auto-replay from line start

### 4. Sync Word to Playback Position

- Snaps the **selected boundary** (word start) to the current playback position
- Useful for "singing along" to set precise timings
- Keyboard shortcut: `z` key
- After syncing, automatically advances selection to the next boundary (word)
- If at the last boundary, pressing `z` stays on that boundary
- Does **not** trigger auto-replay (unlike drag operations)

### 5. Sync Word + Advance to Next Line

- Keyboard shortcut: `x` key
- Syncs the selected boundary to current playback position
- If currently on the **last boundary** (between second-to-last and last word), automatically advances to the next lyric line after syncing
- This allows efficient sequential word timing: tap `x` repeatedly to sync words, and when reaching the last word, it automatically moves to the next line

### 6. Playback Feedback

**Timeline Grid Lines**
- Vertical grid lines appear every 500ms across the timeline
- Provides visual reference for timing precision

**Progress Indicator**
- A vertical playhead indicator moves across the timeline as the song plays
- The indicator shows the current playback position relative to the selected line's time window
- A small circle at the top of the indicator provides a visual anchor point
- Only visible when playback is within the selected line's time window

**Word Highlighting**
- The currently playing word becomes **bold** and receives a **slight background color change**
- This provides immediate visual feedback when timing adjustments are correct
- The active word is determined by: `word.start_ms <= currentTime < next_word.start_ms`

**Auto-Replay After Edit**
- After completing a drag-drop action on a word boundary, the song automatically replays from `line.start_ms`
- This allows instant verification of the timing adjustment
- The playback resumes from the beginning of the current line to hear the word in context
- Also triggered after "Distribute evenly"

---

## Data Model

```javascript
// Line structure (matches LYRICSFILE_CONCEPT.md)
{
  text: "Waiting in a car",
  start_ms: 23500,
  end_ms: 26800,
  words: [
    { text: "Waiting ", start_ms: 23500 },
    { text: "in ", start_ms: 24500 },
    { text: "a ", start_ms: 25200 },
    { text: "car", start_ms: 25800 }
  ]
}
```

### Constraints

- `words[].start_ms` must be monotonically non-decreasing
- `words[0].start_ms === line.start_ms` (first word always starts at line start)
- For middle words: `word[i].start_ms >= word[i-1].start_ms` (minimum 1ms gap to allow handles)
- Implicit `end_ms` for each word is `next_word.start_ms` (or selected line window end)
- All timestamps in milliseconds (integers)

---

## Components

1. **SyncedWordTimingLane.vue**
   - Fixed horizontal timeline container positioned at top of lyrics list
   - Playhead indicator synced to current playback
   - Renders word segments only when availability rules are satisfied
   - Timeline grid lines every 500ms
   - Lane header with timestamp range, line text preview, Play/Sync word/Redistribute buttons
   - Boundary handles with selection state (cyan highlight when selected)
   - Drag overlay with timestamp badge showing current position during drag
   - Empty/placeholder states with reason text when word sync is unavailable
   - Keyboard shortcuts: `z` (sync word to playback), `x` (sync and advance to next line)
   - Handles word timing updates and emits `word-timing-edited` for auto-replay

2. **SyncedWordTimingSegment.vue**
   - Word block positioned absolutely on the timeline based on start_ms/end_ms
   - Shows word text with horizontal fill until next word
   - Visual state: default / currently playing (bold + highlighted background)
   - Minimum width based on text length
   - Tooltip shows word timing: `text (start_ms - end_ms)`

---

## Integration Points

- **SyncedLyricsEditor.vue**: Mount lane as fixed header above the scrollable lyrics list
- **word-tokenizer.js**: Tokenization utilities (Latin + CJK), word validation, distribute evenly
- **useEditLyricsV2Document.js**: Keep word timing consistent when line text/start time changes
- **EditLyricsV2.vue**: Wire playback progress to lane for playhead sync, pass selected line to lane, handle `word-timing-edited` event for auto-replay
