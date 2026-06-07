import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'
import {
  createSyncedLinesFromPlain,
  normalizeSyncedLine,
  parseLyricsfile,
  serializeLyricsfile,
} from '@/utils/lyricsfile.js'

const createEmptySyncedLine = () => normalizeSyncedLine({})

const MIN_WORD_DURATION_MS = 10

// The first word's start_ms is conceptually the same point as the line's
// start_ms, so nudging line start drags word[0] with it. Then we cascade
// forward: each successive word's start_ms must be at least
// MIN_WORD_DURATION_MS after the previous word's start (so every word keeps
// at least that much room). Words that already sit past the cascade
// threshold stay where they are.
const setLineStartMs = (line, newStartMs) => {
  if (!Array.isArray(line.words) || line.words.length === 0) {
    return { ...line, start_ms: newStartMs }
  }

  const firstWord = line.words[0]
  if (!Number.isFinite(firstWord?.start_ms)) {
    return { ...line, start_ms: newStartMs }
  }

  const updatedWords = [...line.words]
  updatedWords[0] = { ...firstWord, start_ms: newStartMs }

  for (let i = 1; i < updatedWords.length; i++) {
    const prevStart = updatedWords[i - 1].start_ms
    const minStart = prevStart + MIN_WORD_DURATION_MS
    const currentStart = updatedWords[i].start_ms
    if (Number.isFinite(currentStart) && currentStart < minStart) {
      updatedWords[i] = { ...updatedWords[i], start_ms: minStart }
    }
  }

  return { ...line, start_ms: newStartMs, words: updatedWords }
}

// if the last word has an explicit end_ms, drag it with the new line end. 
// Words without an explicit end_ms are left untouched 
// (the lane derives their end from the next word's start or the line end).
const setLineEndMs = (line, newEndMs) => {
  if (!Array.isArray(line.words) || line.words.length === 0) {
    return { ...line, end_ms: newEndMs }
  }

  const lastIdx = line.words.length - 1
  const lastWord = line.words[lastIdx]
  if (!Number.isFinite(lastWord?.end_ms)) {
    return { ...line, end_ms: newEndMs }
  }

  const updatedWords = [...line.words]
  updatedWords[lastIdx] = { ...lastWord, end_ms: newEndMs }

  return { ...line, end_ms: newEndMs, words: updatedWords }
}

// Stable-sort the lines by start_ms. The editor maintains a sorted-by-start_ms
// invariant; this is the helper called after any mutation that can move a line
// past its neighbors. Lines with missing/non-finite start_ms sink to the end.
const sortByStartMs = lines => {
  const tagged = lines.map((line, originalIndex) => ({ line, originalIndex }))
  tagged.sort((a, b) => {
    const aStart = Number.isFinite(a.line.start_ms) ? a.line.start_ms : Infinity
    const bStart = Number.isFinite(b.line.start_ms) ? b.line.start_ms : Infinity
    return aStart - bStart || a.originalIndex - b.originalIndex
  })
  return tagged.map(t => t.line)
}

export function useEditLyricsV2Document({ audioSource, lyricsfile, trackId, progress, toast }) {
  const plainLyrics = ref('')
  const syncedLines = ref([])
  const lyricsfileDocument = ref(null)
  const isDirty = ref(false)
  const selectedSyncedLineIndex = ref(-1)
  // Bulk-selection is keyed by stable line id, not array index, so a reorder
  // (triggered by start_ms mutations) doesn't strand the selection on stale
  // positions. `selectedSyncedLineIndices` below derives the live indices
  // from these ids and the current `syncedLines.value` array.
  const selectedSyncedLineIds = ref([])
  const isSyncedLineEditing = ref(false)
  const isInstrumental = ref(false)

  const selectedSyncedLineIndices = computed(() => {
    if (selectedSyncedLineIds.value.length === 0) return []
    const idSet = new Set(selectedSyncedLineIds.value)
    const indices = []
    syncedLines.value.forEach((line, idx) => {
      if (line?.id !== undefined && idSet.has(line.id)) indices.push(idx)
    })
    return indices
  })

  const clearSyncedLineSelection = () => {
    selectedSyncedLineIds.value = []
  }

  const ensureSelectedSyncedLine = () => {
    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      clearSyncedLineSelection()
      return
    }

    if (
      !Number.isInteger(selectedSyncedLineIndex.value) ||
      selectedSyncedLineIndex.value < 0 ||
      selectedSyncedLineIndex.value >= syncedLines.value.length
    ) {
      selectedSyncedLineIndex.value = 0
    }

    const liveIds = new Set(
      syncedLines.value.map(line => line?.id).filter(id => id !== undefined)
    )
    selectedSyncedLineIds.value = selectedSyncedLineIds.value.filter(id => liveIds.has(id))
  }

  // Apply a mutation that may shift a line's start_ms past its neighbors:
  // build the new lines array, re-sort by start_ms, and restore selection by
  // the selected line's stable id
  const applyLineMutation = buildNextLines => {
    const sourceLines = syncedLines.value
    const selectedLineId = sourceLines[selectedSyncedLineIndex.value]?.id ?? null

    const nextLines = buildNextLines(sourceLines)
    const sortedLines = sortByStartMs(nextLines)

    syncedLines.value = sortedLines
    if (selectedLineId !== null) {
      const newIdx = sortedLines.findIndex(line => line.id === selectedLineId)
      if (newIdx >= 0) selectedSyncedLineIndex.value = newIdx
    }
    isDirty.value = true
  }

  const selectSyncedLine = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    selectedSyncedLineIndex.value = lineIndex
    clearSyncedLineSelection()
  }

  const selectSyncedLineRange = (start, end) => {
    if (syncedLines.value.length === 0) {
      clearSyncedLineSelection()
      return
    }
    const min = Math.max(0, Math.min(start, end))
    const max = Math.min(syncedLines.value.length - 1, Math.max(start, end))
    const ids = []
    for (let i = min; i <= max; i++) {
      const id = syncedLines.value[i]?.id
      if (id !== undefined) ids.push(id)
    }
    selectedSyncedLineIds.value = ids
    selectedSyncedLineIndex.value = min
  }

  const toggleSyncedLineSelection = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }
    const id = syncedLines.value[lineIndex]?.id
    if (id === undefined) return

    const current = new Set(selectedSyncedLineIds.value)
    if (current.has(id)) {
      current.delete(id)
    } else {
      current.add(id)
    }
    selectedSyncedLineIds.value = Array.from(current)
    if (selectedSyncedLineIds.value.length > 0) {
      const indices = selectedSyncedLineIndices.value
      if (indices.length > 0) selectedSyncedLineIndex.value = indices[0]
    }
  }

  const setSyncedLineEditingState = value => {
    isSyncedLineEditing.value = value
  }

  const initializeLyrics = () => {
    // Get lyrics content from lyricsfile prop, or empty string if none
    const lyricsfileContent = lyricsfile.value?.content ?? ''

    console.log(lyricsfileContent)
    const parsed = parseLyricsfile(lyricsfileContent)

    plainLyrics.value = parsed.plainLyrics
    // Load synced lines independently from plain lyrics
    // This allows users to have different structures (e.g., annotations, empty lines)
    syncedLines.value = parsed.syncedLines.map(line => normalizeSyncedLine(line))
    isInstrumental.value = parsed.isInstrumental

    lyricsfileDocument.value = parsed.document
    console.log(lyricsfileDocument.value)
    isDirty.value = false
    isSyncedLineEditing.value = false
    ensureSelectedSyncedLine()
  }

  const updatePlainLyrics = lyrics => {
    plainLyrics.value = lyrics
    // Plain and synced lyrics are now independent - editing plain lyrics
    // does not automatically update synced lines, allowing users to have
    // different structures (e.g., annotations like [chorus], empty lines)
    isDirty.value = true
  }

  const updateSyncedLines = lines => {
    // Normalize so bare lines from LRC paste/import paths pick up stable ids.
    syncedLines.value = lines.map(line => normalizeSyncedLine(line))
    isDirty.value = true
    ensureSelectedSyncedLine()
  }

  const addSyncedLineAt = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex > syncedLines.value.length) {
      return
    }

    // Auto-fill timestamps from sandwich neighbors so a fresh line drops in
    // pre-timed. Insert at index N pushes the previous value at N to N+1, so
    // the neighbor on the "next" side is currently at N.
    const newLine = createEmptySyncedLine()
    const prevLine = lineIndex > 0 ? syncedLines.value[lineIndex - 1] : null
    const nextLine = lineIndex < syncedLines.value.length ? syncedLines.value[lineIndex] : null

    if (prevLine && Number.isFinite(prevLine.end_ms)) {
      newLine.start_ms = prevLine.end_ms
    }
    if (nextLine && Number.isFinite(nextLine.start_ms)) {
      newLine.end_ms = nextLine.start_ms
    }

    const nextLines = [...syncedLines.value]
    nextLines.splice(lineIndex, 0, newLine)

    const sortedLines = sortByStartMs(nextLines)
    syncedLines.value = sortedLines
    const newIdx = sortedLines.findIndex(line => line.id === newLine.id)
    selectedSyncedLineIndex.value = newIdx >= 0 ? newIdx : lineIndex
    clearSyncedLineSelection()
    isDirty.value = true
  }

  const syncEndToNextLineStart = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const nextLine = syncedLines.value[lineIndex + 1]
    if (!nextLine || !Number.isFinite(nextLine.start_ms)) {
      return
    }

    withUpdatedLine(lineIndex, line => setLineEndMs(line, nextLine.start_ms))
  }

  const deleteSyncedLine = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    syncedLines.value = syncedLines.value.filter((_, index) => index !== lineIndex)
    isDirty.value = true
    clearSyncedLineSelection()

    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      return
    }

    selectedSyncedLineIndex.value = Math.min(lineIndex, syncedLines.value.length - 1)
  }

  const bulkDeleteLines = indices => {
    if (!Array.isArray(indices) || indices.length === 0) {
      return
    }
    const idsToDelete = new Set()
    for (const idx of indices) {
      const id = syncedLines.value[idx]?.id
      if (id !== undefined) idsToDelete.add(id)
    }
    if (idsToDelete.size === 0) return

    syncedLines.value = syncedLines.value.filter(line => !idsToDelete.has(line.id))
    isDirty.value = true
    clearSyncedLineSelection()

    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      return
    }

    const firstDeleted = Math.min(...indices)
    selectedSyncedLineIndex.value = Math.min(firstDeleted, syncedLines.value.length - 1)
  }

  const bulkShiftLineTimestamps = (indices, offsetMs) => {
    if (!Array.isArray(indices) || indices.length === 0) {
      return
    }
    // Resolve to ids up front: applyLineMutation re-sorts, so a shifted line
    // may move past its neighbors mid-flight. Identifying by id keeps the
    // mutation hitting the right rows regardless of post-sort positions.
    const idsToShift = new Set()
    for (const idx of indices) {
      const id = syncedLines.value[idx]?.id
      if (id !== undefined) idsToShift.add(id)
    }
    if (idsToShift.size === 0) return

    applyLineMutation(lines =>
      lines.map(line => {
        if (!idsToShift.has(line.id)) return line
        const currentStartMs = line?.start_ms
        const baseStartMs = Number.isFinite(currentStartMs) ? currentStartMs : 0
        const newStartMs = Math.max(0, Math.round(baseStartMs + offsetMs))
        return setLineStartMs(line, newStartMs)
      })
    )
  }

  const bulkRewindLines = indices => {
    bulkShiftLineTimestamps(indices, -100)
  }

  const bulkForwardLines = indices => {
    bulkShiftLineTimestamps(indices, 100)
  }

  const importSyncedLinesFromPlain = () => {
    if (!hasPlainLyrics.value) {
      return
    }

    syncedLines.value = createSyncedLinesFromPlain(plainLyrics.value, [])
    isDirty.value = true
    ensureSelectedSyncedLine()
  }

  const withUpdatedLine = (lineIndex, updater) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const nextLines = syncedLines.value.map((line, index) => {
      if (index !== lineIndex) {
        return line
      }

      return updater(line)
    })

    syncedLines.value = nextLines
    isDirty.value = true
  }

  const eraseWordTimings = lineIndex => {
    withUpdatedLine(lineIndex, line => ({
      ...line,
      words: [],
    }))
  }

  const syncLineToCurrentProgress = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const newStartMs = Math.max(0, Math.round(progress.value * 1000))

    const prevLineIndex = lineIndex - 1
    const shouldSetPrevEndMs =
      prevLineIndex >= 0 && !Number.isFinite(syncedLines.value[prevLineIndex]?.end_ms)

    // Touch the previous neighbor's end_ms (if needed) BEFORE re-sorting so we
    // close off the cue the user was sitting on, not whichever line happens
    // to land before the moved one after sort.
    applyLineMutation(lines =>
      lines.map((line, index) => {
        if (index === lineIndex) return setLineStartMs(line, newStartMs)
        if (index === prevLineIndex && shouldSetPrevEndMs) return { ...line, end_ms: newStartMs }
        return line
      })
    )
  }

  const shiftLineTimestampBy = (lineIndex, offsetMs) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const currentStartMs = syncedLines.value[lineIndex]?.start_ms
    const baseStartMs = Number.isFinite(currentStartMs) ? currentStartMs : 0
    const newStartMs = Math.max(0, Math.round(baseStartMs + offsetMs))

    applyLineMutation(lines =>
      lines.map((line, index) =>
        index === lineIndex ? setLineStartMs(line, newStartMs) : line
      )
    )
  }

  const updateLineWords = ({ lineIndex, words, lineStartMs }) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const nextLineStartMs = Number.isFinite(lineStartMs)
      ? Math.max(0, Math.round(lineStartMs))
      : null

    applyLineMutation(lines =>
      lines.map((line, index) => {
        if (index !== lineIndex) return line
        return {
          ...line,
          ...(nextLineStartMs === null ? {} : { start_ms: nextLineStartMs }),
          words,
        }
      })
    )
  }

  const rewindLineBy100 = lineIndex => {
    shiftLineTimestampBy(lineIndex, -100)
  }

  const forwardLineBy100 = lineIndex => {
    shiftLineTimestampBy(lineIndex, 100)
  }

  const syncEndToCurrentProgress = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const newEndMs = Math.max(0, Math.round(progress.value * 1000))

    withUpdatedLine(lineIndex, line => setLineEndMs(line, newEndMs))
  }

  const shiftEndTimestampBy = (lineIndex, offsetMs) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const currentEndMs = syncedLines.value[lineIndex]?.end_ms
    const baseEndMs = Number.isFinite(currentEndMs) ? currentEndMs : 0
    const newEndMs = Math.max(0, Math.round(baseEndMs + offsetMs))

    withUpdatedLine(lineIndex, line => setLineEndMs(line, newEndMs))
  }

  const rewindEndBy100 = lineIndex => {
    shiftEndTimestampBy(lineIndex, -100)
  }

  const forwardEndBy100 = lineIndex => {
    shiftEndTimestampBy(lineIndex, 100)
  }

  const updateLineText = (lineIndex, newText) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const line = syncedLines.value[lineIndex]
    if (!line || line.text === newText) {
      return
    }

    withUpdatedLine(lineIndex, currentLine => ({
      ...currentLine,
      text: newText,
      words: [],
    }))
  }

  const setInstrumental = value => {
    isInstrumental.value = Boolean(value)
    isDirty.value = true
  }

  const saveLyrics = async () => {
    if (!isDirty.value) {
      return true
    }

    try {
      const serializedContent = serializedLyricsfile.value

      // Determine if this is a library track or a standalone lyricsfile
      // trackId is passed separately from audioSource to handle temporary associations
      // where a library track might be used for playback but the lyricsfile should not
      // be associated with the track (e.g., LRCLIB Browser flow)
      const isLibraryTrack = trackId?.value !== null && trackId?.value !== undefined

      if (isLibraryTrack) {
        // Library track: pass track_id (lyricsfile_id can be null if no lyricsfile exists yet)
        await invoke('save_lyrics', {
          trackId: trackId.value,
          lyricsfileId: lyricsfile?.value?.id ?? null,
          lyricsfile: serializedContent,
        })
      } else {
        // Standalone lyricsfile: pass lyricsfile_id (must not be null)
        const standaloneLyricsfileId = lyricsfile?.value?.id
        if (!standaloneLyricsfileId) {
          throw new Error('Standalone lyricsfile must have an ID')
        }
        await invoke('save_lyrics', {
          trackId: null,
          lyricsfileId: standaloneLyricsfileId,
          lyricsfile: serializedContent,
        })
      }

      const parsed = parseLyricsfile(serializedContent)
      // Preserve independent synced lines structure after save (don't re-sync
      // to plain lyrics). Also re-attach the existing line ids by array index
      // so the v-for keys don't churn across save
      const previousLines = syncedLines.value
      syncedLines.value = parsed.syncedLines.map((line, i) => {
        const normalized = normalizeSyncedLine(line)
        if (previousLines[i]?.id !== undefined) {
          normalized.id = previousLines[i].id
        }
        return normalized
      })
      lyricsfileDocument.value = parsed.document
      isDirty.value = false
      return true
    } catch (error) {
      console.error(error)
      toast.error(error)
      return false
    }
  }

  const hasPlainLyrics = computed(() => plainLyrics.value.trim().length > 0)

  // Compute serialized lyricsfile content for export, publish, and debug
  const serializedLyricsfile = computed(() => {
    // Build track data for serialization
    // Prefer existing lyricsfile metadata first, then fall back to audioSource/track data
    const trackData = {
      title: lyricsfileDocument?.value?.metadata?.title ?? audioSource.value?.title ?? null,
      artist_name:
        lyricsfileDocument?.value?.metadata?.artist ?? audioSource.value?.artist_name ?? null,
      album_name:
        lyricsfileDocument?.value?.metadata?.album ?? audioSource.value?.album_name ?? null,
      duration:
        (lyricsfileDocument?.value?.metadata?.duration_ms != null
          ? lyricsfileDocument.value.metadata.duration_ms / 1000
          : null) ??
        audioSource.value?.duration ??
        null,
    }

    console.log(lyricsfileDocument.value)
    console.log(trackData)

    return (
      serializeLyricsfile({
        track: trackData,
        plainLyrics: plainLyrics.value,
        syncedLines: syncedLines.value,
        baseDocument: lyricsfileDocument.value,
        isInstrumental: isInstrumental.value,
      }) || ''
    )
  })

  const selectedLineExists = computed(
    () =>
      Number.isInteger(selectedSyncedLineIndex.value) &&
      selectedSyncedLineIndex.value >= 0 &&
      selectedSyncedLineIndex.value < syncedLines.value.length
  )

  watch(
    syncedLines,
    () => {
      ensureSelectedSyncedLine()
    },
    { deep: true }
  )

  return {
    plainLyrics,
    syncedLines,
    lyricsfileDocument,
    isDirty,
    selectedSyncedLineIndex,
    selectedSyncedLineIndices,
    selectedSyncedLineIds,
    isSyncedLineEditing,
    hasPlainLyrics,
    selectedLineExists,
    isInstrumental,
    serializedLyricsfile,
    initializeLyrics,
    updatePlainLyrics,
    updateSyncedLines,
    selectSyncedLine,
    selectSyncedLineRange,
    toggleSyncedLineSelection,
    clearSyncedLineSelection,
    setSyncedLineEditingState,
    addSyncedLineAt,
    deleteSyncedLine,
    bulkDeleteLines,
    bulkRewindLines,
    bulkForwardLines,
    importSyncedLinesFromPlain,
    syncLineToCurrentProgress,
    rewindLineBy100,
    forwardLineBy100,
    syncEndToCurrentProgress,
    rewindEndBy100,
    forwardEndBy100,
    syncEndToNextLineStart,
    saveLyrics,
    ensureSelectedSyncedLine,
    updateLineText,
    updateLineWords,
    eraseWordTimings,
    setInstrumental,
  }
}
