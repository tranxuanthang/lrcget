import { computed, markRaw, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToast } from 'vue-toastification'

const toast = useToast()

const delay = time => new Promise(resolve => setTimeout(resolve, time))

const exportQueue = ref([])
const log = ref([])
const exportedCount = ref(0)
const skippedCount = ref(0)
const errorCount = ref(0)
const isExporting = ref(false)
const totalCount = ref(0)
const exportFormats = ref({
  plainText: false,
  syncedLrc: false,
  embedIntoTrack: false,
})

let unlistenProgress = null
let unlistenComplete = null

const addLog = logObj => {
  log.value.unshift(markRaw(logObj))
  if (log.value.length > 100) {
    log.value.pop()
  }
}

const exportTrack = async track => {
  try {
    const formats = []
    if (exportFormats.value.plainText) formats.push('txt')
    if (exportFormats.value.syncedLrc) formats.push('lrc')
    if (exportFormats.value.embedIntoTrack) formats.push('embedded')

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
  while (isExporting.value && exportQueue.value.length > 0) {
    const trackId = exportQueue.value.shift()
    try {
      const track = await invoke('get_track', { trackId: trackId })
      await exportTrack(track)
    } catch (error) {
      console.error('Failed to get track for export:', error)
      errorCount.value++
    }

    await delay(1)
  }

  // Export complete
  if (isExporting.value) {
    isExporting.value = false
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

const setupEventListeners = async () => {
  // Clean up existing listeners
  if (unlistenProgress) {
    await unlistenProgress()
    unlistenProgress = null
  }
  if (unlistenComplete) {
    await unlistenComplete()
    unlistenComplete = null
  }

  // Listen for export progress events from backend
  unlistenProgress = await listen('export-progress', event => {
    const { trackId, status, message } = event.payload
    // Frontend already handles progress via exportNext loop
    // This is for real-time updates if backend handles the loop
  })

  unlistenComplete = await listen('export-complete', event => {
    const { exported, skipped, errors } = event.payload
    isExporting.value = false
    toast.success(`Export complete: ${exported} exported, ${skipped} skipped, ${errors} errors`)
  })
}

const cleanupEventListeners = async () => {
  if (unlistenProgress) {
    await unlistenProgress()
    unlistenProgress = null
  }
  if (unlistenComplete) {
    await unlistenComplete()
    unlistenComplete = null
  }
}

const addToQueue = (trackIds, formats) => {
  isExporting.value = true
  exportFormats.value = formats

  for (let i = 0; i < trackIds.length; i++) {
    exportQueue.value.push(trackIds[i])
  }

  totalCount.value += trackIds.length

  console.log(`Added ${trackIds.length} tracks to export queue`)

  // Start exporting
  exportNext()
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
    setupEventListeners,
    cleanupEventListeners,
  }
}
