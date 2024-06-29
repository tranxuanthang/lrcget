import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const downloadQueue = ref([])
const downloadedItems = ref([])
const currentItem = ref(null)
const log = ref([])
const queueResolved = ref(false)
const successCount = ref(0)
const failureCount = ref(0)
const isDownloading = ref(false)
const totalCount = ref(0)

const downloadedCount = computed(() => {
  return downloadedItems.value.length
})

const addLog = (logObj) => {
  log.value.unshift(logObj)
  if (log.value.length > 1000) {
    log.value.pop()
  }
}

const downloadLyrics = async (track) => {
  try {
    const result = await invoke('download_lyrics', { trackId: track.id })
    addLog({ status: 'success', title: track.title, artistName: track.artist_name, message: result })
    successCount.value++
  } catch (error) {
    addLog({ status: 'failure', title: track.title, artistName: track.artist_name, message: error })
    failureCount.value++
  }

  downloadedItems.value.push(currentItem.value)
  currentItem.value = null
}

const downloadNext = async () => {
  while (downloadQueue.value.length > 0) {
    const trackId = downloadQueue.value.shift()
    const track = await invoke('get_track', { trackId: trackId })
    currentItem.value = track
    await downloadLyrics(track)
  }
  queueResolved.value = true
}

const downloadProgress = computed(() => {
  if (!downloadQueue.value) {
    return 100.0
  }

  return downloadedCount.value / totalCount.value
})

const addToQueue = (trackIds) => {
  isDownloading.value = true

  for (let i = 0; i < trackIds.length; i++) {
    downloadQueue.value.push(trackIds[i])
  }

  totalCount.value += trackIds.length

  console.log(`Added ${totalCount.value} tracks to download queue`)

  // Defer the call to downloadNext using setTimeout
  if (!currentItem.value) {
    setTimeout(() => {
      downloadNext()
    }, 0)
  }
}

const startOver = () => {
  queueResolved.value = false
  downloadedItems.value = []
  log.value = []
  successCount.value = 0
  failureCount.value = 0
  totalCount.value = 0
  isDownloading.value = false
}

const stopDownloading = () => {
  downloadQueue.value = []
  queueResolved.value = false
  downloadedItems.value = []
  log.value = []
  successCount.value = 0
  failureCount.value = 0
  totalCount.value = 0
  isDownloading.value = false
}

export function useDownloader() {
  return {
    isDownloading,
    downloadQueue,
    downloadedItems,
    downloadProgress,
    successCount,
    failureCount,
    totalCount,
    downloadedCount,
    addToQueue,
    startOver,
    stopDownloading,
    log
  }
}
