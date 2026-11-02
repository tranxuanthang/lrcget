import { computed, markRaw, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const delay = time => new Promise(resolve => setTimeout(resolve, time))

const REQUEST_SPACING_MS = 500
const REQUEST_JITTER_MS = 250
const COOLDOWN_POLL_MS = 250

const downloadQueue = ref([])
const log = ref([])
const successCount = ref(0)
const failureCount = ref(0)
const downloadedCount = ref(0)
const isDownloading = ref(false)
const totalCount = ref(0)
const cooldownUntil = ref(0)
const cooldownRemaining = ref(0)

const addLog = logObj => {
  log.value.unshift(markRaw(logObj))
  if (log.value.length > 100) {
    log.value.pop()
  }
}

const normalizeError = error => {
  if (error && typeof error === 'object') {
    return {
      kind: error.kind ?? 'unknown',
      message: error.message ?? String(error),
      retryAfterSecs: Number(error.retryAfterSecs) || 0,
    }
  }

  return { kind: 'unknown', message: String(error), retryAfterSecs: 0 }
}

const setCooldown = seconds => {
  const duration = Number(seconds)
  if (!Number.isFinite(duration) || duration <= 0) {
    return
  }

  const until = Date.now() + duration * 1000
  if (until > cooldownUntil.value) {
    cooldownUntil.value = until
  }
}

const waitForCooldown = async () => {
  while (cooldownUntil.value > Date.now()) {
    cooldownRemaining.value = Math.ceil((cooldownUntil.value - Date.now()) / 1000)
    await delay(COOLDOWN_POLL_MS)
  }

  cooldownRemaining.value = 0
}

const requestSpacing = () => REQUEST_SPACING_MS + Math.floor(Math.random() * REQUEST_JITTER_MS)

const downloadLyrics = async (track, autoExport) => {
  try {
    const result = await invoke('download_lyrics', { trackId: track.id, autoExport })

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

    const normalized = normalizeError(error)
    if (
      (normalized.kind === 'rateLimited' || normalized.kind === 'serverOverloaded') &&
      normalized.retryAfterSecs > 0
    ) {
      setCooldown(normalized.retryAfterSecs)
    }

    addLog({
      status: 'failure',
      title: track.title,
      artistName: track.artist_name,
      message: normalized.message,
    })
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

    await waitForCooldown()

    const { trackId, autoExport } = downloadQueue.value.shift()
    const track = await invoke('get_track', { trackId: trackId })
    await downloadLyrics(track, autoExport)

    await delay(requestSpacing())
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

const addToQueue = (trackIds, autoExport = null) => {
  const snapshot = autoExport ? Object.freeze({ ...autoExport }) : null
  isDownloading.value = true

  for (let i = 0; i < trackIds.length; i++) {
    downloadQueue.value.push({ trackId: trackIds[i], autoExport: snapshot })
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
  cooldownUntil.value = 0
  cooldownRemaining.value = 0
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
    cooldownRemaining,
    log,
    addToQueue,
    startOver,
    stopDownloading,
    downloadNext,
  }
}
