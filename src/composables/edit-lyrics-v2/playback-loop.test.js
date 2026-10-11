import { describe, expect, it, vi } from 'vitest'
import { createSSRApp, effectScope, nextTick, ref } from 'vue'
import { renderToString } from 'vue/server-renderer'
import SyncedLyricsEditor from '@/components/library/edit-lyrics-v2/SyncedLyricsEditor.vue'
import { useEditLyricsV2Playback } from './useEditLyricsV2Playback.js'
vi.mock('@/components/library/edit-lyrics-v2/SyncedWordTimingLane.vue', () => ({
  default: { render: () => null },
}))
const line = () => ({
  text: 'test phrase',
  start_ms: 1000,
  end_ms: 2000,
  words: [{ text: 'test', start_ms: 1000, end_ms: 1400 }],
})
const player = () => {
  const scope = effectScope()
  const controls = {
    audioSource: ref({ type: 'library', id: 1 }),
    syncedLines: ref([line(), { ...line(), start_ms: 4000, end_ms: 5000 }]),
    progress: ref(0),
    duration: ref(10),
    selectedLineIndex: ref(0),
    markerPreview: ref(null),
    onError: vi.fn(),
    playingTrack: ref({ id: 1 }),
    status: ref('paused'),
    playTrack: vi.fn(),
    resume: vi.fn(),
    seek: vi.fn(),
  }
  return { scope, controls, state: scope.run(() => useEditLyricsV2Playback(controls)) }
}

describe('phrase loop', () => {
  it('loops a final lyric to the recording end when its end is inferred', async () => {
    const { state, controls, scope } = player()
    controls.selectedLineIndex.value = 1
    controls.syncedLines.value[1].end_ms = null
    expect(state.loopRange.value).toEqual({ start: 4, end: 10 })
    await state.toggleLoop()
    expect(state.loopEnabled.value).toBe(true)
    expect(controls.seek).toHaveBeenLastCalledWith(4)
    controls.status.value = 'playing'
    controls.progress.value = 10
    await nextTick()
    expect(controls.seek).toHaveBeenLastCalledWith(4)
    scope.stop()
  })
  it('keeps an inferred final loop unavailable until duration is known', () => {
    const { state, controls, scope } = player()
    controls.selectedLineIndex.value = 1
    controls.syncedLines.value[1].end_ms = null
    controls.duration.value = 0
    expect(state.loopRange.value).toBeNull()
    controls.duration.value = 10
    expect(state.loopRange.value).toEqual({ start: 4, end: 10 })
    scope.stop()
  })
  it.each(['paused', 'playing'])(
    'continues from the current point when Play sees %s',
    async status => {
      const { state, controls, scope } = player()
      controls.status.value = status
      controls.progress.value = 7
      await state.resumeOrPlay()
      expect(controls.resume).toHaveBeenCalledOnce()
      expect(controls.playTrack).not.toHaveBeenCalled()
      expect(controls.seek).not.toHaveBeenCalled()
      expect(controls.progress.value).toBe(7)
      scope.stop()
    }
  )
  it('sentence Play retains Loop and repeats the newly selected phrase', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    expect(state.loopEnabled.value).toBe(true)
    controls.seek.mockClear()
    await state.playLine(1)
    expect(state.loopEnabled.value).toBe(true)
    expect(controls.selectedLineIndex.value).toBe(1)
    expect(controls.seek).toHaveBeenCalledExactlyOnceWith(4)
    controls.status.value = 'playing'
    controls.progress.value = 6
    await nextTick()
    expect(controls.seek).toHaveBeenCalledTimes(2)
    expect(controls.seek).toHaveBeenLastCalledWith(4)
    scope.stop()
  })

  it('edit auditions and explicit offset playback both retain looping', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    await state.playLineAtOffset(0, 100)
    expect(state.loopEnabled.value).toBe(true)
    await state.playLineAtOffset(1, 100)
    expect(state.loopEnabled.value).toBe(true)
    expect(controls.seek).toHaveBeenLastCalledWith(4.1)
    scope.stop()
  })

  it.each(['source', 'dispose', 'timing'])(
    'cancels a pending phrase replay when %s changes',
    async change => {
      const { state, controls, scope } = player()
      let finish
      controls.resume.mockImplementation(
        () =>
          new Promise(resolve => {
            finish = resolve
          })
      )
      const pending = state.playLine(0)
      if (change === 'source') {
        controls.audioSource.value = { type: 'library', id: 2 }
        controls.playingTrack.value = { id: 2 }
      } else if (change === 'dispose') scope.stop()
      else controls.syncedLines.value[0] = { ...line(), start_ms: 1200 }
      finish()
      await pending
      expect(controls.seek).not.toHaveBeenCalled()
      scope.stop()
    }
  )

  it('keeps the newest replay request instead of seeking back when an older resume finishes', async () => {
    const { state, controls, scope } = player()
    const finish = []
    controls.resume.mockImplementation(
      () =>
        new Promise(resolve => {
          finish.push(resolve)
        })
    )
    const first = state.playLine(0)
    const second = state.playLine(1)
    finish[1]()
    await second
    finish[0]()
    await first
    expect(controls.seek).toHaveBeenCalledExactlyOnceWith(4)
    scope.stop()
  })

  it('reports failed phrase seeks and play/resume without unhandled rejections', async () => {
    const { state, controls, scope } = player()
    controls.seek.mockRejectedValue(new Error('Seek unavailable'))
    await state.playLine(0)
    expect(controls.onError).toHaveBeenCalledOnce()
    controls.resume.mockRejectedValue(new Error('Resume unavailable'))
    await state.resumeOrPlay()
    expect(controls.onError).toHaveBeenCalledTimes(2)
    scope.stop()
  })

  it('does not seek after a pending resume is canceled or the recording changes', async () => {
    for (const changeSource of [false, true]) {
      const { state, controls, scope } = player()
      let finish
      controls.resume.mockImplementation(
        () =>
          new Promise(resolve => {
            finish = resolve
          })
      )
      const starting = state.toggleLoop()
      if (changeSource) controls.audioSource.value = { type: 'library', id: 2 }
      else await state.toggleLoop()
      finish()
      await starting
      expect(state.loopEnabled.value).toBe(false)
      expect(controls.seek).not.toHaveBeenCalled()
      scope.stop()
    }
  })

  it('reports failed loop seeks once and stops without an unhandled rejection', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    controls.seek.mockRejectedValue(new Error('Audio unavailable'))
    controls.status.value = 'playing'
    controls.progress.value = 2
    await nextTick()
    await Promise.resolve()
    expect(state.loopEnabled.value).toBe(false)
    expect(controls.onError).toHaveBeenCalledOnce()
    controls.progress.value = 3
    await nextTick()
    expect(controls.seek).toHaveBeenCalledTimes(2)
    scope.stop()
  })

  it('stops looping when disposed or another track takes over even while paused', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    controls.playingTrack.value = { id: 2 }
    await nextTick()
    expect(state.loopEnabled.value).toBe(false)
    controls.playingTrack.value = { id: 1 }
    await state.toggleLoop()
    scope.stop()
    expect(state.loopEnabled.value).toBe(false)
  })

  it('plays with context and seeks only once until the player acknowledges the seek', async () => {
    const { state, controls, scope } = player()
    state.loopLeadSeconds.value = 1
    state.loopTailSeconds.value = 0.5
    await state.toggleLoop()
    expect(controls.seek).toHaveBeenLastCalledWith(0)
    controls.status.value = 'playing'
    controls.progress.value = 2.6
    await nextTick()
    expect(controls.seek).toHaveBeenCalledTimes(2)
    controls.progress.value = 2.7
    await nextTick()
    expect(controls.seek).toHaveBeenCalledTimes(2)
    controls.progress.value = 0.2
    await nextTick()
    controls.progress.value = 2.6
    await nextTick()
    expect(controls.seek).toHaveBeenCalledTimes(3)
    controls.selectedLineIndex.value = 1
    await nextTick()
    expect(state.loopEnabled.value).toBe(true)
    expect(state.loopRange.value).toEqual({ start: 3, end: 5.5 })
    expect(controls.seek).toHaveBeenLastCalledWith(3)
    scope.stop()
  })

  it('suspends an untimed or reversed selection without turning Loop off', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    controls.syncedLines.value.push(
      { text: 'untimed' },
      { text: 'reversed', start_ms: 3000, end_ms: 2000 }
    )
    controls.status.value = 'playing'
    controls.seek.mockClear()
    for (const index of [2, 3]) {
      controls.selectedLineIndex.value = index
      await nextTick()
      expect(state.loopEnabled.value).toBe(true)
      expect(state.canLoop.value).toBe(false)
      expect(controls.seek).not.toHaveBeenCalled()
    }
    controls.selectedLineIndex.value = 1
    await nextTick()
    expect(state.loopEnabled.value).toBe(true)
    expect(controls.seek).toHaveBeenLastCalledWith(4)
    await state.toggleLoop()
    expect(state.loopEnabled.value).toBe(false)
    scope.stop()
  })

  it('can turn Loop off while its current selection has no usable range', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    controls.selectedLineIndex.value = -1
    await nextTick()
    expect(state.loopEnabled.value).toBe(true)
    expect(state.canLoop.value).toBe(false)
    await state.toggleLoop()
    expect(state.loopEnabled.value).toBe(false)
    scope.stop()
  })

  it('suspends on an inactive tab and resumes the same armed mode on return', async () => {
    const { scope, controls } = player()
    // A separately scoped instance supplies the parent editor tab gate.
    scope.stop()
    const nextScope = effectScope()
    const available = ref(true)
    const loop = nextScope.run(() =>
      useEditLyricsV2Playback({ ...controls, loopAvailable: available })
    )
    await loop.toggleLoop()
    controls.status.value = 'playing'
    controls.seek.mockClear()
    available.value = false
    controls.progress.value = 8
    await nextTick()
    expect(loop.loopEnabled.value).toBe(true)
    expect(controls.seek).not.toHaveBeenCalled()
    available.value = true
    await nextTick()
    expect(loop.loopEnabled.value).toBe(true)
    expect(controls.seek).toHaveBeenLastCalledWith(1)
    nextScope.stop()
  })

  it('does not finish a delayed loop start on an inactive tab', async () => {
    const { scope, controls } = player()
    scope.stop()
    const nextScope = effectScope()
    const available = ref(true)
    const loop = nextScope.run(() =>
      useEditLyricsV2Playback({ ...controls, loopAvailable: available })
    )
    let finish
    controls.resume.mockImplementation(
      () =>
        new Promise(resolve => {
          finish = resolve
        })
    )
    const pending = loop.toggleLoop()
    available.value = false
    finish()
    await pending
    expect(loop.loopEnabled.value).toBe(true)
    expect(controls.seek).not.toHaveBeenCalled()
    nextScope.stop()
  })

  it('keeps the Loop control usable when repetition is suspended', async () => {
    const html = await renderToString(
      createSSRApp(SyncedLyricsEditor, {
        modelValue: [{ text: 'Untimed lyric' }],
        selectedLineIndex: 0,
        loopEnabled: true,
        canLoop: false,
      })
    )
    const button = html.match(/<button\b[^>]*aria-label="Loop selected phrase"[^>]*>/)?.[0]
    expect(button).toBeTruthy()
    expect(button).not.toContain('disabled')
    expect(button).toContain('aria-pressed="true"')
    expect(html).toContain('Loop is on; select a lyric with valid start/end markers')
  })

  it('cancels stale replay and loop-start seeks when navigation changes during resume', async () => {
    for (const startLoop of [false, true]) {
      const { state, controls, scope } = player()
      if (!startLoop) await state.toggleLoop()
      controls.seek.mockClear()
      let finish
      controls.resume.mockImplementation(
        () =>
          new Promise(resolve => {
            finish = resolve
          })
      )
      const pending = startLoop ? state.toggleLoop() : state.playLine(0)
      controls.selectedLineIndex.value = 1
      finish()
      await pending
      expect(state.loopEnabled.value).toBe(true)
      expect(controls.seek).not.toHaveBeenCalled()
      controls.status.value = 'playing'
      controls.progress.value = 0.5
      await nextTick()
      expect(controls.seek).toHaveBeenLastCalledWith(4)
      scope.stop()
    }
  })

  it('ignores a failed repeat seek from the previous lyric after navigation', async () => {
    const { state, controls, scope } = player()
    await state.toggleLoop()
    let reject
    controls.seek.mockImplementationOnce(
      () =>
        new Promise((_, fail) => {
          reject = fail
        })
    )
    controls.status.value = 'playing'
    controls.progress.value = 3
    await nextTick()
    controls.selectedLineIndex.value = 1
    await nextTick()
    reject(new Error('old seek failed'))
    await Promise.resolve()
    expect(state.loopEnabled.value).toBe(true)
    expect(controls.onError).not.toHaveBeenCalled()
    scope.stop()
  })

  it('rejects missing/reversed boundaries and does not loop another track', async () => {
    const { state, controls, scope } = player()
    controls.syncedLines.value = [{ text: 'untimed' }]
    expect(state.canLoop.value).toBe(false)
    await state.toggleLoop()
    expect(controls.seek).not.toHaveBeenCalled()
    controls.syncedLines.value = [line()]
    await state.toggleLoop()
    controls.playingTrack.value = { id: 2 }
    controls.status.value = 'playing'
    await nextTick()
    expect(state.loopEnabled.value).toBe(false)
    scope.stop()
  })

  it('loops exactly over a pending marker pair, restores committed bounds on Cancel, and ignores stale previews', async () => {
    const { state, controls, scope } = player()
    expect(state.loopLeadSeconds.value).toBe(0)
    expect(state.loopTailSeconds.value).toBe(0)
    controls.markerPreview.value = {
      line: controls.syncedLines.value[0],
      lineIndex: 0,
      startMs: 1200,
      endMs: 1800,
    }
    await state.toggleLoop()
    expect(controls.seek).toHaveBeenLastCalledWith(1.2)
    controls.status.value = 'playing'
    controls.progress.value = 1.5
    await nextTick()
    controls.progress.value = 1.8
    await nextTick()
    expect(controls.seek).toHaveBeenLastCalledWith(1.2)
    controls.markerPreview.value = null
    controls.progress.value = 2
    await nextTick()
    expect(controls.seek).toHaveBeenLastCalledWith(1)
    controls.markerPreview.value = { line: {}, lineIndex: 0, startMs: 8000, endMs: 9000 }
    controls.progress.value = 1.1
    await nextTick()
    controls.progress.value = 2
    await nextTick()
    expect(controls.seek).toHaveBeenLastCalledWith(1)
    expect(controls.syncedLines.value[0]).toEqual(line())
    scope.stop()
  })
})
