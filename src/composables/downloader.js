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
  isDownloading.value = true
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
  downloadNext()
}

const downloadNext = async () => {
  if (downloadQueue.value.length > 0) {
    const trackId = downloadQueue.value.shift()
    const track = await invoke('get_track', { trackId: trackId })
    currentItem.value = track
    downloadLyrics(track)
  } else {
    queueResolved.value = true
  }
}

const downloadProgress = computed(() => {
  if (!downloadQueue.value) {
    return 100.0
  }

  return downloadedCount.value / totalCount.value
})

const addToQueue = (trackIds) => {
  downloadQueue.value.push(...trackIds)
  totalCount.value += trackIds.length

  if (!currentItem.value) {
    downloadNext()
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
