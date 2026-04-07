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
────────────────────
[Previous lines]

[Selected Line Row]  ← currently selected line

[Next lines]
```

---

## Core Features

### 1. Drag Behavior

- Words render as draggable horizontal segments in the timeline that fill the space until the next word
- Timeline range: `[line.start_ms, next_line.start_ms]`
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
- Visual feedback during drag with ghost/overlay
- Changes apply immediately on pointer release

### 2. Tokens

- If `line.words[]` exists and is valid → use existing
- If missing → auto-generate from `line.text`:
  - **Latin scripts**: space-delimited, preserve trailing spaces in word tokens
  - **CJK (Chinese/Japanese/Korean)**: treat each character as a token (no spaces needed)
- Initial timing: evenly distributed across the line time window

### 2.1 Availability Rules

Word sync is available only when all conditions are true:

- The selected line has non-empty text content
- The selected line has a valid `start_ms`
- The next line exists and has a valid `start_ms`

When unavailable, the lane stays visible and shows a short placeholder reason (no content, missing current sync, or missing next-line sync).

### 2.2 Data Consistency Rules

- If line text changes, clear `line.words` for that line
- If line `start_ms` changes, offset every `words[].start_ms` by the same actual delta
- If next line `start_ms` changes, previous line words do **not** need mutation (their implicit end updates from timeline context)

### 3. Distribute Evenly

- Single-click action to reset all word timings to equal spacing across the line window
- Useful for quick prototyping or resetting after manual edits
- Preserves word tokens, only adjusts `start_ms` values

### 4. Playback Feedback

**Progress Indicator**
- A vertical playhead indicator moves across the timeline as the song plays
- The indicator shows the current playback position relative to the selected line's time window
- A small circle at the top of the indicator provides a visual anchor point

**Word Highlighting**
- The currently playing word becomes **bold** and receives a **slight background color change**
- This provides immediate visual feedback when timing adjustments are correct
- The active word is determined by: `word.start_ms <= currentTime < next_word.start_ms`

**Auto-Replay After Edit**
- After completing a drag-drop action on a word boundary, the song automatically replays from `line.start_ms`
- This allows instant verification of the timing adjustment
- The playback resumes from the beginning of the current line to hear the word in context

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
   - "Distribute evenly" button
   - Empty/placeholder state when no line selected or when word sync is unavailable

2. **SyncedWordTimingSegment.vue**
   - Draggable word block
   - Shows word text
   - Visual state: default / dragging / selected

---

## Integration Points

- **SyncedLyricsEditor.vue**: Mount lane as fixed header above the scrollable lyrics list
- **useEditLyricsV2Document.js**: Add helpers for word timing updates and tokenization
- **useEditLyricsV2Document.js**: Keep word timing consistent when line text/start time changes
- **EditLyricsV2.vue**: Wire playback progress to lane for playhead sync, pass selected line to lane

---

## Implementation Steps

1. Add tokenization utilities (Latin + CJK)
2. Create `SyncedWordTimingLane.vue` with timeline layout
3. Create `SyncedWordTimingSegment.vue` with drag behavior
4. Add selection-triggered lane rendering in `SyncedLyricsEditor.vue`
5. Implement "Distribute evenly" logic
6. Wire playhead position to lane
7. Persist word timing changes to document

---

## Drag-Drop Library Decision

**Chosen Library:** `interact.js`

**Rationale:**

| Feature | interact.js | Shopify Draggable |
|---------|-------------|-------------------|
| **1D dragging** | Native `lockAxis: 'x'` | No direct support |
| **Time-based snap** | Custom snap targets + grid | Element-to-element only |
| **Neighbor constraints** | Custom modifiers | Not designed for this |
| **Boundary restriction** | Built-in `restrict` modifier | Limited |
| **Control** | Fine-grained (px-level) | Higher-level (reordering) |

**Why interact.js is better:**
- Designed for precise drag positioning with constraints, modifiers, and custom behavior
- Modifier system allows chaining: `restrict` → `custom neighbor constraint` → `snap`
- Perfect for dragging word segments to specific time positions while respecting neighbors
- ~40KB, well-documented, active maintenance

**Why Shopify Draggable is less suitable:**
- Designed for list reordering (Sortable), swapping between containers
- Snap is element-to-element, not time-based
- No native support for 1D axis locking or custom constraint logic

**Key implementation with interact.js:**
```javascript
interact('.word-segment').draggable({
  lockAxis: 'x',
  modifiers: [
    // Restrict to timeline bounds
    interact.modifiers.restrict({ restriction: '.timeline-container', ... }),
    // Custom neighbor constraint (prevents overlap)
    { set: (coords) => constrainToNeighbors(coords, words) },
    // Snap to time grid
    interact.modifiers.snap({ targets: [gridSnapper] })
  ],
  listeners: {
    end: (event) => updateWordStartTime(pxToTime(event.target.dataset.x))
  }
});
```
