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

  const syncEndToCurrentProgress = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const newEndMs = Math.max(0, Math.round(progress.value * 1000))

    withUpdatedLine(lineIndex, line => ({
      ...line,
      end_ms: newEndMs,
    }))
  }

  const shiftEndTimestampBy = (lineIndex, offsetMs) => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
      return
    }

    const currentEndMs = syncedLines.value[lineIndex]?.end_ms
    const baseEndMs = Number.isFinite(currentEndMs) ? currentEndMs : 0
    const newEndMs = Math.max(0, Math.round(baseEndMs + offsetMs))

    withUpdatedLine(lineIndex, line => ({
      ...line,
      end_ms: newEndMs,
    }))
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

      // serializedLyricsfile returns empty string if there's nothing to serialize
      // In that case, we treat it as a successful save (nothing to save)
      if (!serializedContent) {
        return true
      }

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

  // Compute serialized lyricsfile content for export, publish, and debug
  const serializedLyricsfile = computed(() => {
    // Build track data for serialization
    // Prefer existing lyricsfile metadata first, then fall back to audioSource/track data
    const trackData = {
      title: lyricsfileDocument?.value?.metadata?.title ?? audioSource.value?.title ?? null,
      artist_name:
        lyricsfileDocument?.value?.metadata?.artist ?? audioSource.value?.artist_name ?? null,
      album_name: lyricsfileDocument?.value?.metadata?.album ?? audioSource.value?.album_name ?? null,
      duration:
        (lyricsfileDocument?.value?.metadata?.duration_ms != null
          ? lyricsfileDocument.value.metadata.duration_ms / 1000
          : null) ?? audioSource.value?.duration ?? null,
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
    isSyncedLineEditing,
    hasPlainLyrics,
    selectedLineExists,
    isInstrumental,
    serializedLyricsfile,
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
    syncEndToCurrentProgress,
    rewindEndBy100,
    forwardEndBy100,
    saveLyrics,
    ensureSelectedSyncedLine,
    updateLineText,
    eraseWordTimings,
    setInstrumental,
  }
}
