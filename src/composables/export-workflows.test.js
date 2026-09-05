import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('../components/library/MiniSearch.vue', () => ({ default: {} }))

beforeEach(() => {
  vi.resetModules()
  vi.resetAllMocks()
  vi.useFakeTimers()
})
afterEach(() => vi.useRealTimers())

async function popupState() {
  const { createSSRApp } = await import('vue')
  const { renderToString } = await import('vue/server-renderer')
  const { default: LibraryHeader } = await import('../components/library/LibraryHeader.vue')
  let state
  await renderToString(
    createSSRApp({
      setup() {
        state = LibraryHeader.setup({ activeTab: 'tracks' }, { expose: vi.fn(), emit: vi.fn() })
        return () => null
      },
    })
  )
  const options = (await import('./download-options.js')).useDownloadOptions()
  return {
    ...state,
    ...options,
    openDownloadOptions: () => {
      options.close()
      return options.open({ type: 'library' }, {})
    },
  }
}

describe('submitted export choices', () => {
  it('shares embed-only choices, gates execution, and preserves a disabled remembered choice', async () => {
    const state = await popupState()
    let config = {
      skip_tracks_with_synced_lyrics: true,
      skip_tracks_with_plain_lyrics: false,
      auto_export_enabled: true,
      export_txt: false,
      export_lrc: false,
      export_embedded: true,
      try_embed_lyrics: true,
    }
    invoke.mockImplementation(async (command, changes) => {
      if (command === 'get_track_ids') return [1]
      if (command === 'set_export_preferences' && 'exportEmbedded' in changes) {
        config = { ...config, export_embedded: changes.exportEmbedded }
      }
      return config
    })
    await state.openDownloadOptions()
    expect(state.canDownload.value).toBe(true)
    state.downloadDraft.value.embedIntoTrack = false
    expect(state.canDownload.value).toBe(false)
    await state.openDownloadOptions() // Discard the unchecked draft.
    expect(state.downloadDraft.value.embedIntoTrack).toBe(true)
    await state.downloadAllLyrics()
    expect(invoke).toHaveBeenCalledWith('set_export_preferences', {
      autoExportEnabled: true,
      exportTxt: false,
      exportLrc: false,
      exportEmbedded: true,
      skipTracksWithSyncedLyrics: true,
      skipTracksWithPlainLyrics: false,
    })
    const downloader = (await import('./downloader.js')).useDownloader()
    expect(downloader.downloadQueue.value[0].autoExport).toEqual({
      plainText: false,
      syncedLrc: false,
      embedIntoTrack: true,
    })
    await state.openExportOptions()
    expect(state.exportDraft.value.embedIntoTrack).toBe(true)

    config = { ...config, try_embed_lyrics: false }
    await state.openDownloadOptions()
    expect(state.downloadDraft.value.embedIntoTrack).toBe(true)
    expect(state.canDownload.value).toBe(false)
    state.downloadDraft.value.syncedLrc = true
    invoke.mockClear()
    await state.downloadAllLyrics()
    expect(
      invoke.mock.calls.find(([command]) => command === 'set_export_preferences')[1]
    ).not.toHaveProperty('exportEmbedded')
    expect(downloader.downloadQueue.value[1].autoExport.embedIntoTrack).toBe(false)
    expect(downloader.downloadQueue.value[0].autoExport.embedIntoTrack).toBe(true)
    await state.openExportOptions()
    expect(state.exportDraft.value.embedIntoTrack).toBe(true)
    expect(state.canExport.value).toBe(false)
  })

  it('submits the popup filter snapshot and reloads discarded drafts from backend settings', async () => {
    const state = await popupState()
    const config = {
      skip_tracks_with_synced_lyrics: true,
      skip_tracks_with_plain_lyrics: false,
      auto_export_enabled: false,
      export_txt: false,
      export_lrc: true,
    }
    invoke.mockResolvedValue(config)
    await state.openDownloadOptions()
    expect(state.downloadDraft.value.downloadLyricsFor).toBe('skipSynced')
    state.downloadDraft.value.downloadLyricsFor = 'all'
    await state.openDownloadOptions() // Reopening discards unsubmitted choices.
    expect(state.downloadDraft.value.downloadLyricsFor).toBe('skipSynced')
    expect(invoke.mock.calls.every(([command]) => command === 'get_config')).toBe(true)

    let finishSave
    invoke.mockImplementation(async command => {
      if (command === 'set_export_preferences') {
        return new Promise(resolve => {
          finishSave = resolve
        })
      }
      if (command === 'get_track_ids') return []
    })
    state.downloadDraft.value.downloadLyricsFor = 'skipPlain'
    const submission = state.downloadAllLyrics()
    state.downloadDraft.value.downloadLyricsFor = 'all'
    finishSave(config) // A stale response must not override the submitted selection.
    await submission
    expect(invoke).toHaveBeenCalledWith('set_export_preferences', {
      autoExportEnabled: false,
      skipTracksWithSyncedLyrics: true,
      skipTracksWithPlainLyrics: true,
    })
    expect(invoke).toHaveBeenLastCalledWith('get_track_ids', {
      searchQuery: '',
      syncedLyricsTracks: false,
      plainLyricsTracks: false,
      instrumentalTracks: false,
      noLyricsTracks: true,
    })
  })

  it('isolates download batches and keeps an export failure in the existing success log', async () => {
    const downloader = (await import('./downloader.js')).useDownloader()
    const message = 'Synced lyrics downloaded; LRC export failed: permission denied'
    invoke.mockImplementation(async (command, args) => {
      if (command === 'get_track') return { id: args.trackId, title: 'Song', artist_name: 'Artist' }
      if (args.trackId === 2) throw 'Not found'
      return message
    })
    const choice = { plainText: true, syncedLrc: false, embedIntoTrack: true }
    downloader.addToQueue([1, 2], choice)
    choice.plainText = false
    choice.syncedLrc = true
    choice.embedIntoTrack = false
    downloader.addToQueue([3], choice)
    downloader.addToQueue([4]) // Other download entry points do not inherit remembered formats.
    void downloader.downloadNext()
    await vi.advanceTimersByTimeAsync(5)
    const calls = invoke.mock.calls
      .filter(([name]) => name === 'download_lyrics')
      .map(([, args]) => args)
    expect(calls).toEqual([
      { trackId: 1, autoExport: { plainText: true, syncedLrc: false, embedIntoTrack: true } },
      { trackId: 2, autoExport: { plainText: true, syncedLrc: false, embedIntoTrack: true } },
      { trackId: 3, autoExport: { plainText: false, syncedLrc: true, embedIntoTrack: false } },
      { trackId: 4, autoExport: null },
    ])
    expect(downloader.successCount.value).toBe(3)
    expect(downloader.failureCount.value).toBe(1)
    expect(downloader.downloadedCount.value).toBe(4)
    expect(downloader.log.value[0]).toMatchObject({ status: 'success', message })
  })

  it('keeps manual batch formats and writer order when later submissions change them', async () => {
    const exporter = (await import('./export.js')).useExporter()
    invoke.mockImplementation(async (command, args) =>
      command === 'get_track'
        ? { id: args.trackId, title: 'Song', artist_name: 'Artist' }
        : { errors: 0, exported: 1, skipped: 0 }
    )
    const choice = { plainText: true, syncedLrc: true, embedIntoTrack: true }
    exporter.addToQueue([1, 2], choice)
    choice.plainText = false
    choice.embedIntoTrack = false
    exporter.addToQueue([3], choice)
    void exporter.exportNext()
    await vi.advanceTimersByTimeAsync(4)
    expect(
      invoke.mock.calls.filter(([name]) => name === 'export_track_lyrics').map(([, args]) => args)
    ).toEqual([
      { trackId: 1, formats: ['txt', 'lrc', 'embedded'] },
      { trackId: 2, formats: ['txt', 'lrc', 'embedded'] },
      { trackId: 3, formats: ['lrc'] },
    ])
  })

  it('shares only backend-confirmed preferences across consumers and preserves them on save error', async () => {
    const { useExportPreferences } = await import('./export-preferences.js')
    const first = useExportPreferences()
    const second = useExportPreferences()
    invoke.mockResolvedValueOnce({ auto_export_enabled: false, export_lrc: true })
    await first.refresh()
    expect(second.preferences.value.export_lrc).toBe(true)
    invoke.mockResolvedValueOnce({
      auto_export_enabled: false,
      export_lrc: false,
      export_txt: true,
    })
    await first.save({ exportLrc: false, exportTxt: true })
    expect(second.preferences.value.export_txt).toBe(true)
    expect(invoke).toHaveBeenLastCalledWith('set_export_preferences', {
      exportLrc: false,
      exportTxt: true,
    })
    invoke.mockRejectedValueOnce('Database unavailable')
    await expect(second.save({ autoExportEnabled: true })).rejects.toBe('Database unavailable')
    expect(first.preferences.value.auto_export_enabled).toBe(false)
  })
})
