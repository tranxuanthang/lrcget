import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const createDocument = async ({ lyricsfileContent = '' } = {}) => {
  const { useEditLyricsV2Document } = await import('./useEditLyricsV2Document.js')
  return useEditLyricsV2Document({
    audioSource: ref(null),
    lyricsfile: ref(lyricsfileContent ? { content: lyricsfileContent } : ''),
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

  it('highlights a dragged range without changing the active line until release', async () => {
    const { updateSyncedLines, selectSyncedLine, selectSyncedLineRange, activateSyncedLine, syncedLines, selectedSyncedLineIndex, selectedSyncedLineIndices } =
      await createDocument()

    updateSyncedLines([
      { text: 'one', start_ms: 1000, end_ms: 2000 },
      { text: 'two', start_ms: 2000, end_ms: 3000 },
      { text: 'three', start_ms: 3000, end_ms: 4000 },
    ])
    selectSyncedLine(0)

    selectSyncedLineRange(1, 2)

    expect(selectedSyncedLineIndex.value).toBe(0)
    expect(selectedSyncedLineIndices.value).toEqual([1, 2])
    expect(syncedLines.value[0].text).toBe('one')

    activateSyncedLine(1)

    expect(selectedSyncedLineIndex.value).toBe(1)
    expect(selectedSyncedLineIndices.value).toEqual([1, 2])
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

describe('splitSyncedLineAtCaret', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  it('splits the active line at the cursor and divides time by word count', async () => {
    const { updateSyncedLines, splitSyncedLineAtCaret, syncedLines, selectedSyncedLineIndex } =
      await createDocument()
    const text = "I got sunshine in a bag I'm useless, but not for long"

    updateSyncedLines([{ text, start_ms: 10000, end_ms: 16000 }])

    const result = splitSyncedLineAtCaret(0, text, text.indexOf("I'm"))

    expect(result.text).toBe("I'm useless, but not for long")
    expect(syncedLines.value.map(line => line.text)).toEqual([
      'I got sunshine in a bag',
      "I'm useless, but not for long",
    ])
    expect(syncedLines.value[0]).toMatchObject({ start_ms: 10000, end_ms: 13000 })
    expect(syncedLines.value[1]).toMatchObject({ start_ms: 13000, end_ms: 16000 })
    expect(selectedSyncedLineIndex.value).toBe(1)
  })

  it('keeps word timings when the caret is on a word boundary', async () => {
    const { updateSyncedLines, splitSyncedLineAtCaret, syncedLines } = await createDocument()
    const text = 'hello world'

    updateSyncedLines([
      {
        text,
        start_ms: 0,
        end_ms: 1000,
        words: [
          { text: 'hello ', start_ms: 0 },
          { text: 'world', start_ms: 500, end_ms: 1000 },
        ],
      },
    ])

    splitSyncedLineAtCaret(0, text, text.indexOf('world'))

    expect(syncedLines.value[0]).toMatchObject({
      text: 'hello',
      start_ms: 0,
      end_ms: 500,
    })
    expect(syncedLines.value[0].words).toEqual([{ text: 'hello', start_ms: 0, end_ms: 500 }])
    expect(syncedLines.value[1]).toMatchObject({
      text: 'world',
      start_ms: 500,
      end_ms: 1000,
    })
    expect(syncedLines.value[1].words).toEqual([{ text: 'world', start_ms: 500, end_ms: 1000 }])
  })

  it('does not split when the caret leaves one side empty', async () => {
    const { updateSyncedLines, splitSyncedLineAtCaret, syncedLines } = await createDocument()

    updateSyncedLines([{ text: 'only', start_ms: 0, end_ms: 1000 }])

    expect(splitSyncedLineAtCaret(0, 'only', 0)).toBeNull()
    expect(splitSyncedLineAtCaret(0, 'only', 4)).toBeNull()
    expect(syncedLines.value).toHaveLength(1)
  })
})

describe('plain lyrics from synced lines', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  const lyricsfileContent = [
    "version: '1.0'",
    'metadata:',
    '  title: Lost',
    '  artist: Culture Shock',
    '  instrumental: false',
    'lines:',
    '  - text: If you feel alone',
    '    start_ms: 330',
    '    end_ms: 2923',
    '  - text: Let me know',
    '    start_ms: 3295',
    '    end_ms: 5049',
    'plain: "If you feel alone, let me know"',
  ].join('\n')

  it('keeps the stored plain lyrics until a synced line is edited', async () => {
    const { initializeLyrics, plainLyrics, rewindLineBy100 } = await createDocument({
      lyricsfileContent,
    })

    initializeLyrics()

    expect(plainLyrics.value).toBe('If you feel alone, let me know')

    rewindLineBy100(0)

    expect(plainLyrics.value).toBe('If you feel alone\nLet me know')
  })

  it('rebuilds plain lyrics when a synced line is split', async () => {
    const { initializeLyrics, plainLyrics, splitSyncedLineAtCaret } = await createDocument({
      lyricsfileContent,
    })

    initializeLyrics()
    splitSyncedLineAtCaret(0, 'If you feel alone', 'If you feel '.length)

    expect(plainLyrics.value).toBe('If you feel\nalone\nLet me know')
  })

  it('uses joined word text for a word-synced line', async () => {
    const { updateSyncedLines, updateLineWords, plainLyrics } = await createDocument()

    updateSyncedLines([{ text: 'hello world', start_ms: 0, end_ms: 1000 }])
    updateLineWords({
      lineIndex: 0,
      words: [
        { text: 'hello ', start_ms: 0 },
        { text: 'there', start_ms: 500 },
      ],
    })

    expect(plainLyrics.value).toBe('hello there')
  })
})
