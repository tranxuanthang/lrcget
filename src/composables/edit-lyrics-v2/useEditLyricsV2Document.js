import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'
import {
  createSyncedLinesFromPlain,
  normalizeSyncedLine,
  parseLyricsfile,
  serializeLyricsfile,
} from '@/utils/lyricsfile.js'

const createEmptySyncedLine = () => ({
  text: '',
  words: [],
})

export function useEditLyricsV2Document({ audioSource, lyricsfile, trackId, progress, toast }) {
  const plainLyrics = ref('')
  const syncedLines = ref([])
  const lyricsfileDocument = ref(null)
  const isDirty = ref(false)
  const selectedSyncedLineIndex = ref(-1)
  const isSyncedLineEditing = ref(false)
  const isInstrumental = ref(false)

  const ensureSelectedSyncedLine = () => {
    if (syncedLines.value.length === 0) {
      selectedSyncedLineIndex.value = -1
      return
    }

    if (
      !Number.isInteger(selectedSyncedLineIndex.value) ||
      selectedSyncedLineIndex.value < 0 ||
      selectedSyncedLineIndex.value >= syncedLines.value.length
    ) {
      selectedSyncedLineIndex.value = 0
    }
  }

  const selectSyncedLine = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    selectedSyncedLineIndex.value = lineIndex
  }

  const setSyncedLineEditingState = value => {
    isSyncedLineEditing.value = value
  }

  const initializeLyrics = () => {
    // Get lyrics content from lyricsfile prop, or empty string if none
    const lyricsfileContent = lyricsfile.value?.content ?? ''
    const parsed = parseLyricsfile(lyricsfileContent)

    plainLyrics.value = parsed.plainLyrics
    // Load synced lines independently from plain lyrics
    // This allows users to have different structures (e.g., annotations, empty lines)
    syncedLines.value = parsed.syncedLines.map(line => normalizeSyncedLine(line))
    isInstrumental.value = parsed.isInstrumental

    lyricsfileDocument.value = parsed.document
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
    syncedLines.value = lines
    isDirty.value = true
    ensureSelectedSyncedLine()
  }

  const addSyncedLineAt = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex > syncedLines.value.length) {
      return
    }

    const nextLines = [...syncedLines.value]
    nextLines.splice(lineIndex, 0, createEmptySyncedLine())

    syncedLines.value = nextLines
    selectedSyncedLineIndex.value = lineIndex
    isDirty.value = true
  }

  const deleteSyncedLine = lineIndex => {
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

    withUpdatedLine(lineIndex, line => ({
      ...line,
      start_ms: newStartMs,
    }))
  }

  const shiftLineTimestampBy = (lineIndex, offsetMs) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const currentStartMs = syncedLines.value[lineIndex]?.start_ms
    const baseStartMs = Number.isFinite(currentStartMs) ? currentStartMs : 0
    const newStartMs = Math.max(0, Math.round(baseStartMs + offsetMs))

    withUpdatedLine(lineIndex, line => ({
      ...line,
      start_ms: newStartMs,
    }))
  }

  const rewindLineBy100 = lineIndex => {
    shiftLineTimestampBy(lineIndex, -100)
  }

  const forwardLineBy100 = lineIndex => {
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
      // Build track data from audioSource for serialization
      const trackData = {
        title: audioSource.value?.title ?? lyricsfile?.value?.metadata?.title ?? 'Unknown',
        artist_name:
          audioSource.value?.artist_name ?? lyricsfile?.value?.metadata?.artist ?? 'Unknown',
        album_name: audioSource.value?.album_name ?? lyricsfile?.value?.metadata?.album ?? '',
        duration:
          audioSource.value?.duration ?? lyricsfile?.value?.metadata?.duration_ms / 1000 ?? 0,
      }

      const serializedContent = serializeLyricsfile({
        track: trackData,
        plainLyrics: plainLyrics.value,
        syncedLines: syncedLines.value,
        baseDocument: lyricsfileDocument.value,
        isInstrumental: isInstrumental.value,
      })

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
      // Preserve independent synced lines structure after save
      // Don't re-sync to plain lyrics to maintain separate structures
      syncedLines.value = parsed.syncedLines.map(line => normalizeSyncedLine(line))
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

  const selectedLineExists = computed(
    () =>
      Number.isInteger(selectedSyncedLineIndex.value) &&
      selectedSyncedLineIndex.value >= 0 &&
      selectedSyncedLineIndex.value < syncedLines.value.length
  )

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
    isSyncedLineEditing,
    hasPlainLyrics,
    selectedLineExists,
    currentPlayingSyncedLineIndex,
    isInstrumental,
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
    setInstrumental,
  }
}
