import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const createDocument = async () => {
  const { useEditLyricsV2Document } = await import('./useEditLyricsV2Document.js')
  return useEditLyricsV2Document({
    audioSource: ref(null),
    lyricsfile: ref(''),
    trackId: ref(null),
    progress: ref(0),
    toast: { error: () => {}, success: () => {} },
  })
}

describe('addSyncedLineAt', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  it('copies start and end from the line above a between-line insert', async () => {
    const { updateSyncedLines, addSyncedLineAt, syncedLines } = await createDocument()

    updateSyncedLines([
      { text: 'one', start_ms: 1000, end_ms: 2500 },
      { text: 'two', start_ms: 2500, end_ms: 4000 },
    ])

    addSyncedLineAt(1, { copyTimestampsFrom: 0 })

    expect(syncedLines.value.map(line => line.text)).toEqual(['one', '', 'two'])
    expect(syncedLines.value[1]).toMatchObject({ start_ms: 1000, end_ms: 2500 })
  })

  it('still fills a trailing insert from the previous line end', async () => {
    const { updateSyncedLines, addSyncedLineAt, syncedLines } = await createDocument()

    updateSyncedLines([
      { text: 'one', start_ms: 1000, end_ms: 2500 },
      { text: 'two', start_ms: 2500, end_ms: 4000 },
    ])

    addSyncedLineAt(2)

    expect(syncedLines.value[2]).toMatchObject({ text: '', start_ms: 4000 })
    expect(syncedLines.value[2].end_ms).toBeUndefined()
  })
})
