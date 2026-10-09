import { afterEach, describe, expect, it, vi } from 'vitest'
import { computed, ref } from 'vue'
import { useEditLyricsV2WordBoundaryDrag } from './useEditLyricsV2WordBoundaryDrag.js'

const installDocument = () => {
  const previousDocument = globalThis.document
  const listeners = new Map()
  globalThis.document = {
    addEventListener(type, handler) {
      const list = listeners.get(type) ?? []
      list.push(handler)
      listeners.set(type, list)
    },
    removeEventListener(type, handler) {
      const list = listeners.get(type) ?? []
      listeners.set(
        type,
        list.filter(existing => existing !== handler)
      )
    },
  }

  return {
    emit(type, event) {
      for (const handler of [...(listeners.get(type) ?? [])]) {
        handler(event)
      }
    },
    restore() {
      if (previousDocument === undefined) {
        delete globalThis.document
      } else {
        globalThis.document = previousDocument
      }
    },
  }
}

const previewWords = () => [
  { text: 'hello ', start_ms: 0 },
  { text: 'world', start_ms: 500 },
]

const setupDrag = ({ hasPersistedWords = false, words = previewWords() } = {}) => {
  const updates = []
  const replays = []
  const drag = useEditLyricsV2WordBoundaryDrag({
    isWordSyncAvailable: computed(() => true),
    words: ref(words),
    lineStartMs: computed(() => 0),
    lineEndMs: computed(() => 1000),
    timelineStartMs: computed(() => -500),
    timelineEndMs: computed(() => 1500),
    selectedLineIndex: computed(() => 2),
    hasPersistedWords: computed(() => hasPersistedWords),
    onUpdateWords: payload => updates.push(payload),
    onWordTimingEdited: payload => replays.push(payload),
  })

  return { drag, updates, replays }
}

const dragBoundary = (drag, boundaryIndex, nextTimeMs) => {
  const documentEvents = installDocument()
  drag.startBoundaryDrag(
    boundaryIndex,
    {
      preventDefault() {},
      stopPropagation() {},
      clientX: 0,
      clientY: 0,
    },
    () => nextTimeMs
  )
  documentEvents.emit('pointermove', { clientX: 20, clientY: 0 })
  documentEvents.emit('pointerup', {})
  documentEvents.restore()
}

describe('useEditLyricsV2WordBoundaryDrag line-edge drags', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('moves only the line start when the first separator is still a preview', () => {
    const { drag, updates, replays } = setupDrag()

    dragBoundary(drag, 0, 200)

    expect(updates).toEqual([
      {
        lineIndex: 2,
        preserveWords: true,
        lineStartMs: 200,
      },
    ])
    expect(replays).toEqual([{ lineIndex: 2, startMs: 200 }])
  })

  it('moves only the line end when the last separator is still a preview', () => {
    const { drag, updates } = setupDrag()

    dragBoundary(drag, 2, 1200)

    expect(updates).toEqual([
      {
        lineIndex: 2,
        preserveWords: true,
        lineEndMs: 1200,
      },
    ])
  })

  it('still writes word timings when a middle separator is dragged', () => {
    const { drag, updates } = setupDrag()

    dragBoundary(drag, 1, 300)

    expect(updates).toHaveLength(1)
    expect(updates[0].preserveWords).toBeUndefined()
    expect(updates[0].words[0]).toMatchObject({ text: 'hello ', end_ms: 300 })
    expect(updates[0].words[1]).toMatchObject({ text: 'world', start_ms: 300 })
  })

  it('keeps persisted word timings when the first separator moves', () => {
    const { drag, updates } = setupDrag({ hasPersistedWords: true })

    dragBoundary(drag, 0, 200)

    expect(updates).toHaveLength(1)
    expect(updates[0].preserveWords).toBeUndefined()
    expect(updates[0].lineStartMs).toBe(200)
    expect(updates[0].words[0].start_ms).toBe(200)
  })
})
