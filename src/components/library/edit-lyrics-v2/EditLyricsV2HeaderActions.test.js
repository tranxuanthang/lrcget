import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createSSRApp, h } from 'vue'
import { renderToString } from 'vue/server-renderer'
import HeaderActions from './EditLyricsV2HeaderActions.vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
beforeEach(() => vi.resetAllMocks())

const popup = {
  setup(_props, { slots }) {
    return () => h('div', [slots.default?.(), slots.popper?.()])
  },
}

describe('editor export defaults', () => {
  it('selects synced LRC, not plain lyrics or embedding, without exporting or publishing', async () => {
    const app = createSSRApp(HeaderActions, { isDirty: false, isExporting: false })
    app.component('VTooltip', popup)
    app.component('VDropdown', popup)
    app.directive('close-popper', {})
    const html = await renderToString(app)
    const input = id => html.match(new RegExp(`<input[^>]*id="${id}"[^>]*>`))[0]
    expect(input('export-synced-lrc')).toContain('checked')
    expect(input('export-plain-text')).not.toContain('checked')
    expect(input('embed-into-track')).not.toContain('checked')
    const { invoke } = await import('@tauri-apps/api/core')
    expect(invoke).not.toHaveBeenCalled()
  })

  it('loads enabled embedding once and preserves checkbox edits when the menu reopens', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    invoke.mockResolvedValue({ try_embed_lyrics: true, export_embedded: true })
    let state
    const emit = vi.fn()
    await renderToString(
      createSSRApp({
        setup() {
          state = HeaderActions.setup(
            { isDirty: false, isExporting: false },
            { expose: vi.fn(), emit }
          )
          return () => null
        },
      })
    )
    await state.refreshEmbedConfig()
    expect(state.embedIntoTrack.value).toBe(true)
    state.handleExportClick()
    expect(emit).toHaveBeenCalledWith('export', {
      plainText: false,
      syncedLrc: true,
      embedIntoTrack: true,
    })
    state.embedIntoTrack.value = false
    await state.refreshEmbedConfig()
    expect(state.embedIntoTrack.value).toBe(false)
    expect(emit).toHaveBeenCalledTimes(1)
  })

  it('does not submit embedding when the experimental setting is disabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    invoke.mockResolvedValue({ try_embed_lyrics: false, export_embedded: true })
    let state
    const emit = vi.fn()
    await renderToString(
      createSSRApp({
        setup() {
          state = HeaderActions.setup(
            { isDirty: false, isExporting: false },
            { expose: vi.fn(), emit }
          )
          return () => null
        },
      })
    )
    await state.refreshEmbedConfig()
    expect(state.embedIntoTrack.value).toBe(false)
    state.embedIntoTrack.value = true
    state.handleExportClick()
    expect(emit).toHaveBeenCalledWith('export', {
      plainText: false,
      syncedLrc: true,
      embedIntoTrack: false,
    })
    state.exportSyncedLrc.value = false
    expect(state.hasSelectedExportFormat.value).toBe(false)
  })
})
