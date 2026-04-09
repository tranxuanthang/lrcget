# EditLyricsV2 Word Sync Plan

## Overview & Goals

Fixed horizontal word timing lane at top of synced lyrics container for drag-based word timing adjustments with CJK support and even distribution. Enables precise per-word synchronization for selected lyric line.

## UX/UI Design

**Fixed Lane** positioned below player progress bar, above lyrics list. Shows word timing for **currently selected line** only. Stable layout without shifts when changing selections.

**Layout:**
```
[Player Progress Bar]
[Word Timing Lane]  ← fixed
  [Header: timestamps, text preview, Play/Sync/Redistribute buttons]
  [Timeline with 500ms grid lines]
────────────────────
[Previous lines]
[Selected Line Row]
[Next lines]
```

**Lane Header** shows when word sync available:
- Timestamp range: `line.start_ms - next_line.start_ms`
- Line text preview (truncated)
- **Play**: replay current line from start
- **Sync word**: sync selected boundary to playback position (key `z`)
- **Redistribute**: evenly distribute word timings

## Core Features

### 1. Drag Behavior
- Words render as draggable horizontal segments in timeline range `[line.start_ms, next_line.start_ms]`
- Last line fallback: timeline extends to `line.start_ms + 2000ms` if no next line
- **First word**: `words[0].start_ms` = `line.start_ms` (fixed, not draggable)
- **Last word**: implicitly extends to line end
- **Middle words**: Drag handle between words to adjust `start_ms` of right word
- Constraints: monotonic non-decreasing, min 1ms gap, first word fixed
- Visual feedback during drag with timestamp badge
- Changes apply on pointer release

### 2. Boundary Selection
- Each boundary handle (between words) individually selectable
- Selected boundary highlighted cyan
- Default: first boundary when line changes
- Used for **Sync word** action

### 3. Tokens & Validation
- If `line.words[]` exists and matches line text → use existing
- If missing/invalid → auto‑regenerate from `line.text`:
  - **Latin scripts**: space‑delimited, preserve trailing spaces
  - **CJK**: each character as token
- Initial timing: even distribution across line window
- **Validation**: words valid only if array exists and reconstructed text matches `line.text`
- If missing `start_ms` values → redistribute evenly

### 4. Availability Rules
Word sync available when all true:
1. Selected line has non‑empty text
2. Selected line has valid `start_ms`
3. Next line exists with valid `start_ms`

When unavailable: lane shows placeholder reason.

### 5. Data Consistency Rules
- Line text changes → clear `line.words` for that line
- Line `start_ms` changes → offset all `words[].start_ms` by same delta
- Next line `start_ms` changes → no mutation needed (implicit end updates)

### 6. Actions
- **Distribute evenly**: single‑click reset all word timings to equal spacing
- **Sync word to playback**: snap selected boundary to current playback position (key `z`)
- **Sync word + advance**: if on last boundary, sync and advance to next lyric line (key `x`)

### 7. Playback Feedback
- **Timeline grid lines**: vertical lines every 500ms
- **Progress indicator**: vertical playhead moves across timeline
- **Word highlighting**: currently playing word becomes bold with background color
- **Auto‑replay after edit**: after drag or distribute, replay from `line.start_ms` for verification

## Data Model & Constraints

```javascript
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

**Constraints:**
- `words[].start_ms` monotonically non‑decreasing
- `words[0].start_ms === line.start_ms`
- Minimum 1ms gap between consecutive word start times
- Implicit `end_ms` for each word = `next_word.start_ms` (or line window end)
- All timestamps in milliseconds (integers)

## Components & Integration

| Component | Purpose |
|-----------|---------|
| **SyncedWordTimingLane.vue** | Fixed horizontal timeline container, playhead sync, word segments, lane header, boundary handles, drag overlay, keyboard shortcuts (`z`/`x`), empty states |
| **SyncedWordTimingSegment.vue** | Word block positioned absolutely on timeline, shows word text with horizontal fill, visual states, tooltip with timing |

**Integration points:**
- **SyncedLyricsEditor.vue**: Mount lane as fixed header above scrollable lyrics list
- **word‑tokenizer.js**: Tokenization utilities (Latin + CJK), word validation, distribute evenly
- **useEditLyricsV2Document.js**: Maintain word timing consistency when line text/start time changes
- **EditLyricsV2.vue**: Wire playback progress to lane, pass selected line, handle `word‑timing‑edited` event for auto‑replay

**Note:** Plain lyrics and synced lyrics are independent. Editing plain lyrics does not automatically update synced lines. Use `importSyncedLinesFromPlain` for explicit import.

## Keyboard Shortcuts & Actions

| Key | Action |
|-----|--------|
| `z` | Sync selected boundary to current playback position |
| `x` | Sync selected boundary to playback, and if on last boundary, advance to next lyric line |
