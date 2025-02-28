import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'vue-toastification'

const delay = (time) => new Promise((resolve) => setTimeout(resolve, time))

// State management
const downloadQueue = ref([])
const downloadedItems = ref([])
const currentItem = ref(null)
const log = ref([])
const successCount = ref(0)
const failureCount = ref(0)
const isDownloading = ref(false)
const totalCount = ref(0)
const isPaused = ref(false)
const downloadSpeed = ref(0) // downloads per minute
const estimatedTimeRemaining = ref(0)
let downloadLoop = null
let lastDownloadTimestamps = []

const downloadedCount = computed(() => {
  return downloadedItems.value.length
})

const addLog = (logObj) => {
  const timestamp = new Date().toISOString()
  log.value.unshift({ ...logObj, timestamp })
  if (log.value.length > 1000) {
    log.value.pop()
  }
}

const updateDownloadStats = () => {
  const now = Date.now()

  // Keep last 10 download timestamps for moving average
  lastDownloadTimestamps.push(now)
  if (lastDownloadTimestamps.length > 10) {
    lastDownloadTimestamps.shift()
  }

  if (lastDownloadTimestamps.length >= 2) {
    const oldestTimestamp = lastDownloadTimestamps[0]
    const timeSpanMinutes = (now - oldestTimestamp) / 1000 / 60
    const completedDownloads = lastDownloadTimestamps.length - 1

    downloadSpeed.value = timeSpanMinutes > 0 ? completedDownloads / timeSpanMinutes : 0

    // Estimate remaining time
    const remainingItems = totalCount.value - downloadedCount.value
    if (downloadSpeed.value > 0) {
      estimatedTimeRemaining.value = remainingItems / downloadSpeed.value
    }
  }
}

const downloadLyrics = async (track) => {
  try {
    const result = await invoke('download_lyrics', { trackId: track.id })

    if (!isDownloading.value) {
      return
    }

    addLog({ status: 'success', title: track.title, artistName: track.artist_name, message: result })
    successCount.value++
    updateDownloadStats()
  } catch (error) {
    if (!isDownloading.value) {
      return
    }

    addLog({ status: 'failure', title: track.title, artistName: track.artist_name, message: error })
    failureCount.value++
  }

  downloadedItems.value.push(currentItem.value)
  currentItem.value = null
}

const downloadNext = async () => {
  downloadLoop = async () => {
    while (isDownloading.value) {
      if (isPaused.value) {
        await delay(500)
        continue
      }

      if (downloadQueue.value.length === 0) {
        if (downloadedCount.value >= totalCount.value && totalCount.value > 0) {
          // All downloads completed
          const toast = useToast()
          toast.success(`Download completed: ${successCount.value} succeeded, ${failureCount.value} failed`)
          isDownloading.value = false
          break
        }
        await delay(500)
        continue
      }

      const trackId = downloadQueue.value.shift()
      try {
        const track = await invoke('get_track', { trackId: trackId })
        currentItem.value = track
        await downloadLyrics(track)
      } catch (error) {
        addLog({ status: 'error', message: `Failed to get track info for ID: ${trackId}`, error: error.toString() })
        failureCount.value++
      }
    }
  }

  // Start the download loop
  downloadLoop()
}

const downloadProgress = computed(() => {
  if (!totalCount.value) {
    return 0.0
  }

  if (downloadedCount.value >= totalCount.value) {
    return 1.0
  }

  return downloadedCount.value / totalCount.value
})

const remainingTime = computed(() => {
  if (estimatedTimeRemaining.value <= 0 || !isDownloading.value || isPaused.value) {
    return 'Unknown'
  }

  const minutes = Math.floor(estimatedTimeRemaining.value)
  const seconds = Math.round((estimatedTimeRemaining.value - minutes) * 60)

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`
  } else {
    return `${seconds}s`
  }
})

const addToQueue = (trackIds) => {
  if (!Array.isArray(trackIds)) {
    trackIds = [trackIds]
  }

  const uniqueIds = [...new Set(trackIds)].filter(id => {
    // Filter out already downloaded or queued tracks
    const isDownloaded = downloadedItems.value.some(item => item.id === id)
    const isQueued = downloadQueue.value.includes(id)
    return !isDownloaded && !isQueued
  })

  if (uniqueIds.length === 0) {
    return 0
  }

  for (let i = 0; i < uniqueIds.length; i++) {
    downloadQueue.value.push(uniqueIds[i])
  }

  totalCount.value += uniqueIds.length

  if (!isDownloading.value) {
    isDownloading.value = true
    downloadNext()
  }

  console.log(`Added ${uniqueIds.length} tracks to download queue`)
  return uniqueIds.length
}

const pauseDownloading = () => {
  isPaused.value = true
}

const resumeDownloading = () => {
  isPaused.value = false
}

const startOver = () => {
  downloadedItems.value = []
  log.value = []
  successCount.value = 0
  failureCount.value = 0
  totalCount.value = 0
  isDownloading.value = false
  isPaused.value = false
  lastDownloadTimestamps = []
  downloadSpeed.value = 0
  estimatedTimeRemaining.value = 0
}

const stopDownloading = () => {
  downloadQueue.value = []
  isDownloading.value = false
  isPaused.value = false
}

// Save download history to local storage on changes
const saveHistory = () => {
  try {
    localStorage.setItem('lrcget_download_history', JSON.stringify({
      downloadedItems: downloadedItems.value,
      log: log.value,
      successCount: successCount.value,
      failureCount: failureCount.value,
      timestamp: new Date().toISOString()
    }))
  } catch (e) {
    console.error('Failed to save download history to local storage', e)
  }
}

// Load download history from local storage on init
const loadHistory = () => {
  try {
    const savedHistory = localStorage.getItem('lrcget_download_history')
    if (savedHistory) {
      const parsed = JSON.parse(savedHistory)
      if (parsed.downloadedItems) downloadedItems.value = parsed.downloadedItems
      if (parsed.log) log.value = parsed.log
      if (parsed.successCount) successCount.value = parsed.successCount
      if (parsed.failureCount) failureCount.value = parsed.failureCount
    }
  } catch (e) {
    console.error('Failed to load download history from local storage', e)
  }
}

// Watch for changes to save to localStorage
watch([downloadedItems, log], () => {
  saveHistory();
}, { deep: true });

export function useDownloader() {
  // Load history on first use
  loadHistory();

  return {
    isDownloading,
    isPaused,
    downloadQueue,
    downloadedItems,
    downloadProgress,
    successCount,
    failureCount,
    totalCount,
    downloadedCount,
    downloadSpeed,
    remainingTime,
    log,
    currentItem,
    addToQueue,
    startOver,
    stopDownloading,
    pauseDownloading,
    resumeDownloading,
    downloadNext,
  }
}
