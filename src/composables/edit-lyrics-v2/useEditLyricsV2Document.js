import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'
import { createSyncedLinesFromPlain, normalizeSyncedLine, parseLyricsfile, serializeLyricsfile } from '@/utils/lyricsfile.js'

const createEmptySyncedLine = () => ({
  text: '',
  words: []
})

export function useEditLyricsV2Document({ editingTrack, progress, toast }) {
  const plainLyrics = ref('')
  const syncedLines = ref([])
  const lyricsfileDocument = ref(null)
  const isDirty = ref(false)
  const selectedSyncedLineIndex = ref(-1)
  const isSyncedLineEditing = ref(false)

  const ensureSelectedSyncedLine = () => {
    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      return
    }

    if (
      !Number.isInteger(selectedSyncedLineIndex.value)
      || selectedSyncedLineIndex.value < 0
      || selectedSyncedLineIndex.value >= syncedLines.value.length
    ) {
      selectedSyncedLineIndex.value = 0
    }
  }

  const selectSyncedLine = (lineIndex) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    selectedSyncedLineIndex.value = lineIndex
  }

  const setSyncedLineEditingState = (value) => {
    isSyncedLineEditing.value = value
  }

  const initializeLyrics = () => {
    const track = editingTrack.value
    if (!track) {
      plainLyrics.value = ''
      syncedLines.value = []
      lyricsfileDocument.value = null
      isDirty.value = false
      return
    }

    const parsed = parseLyricsfile(track.lyricsfile)

    plainLyrics.value = parsed.plainLyrics
    // Load synced lines independently from plain lyrics
    // This allows users to have different structures (e.g., annotations, empty lines)
    syncedLines.value = parsed.syncedLines.map((line) => normalizeSyncedLine(line))

    console.log('syncedLines', syncedLines.value)
    lyricsfileDocument.value = parsed.document
    isDirty.value = false
    isSyncedLineEditing.value = false
    ensureSelectedSyncedLine()
  }

  const updatePlainLyrics = (lyrics) => {
    plainLyrics.value = lyrics
    // Plain and synced lyrics are now independent - editing plain lyrics
    // does not automatically update synced lines, allowing users to have
    // different structures (e.g., annotations like [chorus], empty lines)
    isDirty.value = true
  }

  const updateSyncedLines = (lines) => {
    syncedLines.value = lines
    isDirty.value = true
    ensureSelectedSyncedLine()
  }

  const addSyncedLineAt = (lineIndex) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex > syncedLines.value.length) {
      return
    }

    const nextLines = [...syncedLines.value]
    nextLines.splice(lineIndex, 0, createEmptySyncedLine())

    syncedLines.value = nextLines
    selectedSyncedLineIndex.value = lineIndex
    isDirty.value = true
  }

  const deleteSyncedLine = (lineIndex) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    syncedLines.value = syncedLines.value.filter((_, index) => index !== lineIndex)
    isDirty.value = true

    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      return
    }

    selectedSyncedLineIndex.value = Math.min(lineIndex, syncedLines.value.length - 1)
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

  const eraseWordTimings = (lineIndex) => {
    withUpdatedLine(lineIndex, (line) => ({
      ...line,
      words: []
    }))
  }

  const offsetWordTimings = (lineIndex, offsetMs) => {
    if (!Number.isFinite(offsetMs) || offsetMs === 0) {
      return
    }

    withUpdatedLine(lineIndex, (line) => {
      if (!Array.isArray(line.words) || line.words.length === 0) {
        return line
      }

      return {
        ...line,
        words: line.words.map((word) => ({
          ...word,
          start_ms: Number.isFinite(word.start_ms)
            ? Math.max(0, Math.round(word.start_ms + offsetMs))
            : word.start_ms
        }))
      }
    })
  }

  const syncLineToCurrentProgress = (lineIndex) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const oldStartMs = syncedLines.value[lineIndex]?.start_ms
    const newStartMs = Math.max(0, Math.round(progress.value * 1000))

    withUpdatedLine(lineIndex, (line) => ({
      ...line,
      start_ms: newStartMs
    }))

    if (Number.isFinite(oldStartMs)) {
      offsetWordTimings(lineIndex, newStartMs - oldStartMs)
    }
  }

  const shiftLineTimestampBy = (lineIndex, offsetMs) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const oldStartMsRaw = syncedLines.value[lineIndex]?.start_ms
    const oldStartMs = Number.isFinite(oldStartMsRaw) ? oldStartMsRaw : 0
    const newStartMs = Math.max(0, Math.round(oldStartMs + offsetMs))

    withUpdatedLine(lineIndex, (line) => ({
      ...line,
      start_ms: newStartMs
    }))

    offsetWordTimings(lineIndex, newStartMs - oldStartMs)
  }

  const rewindLineBy100 = (lineIndex) => {
    shiftLineTimestampBy(lineIndex, -100)
  }

  const forwardLineBy100 = (lineIndex) => {
    shiftLineTimestampBy(lineIndex, 100)
  }

  const updateLineText = (lineIndex, newText) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const line = syncedLines.value[lineIndex]
    if (!line || line.text === newText) {
      return
    }

    withUpdatedLine(lineIndex, (currentLine) => ({
      ...currentLine,
      text: newText,
      words: []
    }))
  }

  const saveLyrics = async () => {
    if (!editingTrack.value || !isDirty.value) {
      return
    }

    try {
      const lyricsfile = serializeLyricsfile({
        track: editingTrack.value,
        plainLyrics: plainLyrics.value,
        syncedLines: syncedLines.value,
        baseDocument: lyricsfileDocument.value
      })

      await invoke('save_lyrics', {
        trackId: editingTrack.value.id,
        lyricsfile
      })

      const parsed = parseLyricsfile(lyricsfile)
      // Preserve independent synced lines structure after save
      // Don't re-sync to plain lyrics to maintain separate structures
      syncedLines.value = parsed.syncedLines.map((line) => normalizeSyncedLine(line))
      lyricsfileDocument.value = parsed.document
      isDirty.value = false
    } catch (error) {
      console.error(error)
      toast.error(error)
    }
  }

  const hasPlainLyrics = computed(() => plainLyrics.value.trim().length > 0)

  const selectedLineExists = computed(() => (
    Number.isInteger(selectedSyncedLineIndex.value)
    && selectedSyncedLineIndex.value >= 0
    && selectedSyncedLineIndex.value < syncedLines.value.length
  ))

  const currentPlayingSyncedLineIndex = computed(() => {
    if (!Number.isFinite(progress.value) || syncedLines.value.length === 0) {
      return -1
    }

    const progressMs = Math.max(0, Math.round(progress.value * 1000))

    for (let index = syncedLines.value.length - 1; index >= 0; index -= 1) {
      const startMs = syncedLines.value[index]?.start_ms
      if (Number.isFinite(startMs) && startMs <= progressMs) {
        return index
      }
    }

    return -1
  })

  watch(syncedLines, () => {
    ensureSelectedSyncedLine()
  }, { deep: true })

  return {
    plainLyrics,
    syncedLines,
    lyricsfileDocument,
    isDirty,
    selectedSyncedLineIndex,
    isSyncedLineEditing,
    hasPlainLyrics,
    selectedLineExists,
    currentPlayingSyncedLineIndex,
    initializeLyrics,
    updatePlainLyrics,
    updateSyncedLines,
    selectSyncedLine,
    setSyncedLineEditingState,
    addSyncedLineAt,
    deleteSyncedLine,
    importSyncedLinesFromPlain,
    syncLineToCurrentProgress,
    rewindLineBy100,
    forwardLineBy100,
    saveLyrics,
    ensureSelectedSyncedLine,
    updateLineText,
    eraseWordTimings,
    offsetWordTimings
  }
}
