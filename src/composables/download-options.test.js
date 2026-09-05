import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('../components/library/track-list/TrackItem.vue', () => ({ default: {} }))
vi.mock('../components/library/MiniSearch.vue', () => ({ default: {} }))
beforeEach(() => {
  vi.resetModules()
  vi.resetAllMocks()
})

const config = {
  skip_tracks_with_synced_lyrics: true,
  skip_tracks_with_plain_lyrics: false,
  auto_export_enabled: true,
  export_txt: false,
  export_lrc: true,
  export_embedded: true,
  try_embed_lyrics: true,
}

const mountedApps = []
afterEach(() => mountedApps.splice(0).forEach(app => app.unmount()))

async function setupComponent(component, props) {
  const { createRenderer, ssrContextKey } = await import('vue')
  let state
  // Render only a comment: exercise live setup/watch/unmount without a DOM dependency.
  const renderer = createRenderer({
    createComment: () => ({}),
    insert() {},
    remove() {},
    parentNode: () => null,
    nextSibling: () => null,
  })
  const app = renderer.createApp({
    setup() {
      state = component.setup(props, { expose: vi.fn(), emit: vi.fn() })
      return () => null
    },
  })
  app.provide(ssrContextKey, { modules: new Set() })
  app.mount({})
  mountedApps.push(app)
  await Promise.resolve()
  return state
}

describe('shared Download targets', () => {
  it('opens the library target from Download All and cancels when its trigger unmounts', async () => {
    invoke.mockResolvedValue(config)
    const { default: LibraryHeader } = await import('../components/library/LibraryHeader.vue')
    const state = await setupComponent(LibraryHeader, { activeTab: 'tracks' })
    const options = (await import('./download-options.js')).useDownloadOptions()
    await state.openDownloadOptions({ currentTarget: {} })
    expect(options.request.value.target).toEqual({ type: 'library' })
    mountedApps.pop().unmount()
    expect(options.request.value).toBeNull()
    expect(invoke.mock.calls.map(([command]) => command)).toEqual(['get_config'])
  })

  it.each([
    ['album row', 'album', 'AlbumItem', 'downloadLyricsMultiple', true],
    ['artist row', 'artist', 'ArtistItem', 'downloadLyricsMultiple', true],
    ['album detail', 'album', 'AlbumTrackList', 'downloadAlbumLyrics', false],
    ['artist detail', 'artist', 'ArtistTrackList', 'downloadArtistLyrics', false],
  ])(
    '%s confirms only its captured target and dismisses recycled drafts',
    async (_, type, name, handler, row) => {
      const { reactive, nextTick } = await import('vue')
      const component = await import(`../components/library/${type}-list/${name}.vue`)
      const target = reactive({ id: 7, name: 'Original target' })
      const props = reactive(row ? { [`${type}Id`]: 7 } : { [type]: target })
      invoke.mockImplementation(async command => {
        if (command === `get_${type}`) return target
        return command === 'get_config' ? config : [70]
      })
      const state = await setupComponent(component.default, props)
      if (row) state[type].value = target
      const options = (await import('./download-options.js')).useDownloadOptions()
      invoke.mockClear()
      const anchor = {}
      await state[handler]({ currentTarget: anchor })
      expect(options.request.value.target).toEqual({ type, id: 7, name: 'Original target' })
      expect(invoke.mock.calls.map(([command]) => command)).toEqual(['get_config'])

      // Recycling/retargeting before confirmation dismisses the popup with no writes.
      if (row) props[`${type}Id`] = 8
      else props[type] = { id: 8, name: 'Next target' }
      await nextTick()
      expect(options.request.value).toBeNull()
      expect(invoke.mock.calls.some(([command]) => command === 'set_export_preferences')).toBe(
        false
      )

      if (row) {
        props[`${type}Id`] = 7
        state[type].value = target
      } else props[type] = target
      await nextTick()
      if (row) state[type].value = target
      await state[handler]({ currentTarget: anchor })
      options.downloadDraft.value.downloadLyricsFor = 'skipPlain'
      let finishSave
      invoke.mockImplementation(async command => {
        if (command === 'set_export_preferences')
          return new Promise(resolve => {
            finishSave = resolve
          })
        return [70]
      })
      const submission = options.downloadAllLyrics()
      target.id = 99
      target.name = 'Recycled during save'
      finishSave(config)
      await submission
      expect(invoke).toHaveBeenLastCalledWith(`get_${type}_track_ids`, {
        [`${type}Id`]: 7,
        withoutPlainLyrics: true,
        withoutSyncedLyrics: true,
      })
      const queue = (await import('./downloader.js')).useDownloader().downloadQueue.value
      expect(queue).toEqual([
        {
          trackId: 70,
          autoExport: {
            plainText: false,
            syncedLrc: true,
            embedIntoTrack: true,
          },
        },
      ])
    }
  )

  it.each([
    ['all', false, false],
    ['skipSynced', true, false],
    ['skipPlain', true, true],
  ])('maps %s to library, album and artist queries', async (mode, synced, plain) => {
    const options = (await import('./download-options.js')).useDownloadOptions()
    invoke.mockImplementation(async command => (command === 'get_config' ? config : []))
    for (const type of ['library', 'album', 'artist']) {
      await options.open({ type, id: 3, name: 'Target' }, {})
      options.downloadDraft.value.downloadLyricsFor = mode
      await options.downloadAllLyrics()
      if (type === 'library') {
        expect(invoke).toHaveBeenLastCalledWith('get_track_ids', {
          searchQuery: '',
          syncedLyricsTracks: !synced,
          plainLyricsTracks: !plain,
          instrumentalTracks: !synced && !plain,
          noLyricsTracks: true,
        })
      } else {
        expect(invoke).toHaveBeenLastCalledWith(`get_${type}_track_ids`, {
          [`${type}Id`]: 3,
          withoutPlainLyrics: plain,
          withoutSyncedLyrics: synced,
        })
      }
    }
  })

  it('ignores late hydration and old close events after a new target opens', async () => {
    const options = (await import('./download-options.js')).useDownloadOptions()
    let firstRead
    invoke.mockImplementationOnce(
      () =>
        new Promise(resolve => {
          firstRead = resolve
        })
    )
    const first = options.open({ type: 'album', id: 1, name: 'First' }, {})
    const old = options.request.value
    invoke.mockResolvedValue(config)
    await options.open({ type: 'artist', id: 2, name: 'Second' }, {})
    firstRead({ ...config, export_lrc: false })
    await first
    options.close(old)
    expect(options.request.value.target.id).toBe(2)
    expect(options.downloadDraft.value.syncedLrc).toBe(true)
    options.close()
    await options.downloadAllLyrics()
    expect(invoke.mock.calls.every(([command]) => command === 'get_config')).toBe(true)
  })
})
