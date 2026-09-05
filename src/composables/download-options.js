import { computed, ref, shallowRef, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useExportPreferences } from './export-preferences.js'
import { useDownloader } from './downloader.js'
import { downloadModeFromConfig, downloadFlagsForMode } from '@/utils/download-filter.js'

// One workflow/popup for the library, virtualized rows, and detail views.
const request = shallowRef(null)
const emptyDraft = () => ({
  downloadLyricsFor: 'all',
  autoExportEnabled: false,
  plainText: false,
  syncedLrc: true,
  embedIntoTrack: false,
})
const downloadDraft = ref(emptyDraft())
const tryEmbedLyrics = ref(false)
const downloadLoading = ref(false)
const submittingDownload = ref(false)
const isBuildingQueue = ref(false)
const downloadError = ref('')
let nextToken = 0

const canDownload = computed(
  () =>
    !!request.value &&
    !downloadLoading.value &&
    !submittingDownload.value &&
    !downloadError.value &&
    (!downloadDraft.value.autoExportEnabled ||
      downloadDraft.value.plainText ||
      downloadDraft.value.syncedLrc ||
      (downloadDraft.value.embedIntoTrack && tryEmbedLyrics.value))
)

const close = (session = request.value) => {
  if (request.value !== session) return
  request.value = null
  downloadDraft.value = emptyDraft()
  downloadError.value = ''
  downloadLoading.value = false
}

const open = async (target, anchor) => {
  if (!target || !anchor || submittingDownload.value) return
  if (request.value?.anchor === anchor) {
    close()
    return
  }
  const session = Object.freeze({
    token: ++nextToken,
    target: Object.freeze({ ...target }),
    anchor,
  })
  request.value = session
  downloadDraft.value = emptyDraft()
  downloadLoading.value = true
  downloadError.value = ''
  try {
    const config = await useExportPreferences().refresh()
    if (request.value !== session) return
    downloadDraft.value = {
      downloadLyricsFor: downloadModeFromConfig(config),
      autoExportEnabled: config.auto_export_enabled,
      plainText: config.export_txt,
      syncedLrc: config.export_lrc,
      embedIntoTrack: config.export_embedded,
    }
    tryEmbedLyrics.value = config.try_embed_lyrics
  } catch (error) {
    if (request.value === session) downloadError.value = String(error)
  } finally {
    if (request.value === session) downloadLoading.value = false
  }
}

const downloadAllLyrics = async () => {
  if (!canDownload.value) return
  submittingDownload.value = true
  const session = request.value
  const draft = { ...downloadDraft.value }
  const filter = downloadFlagsForMode(draft.downloadLyricsFor)
  const embedEnabled = tryEmbedLyrics.value
  try {
    await useExportPreferences().save({
      ...filter,
      autoExportEnabled: draft.autoExportEnabled,
      ...(draft.autoExportEnabled
        ? {
            exportTxt: draft.plainText,
            exportLrc: draft.syncedLrc,
            ...(embedEnabled ? { exportEmbedded: draft.embedIntoTrack } : {}),
          }
        : {}),
    })
    isBuildingQueue.value = true
    const { type, id } = session.target
    let ids
    if (type === 'library') {
      ids = await invoke('get_track_ids', {
        searchQuery: '',
        syncedLyricsTracks: !filter.skipTracksWithSyncedLyrics,
        plainLyricsTracks: !filter.skipTracksWithPlainLyrics,
        instrumentalTracks: !filter.skipTracksWithSyncedLyrics && !filter.skipTracksWithPlainLyrics,
        noLyricsTracks: true,
      })
    } else {
      ids = await invoke(type === 'album' ? 'get_album_track_ids' : 'get_artist_track_ids', {
        [type === 'album' ? 'albumId' : 'artistId']: id,
        withoutPlainLyrics: filter.skipTracksWithPlainLyrics,
        withoutSyncedLyrics: filter.skipTracksWithSyncedLyrics,
      })
    }
    useDownloader().addToQueue(
      ids,
      draft.autoExportEnabled
        ? {
            plainText: draft.plainText,
            syncedLrc: draft.syncedLrc,
            embedIntoTrack: draft.embedIntoTrack && embedEnabled,
          }
        : null
    )
    close(session)
  } catch (error) {
    if (request.value === session) downloadError.value = String(error)
  } finally {
    submittingDownload.value = false
    isBuildingQueue.value = false
  }
}

export function useDownloadOptions() {
  return {
    request,
    downloadDraft,
    tryEmbedLyrics,
    downloadLoading,
    submittingDownload,
    isBuildingQueue,
    downloadError,
    canDownload,
    open,
    close,
    downloadAllLyrics,
  }
}

// Triggers own no popup/draft. Recycling or unmounting a trigger dismisses its session.
export function useDownloadTrigger(getTarget) {
  let anchor
  const dismiss = () => {
    if (anchor && request.value?.anchor === anchor) close()
  }
  watch(() => getTarget()?.id, dismiss, { flush: 'sync' })
  onBeforeUnmount(dismiss)
  return event => {
    anchor = event.currentTarget
    return open(getTarget(), anchor)
  }
}
