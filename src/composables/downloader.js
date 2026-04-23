import { computed, markRaw, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const delay = time => new Promise((resolve, reject) => setTimeout(resolve, time))

const downloadQueue = ref([])
const log = ref([])
const successCount = ref(0)
const failureCount = ref(0)
const downloadedCount = ref(0)
const isDownloading = ref(false)
const totalCount = ref(0)

const addLog = logObj => {
  log.value.unshift(markRaw(logObj))
  if (log.value.length > 100) {
    log.value.pop()
  }
}

const downloadLyrics = async track => {
  try {
    const result = await invoke('download_lyrics', { trackId: track.id })

    if (!isDownloading.value) {
      return
    }

    addLog({
      status: 'success',
      title: track.title,
      artistName: track.artist_name,
      message: result,
    })
    successCount.value++
  } catch (error) {
    if (!isDownloading.value) {
      return
    }

    addLog({ status: 'failure', title: track.title, artistName: track.artist_name, message: error })
    failureCount.value++
  }

  downloadedCount.value++
}

const downloadNext = async () => {
  while (true) {
    if (downloadQueue.value.length === 0) {
      await delay(1000)
      continue
    }

    const trackId = downloadQueue.value.shift()
    const track = await invoke('get_track', { trackId: trackId })
    await downloadLyrics(track)

    await delay(1)
  }
}

const downloadProgress = computed(() => {
  if (!isDownloading.value) {
    return 0.0
  }

  if (totalCount.value === 0) {
    return 0.0
  }

  if (downloadedCount.value >= totalCount.value) {
    return 1.0
  }

  return downloadedCount.value / totalCount.value
})

const addToQueue = trackIds => {
  isDownloading.value = true

  for (let i = 0; i < trackIds.length; i++) {
    downloadQueue.value.push(trackIds[i])
  }

  totalCount.value += trackIds.length

  console.log(`Added ${totalCount.value} tracks to download queue`)
}

const startOver = () => {
  downloadQueue.value = []
  log.value = []
  successCount.value = 0
  failureCount.value = 0
  downloadedCount.value = 0
  totalCount.value = 0
  isDownloading.value = false
}

const stopDownloading = () => {
  startOver()
}

export function useDownloader() {
  return {
    isDownloading,
    downloadQueue,
    downloadProgress,
    successCount,
    failureCount,
    totalCount,
    downloadedCount,
    log,
    addToQueue,
    startOver,
    stopDownloading,
    downloadNext,
  }
}
