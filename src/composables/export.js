import { computed, markRaw, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const delay = time => new Promise(resolve => setTimeout(resolve, time))

const exportQueue = ref([])
const log = ref([])
const exportedCount = ref(0)
const skippedCount = ref(0)
const errorCount = ref(0)
const isExporting = ref(false)
const totalCount = ref(0)

const addLog = logObj => {
  log.value.unshift(markRaw(logObj))
  if (log.value.length > 100) {
    log.value.pop()
  }
}

const exportTrack = async (track, formats) => {
  try {
    const result = await invoke('export_track_lyrics', {
      trackId: track.id,
      formats,
    })

    if (!isExporting.value) {
      return
    }

    // Check result from backend
    // Use the backend's skipped count to determine if this was a skip
    const hasErrors = result.errors > 0
    const hasExported = result.exported > 0
    const hasSkipped = result.skipped > 0

    if (hasErrors) {
      addLog({
        status: 'error',
        title: track.title,
        artistName: track.artist_name,
        message: result.message || 'Export failed',
        // details: result.details,
      })
      errorCount.value++
    } else if (hasExported) {
      addLog({
        status: 'exported',
        title: track.title,
        artistName: track.artist_name,
        message: result.message || `Exported to ${result.exported} format(s)`,
        // details: result.details,
      })
      exportedCount.value++
    } else if (hasSkipped) {
      addLog({
        status: 'skipped',
        title: track.title,
        artistName: track.artist_name,
        message: result.message || 'Skipped: no lyrics available for selected formats',
        // details: result.details,
      })
      skippedCount.value++
    } else {
      addLog({
        status: 'skipped',
        title: track.title,
        artistName: track.artist_name,
        message: result.message || 'Nothing to export',
        // details: result.details,
      })
      skippedCount.value++
    }
  } catch (error) {
    if (!isExporting.value) {
      return
    }

    addLog({
      status: 'error',
      title: track.title,
      artistName: track.artist_name,
      message: error,
    })
    errorCount.value++
  }
}

const exportNext = async () => {
  while (true) {
    if (exportQueue.value.length === 0) {
      await delay(1000)
      continue
    }

    const { trackId, formats } = exportQueue.value.shift()
    try {
      const track = await invoke('get_track', { trackId: trackId })
      await exportTrack(track, formats)
    } catch (error) {
      if (!isExporting.value) {
        continue
      }
      console.error('Failed to get track for export:', error)
      errorCount.value++
    }

    await delay(1)
  }
}

const exportProgress = computed(() => {
  if (!isExporting.value) {
    return 0.0
  }

  if (totalCount.value === 0) {
    return 0.0
  }

  const processedCount = exportedCount.value + skippedCount.value + errorCount.value
  if (processedCount >= totalCount.value) {
    return 1.0
  }

  return processedCount / totalCount.value
})

const addToQueue = (trackIds, formats) => {
  isExporting.value = true
  const snapshot = Object.freeze([
    ...(formats.plainText ? ['txt'] : []),
    ...(formats.syncedLrc ? ['lrc'] : []),
    ...(formats.embedIntoTrack ? ['embedded'] : []),
  ])

  for (let i = 0; i < trackIds.length; i++) {
    exportQueue.value.push({ trackId: trackIds[i], formats: snapshot })
  }

  totalCount.value += trackIds.length

  console.log(`Added ${trackIds.length} tracks to export queue`)
}

const startOver = () => {
  exportQueue.value = []
  log.value = []
  exportedCount.value = 0
  skippedCount.value = 0
  errorCount.value = 0
  totalCount.value = 0
  isExporting.value = false
}

const stopExporting = () => {
  startOver()
}

export function useExporter() {
  return {
    isExporting,
    exportQueue,
    exportProgress,
    exportedCount,
    skippedCount,
    errorCount,
    totalCount,
    log,
    addToQueue,
    startOver,
    stopExporting,
    exportNext,
  }
}
