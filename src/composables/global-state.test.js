import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const freshGlobalState = async () => {
  const { useGlobalState } = await import('./global-state.js')
  return useGlobalState()
}

const deferred = () => {
  let resolve
  let reject
  const promise = new Promise((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

beforeEach(() => {
  vi.resetModules()
  vi.resetAllMocks()
})

describe('spectrogram display settings', () => {
  it('loads persisted values snapped to valid stops', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 50, maxHz: 8000, midpointDb: -42 })
    expect(state.spectrogramMinHz.value).toBe(62.5)
    expect(state.spectrogramMaxHz.value).toBe(8000)
    expect(state.spectrogramMidpointDb.value).toBe(-40)
  })

  it('lets the bottom use stops above 1 kHz and repairs ranges closer than two octaves', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 2000, maxHz: 8000, midpointDb: -35 })
    expect(state.spectrogramMinHz.value).toBe(2000)
    expect(state.spectrogramMaxHz.value).toBe(8000)

    state.loadSpectrogramDisplay({ minHz: 4000, maxHz: 4000, midpointDb: -35 })
    expect(state.spectrogramMinHz.value).toBe(4000)
    expect(state.spectrogramMaxHz.value).toBe(16000)

    state.setSpectrogramFrequencyRange(500, 1000)
    expect(state.spectrogramMinHz.value).toBe(500)
    expect(state.spectrogramMaxHz.value).toBe(2000)
  })

  it('applies changes immediately and persists them only when saved', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 62.5, maxHz: 8000, midpointDb: -35 })

    state.setSpectrogramFrequencyRange(125, 4000)
    state.setSpectrogramMidpointDb(-45)
    expect(state.spectrogramMinHz.value).toBe(125)
    expect(state.spectrogramMidpointDb.value).toBe(-45)
    expect(invoke).not.toHaveBeenCalled()

    await state.saveSpectrogramFrequencyRange()
    await state.saveSpectrogramMidpointDb()
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_frequency_range', {
      minHz: 125,
      maxHz: 4000,
    })
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_midpoint', { midpointDb: -45 })
  })

  it('skips saving values that are already persisted', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 250, maxHz: 16000, midpointDb: -25 })
    await state.saveSpectrogramFrequencyRange()
    await state.saveSpectrogramMidpointDb()
    expect(invoke).not.toHaveBeenCalled()
  })

  it('restores the last saved values when persisting fails', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 62.5, maxHz: 8000, midpointDb: -35 })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    invoke.mockRejectedValue(new Error('database is locked'))

    state.setSpectrogramFrequencyRange(500, 2000)
    state.setSpectrogramMidpointDb(-15)
    await state.saveSpectrogramFrequencyRange()
    await state.saveSpectrogramMidpointDb()

    expect(state.spectrogramMinHz.value).toBe(62.5)
    expect(state.spectrogramMaxHz.value).toBe(8000)
    expect(state.spectrogramMidpointDb.value).toBe(-35)
  })

  it('saves a return to the persisted value while an earlier save is still pending', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 62.5, maxHz: 8000, midpointDb: -35 })
    const pending = deferred()
    invoke.mockReturnValueOnce(pending.promise).mockResolvedValueOnce(undefined)

    state.setSpectrogramFrequencyRange(125, 8000)
    const firstSave = state.saveSpectrogramFrequencyRange()
    state.setSpectrogramFrequencyRange(62.5, 8000)
    const secondSave = state.saveSpectrogramFrequencyRange()
    pending.resolve()
    await Promise.all([firstSave, secondSave])

    expect(invoke).toHaveBeenNthCalledWith(2, 'set_spectrogram_frequency_range', {
      minHz: 62.5,
      maxHz: 8000,
    })
    expect(state.spectrogramMinHz.value).toBe(62.5)
  })

  it('keeps a newer pending value when an older save fails', async () => {
    const state = await freshGlobalState()
    state.loadSpectrogramDisplay({ minHz: 62.5, maxHz: 8000, midpointDb: -35 })
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const olderSave = deferred()
    const newerSave = deferred()
    invoke.mockReturnValueOnce(olderSave.promise).mockReturnValueOnce(newerSave.promise)

    state.setSpectrogramMidpointDb(-40)
    const firstSave = state.saveSpectrogramMidpointDb()
    state.setSpectrogramMidpointDb(-50)
    const secondSave = state.saveSpectrogramMidpointDb()

    olderSave.reject(new Error('database is locked'))
    await firstSave
    expect(state.spectrogramMidpointDb.value).toBe(-50)

    newerSave.resolve()
    await secondSave
    expect(state.spectrogramMidpointDb.value).toBe(-50)
  })

  it('resets the theme, range and midpoint to their defaults and persists them', async () => {
    const state = await freshGlobalState()
    state.setSpectrogramTheme('green')
    state.loadSpectrogramDisplay({ minHz: 500, maxHz: 2000, midpointDb: -55 })

    await state.resetSpectrogramSettings()

    expect(state.spectrogramTheme.value).toBe('inferno')
    expect(state.spectrogramMinHz.value).toBe(62.5)
    expect(state.spectrogramMaxHz.value).toBe(8000)
    expect(state.spectrogramMidpointDb.value).toBe(-35)
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_theme', { theme: 'inferno' })
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_frequency_range', {
      minHz: 62.5,
      maxHz: 8000,
    })
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_midpoint', { midpointDb: -35 })
  })
})
