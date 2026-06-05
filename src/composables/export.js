import { computed, markRaw, ref, shallowRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const delay = time => new Promise(resolve => setTimeout(resolve, time))

const exportQueue = ref([])
const log = shallowRef([])
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

let logIdCounter = 0

const addLog = logObj => {
  log.value = [{ id: ++logIdCounter, ...markRaw(logObj) }, ...log.value]
  if (log.value.length > 100) {
    log.value.pop()
  }
}

const exportTrack = async trackId => {
  try {
    const formats = []
    if (exportFormats.value.plainText) formats.push('txt')
    if (exportFormats.value.syncedLrc) formats.push('lrc')
    if (exportFormats.value.embedIntoTrack) formats.push('embedded')

    const result = await invoke('export_track_lyrics', {
      trackId,
      formats,
    })

    if (!isExporting.value) {
      return
    }

    // title/artistName come from the backend response now
    const title = result.title || 'Unknown'
    const artistName = result.artistName || ''

    const hasErrors = result.errors > 0
    const hasExported = result.exported > 0
    const hasSkipped = result.skipped > 0

    if (hasErrors) {
      addLog({
        status: 'error',
        title,
        artistName,
        message: result.message || 'Export failed',
      })
      errorCount.value++
    } else if (hasExported) {
      addLog({
        status: 'exported',
        title,
        artistName,
        message: result.message || `Exported to ${result.exported} format(s)`,
      })
      exportedCount.value++
    } else if (hasSkipped) {
      addLog({
        status: 'skipped',
        title,
        artistName,
        message: result.message || 'Skipped: no lyrics available for selected formats',
      })
      skippedCount.value++
    } else {
      addLog({
        status: 'skipped',
        title,
        artistName,
        message: result.message || 'Nothing to export',
      })
      skippedCount.value++
    }
  } catch (error) {
    if (!isExporting.value) {
      return
    }

    addLog({
      status: 'error',
      title: `Track #${trackId}`,
      artistName: '',
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

    const trackId = exportQueue.value.shift()
    await exportTrack(trackId)

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
  exportFormats.value = formats

  for (let i = 0; i < trackIds.length; i++) {
    exportQueue.value.push(trackIds[i])
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
