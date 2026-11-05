import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const freshSettings = async () => {
  const { useSpectrogramSettings } = await import('./spectrogram-settings.js')
  return useSpectrogramSettings()
}

const storedConfig = {
  spectrogram_visible: true,
  spectrogram_theme: 'inferno',
  spectrogram_min_hz: 62.5,
  spectrogram_max_hz: 8000,
  spectrogram_midpoint_db: -35,
}

beforeEach(() => {
  vi.resetModules()
  vi.resetAllMocks()
})

describe('useSpectrogramSettings', () => {
  it('loads persisted values snapped to valid stops and themes', async () => {
    const { settings, load } = await freshSettings()
    load({
      spectrogram_visible: false,
      spectrogram_theme: 'unknown',
      spectrogram_min_hz: 50,
      spectrogram_max_hz: 8000,
      spectrogram_midpoint_db: -42,
    })
    expect(settings.value).toEqual({
      visible: false,
      theme: 'inferno',
      minHz: 62.5,
      maxHz: 8000,
      midpointDb: -40,
    })
  })

  it('repairs a loaded range closer than two octaves', async () => {
    const { settings, load } = await freshSettings()
    load({ ...storedConfig, spectrogram_min_hz: 4000, spectrogram_max_hz: 4000 })
    expect(settings.value.minHz).toBe(4000)
    expect(settings.value.maxHz).toBe(16000)
  })

  it('applies changes before the backend confirms them', async () => {
    const { settings, load, save } = await freshSettings()
    load(storedConfig)
    invoke.mockReturnValue(new Promise(() => {}))

    save({ minHz: 125, maxHz: 4000 })

    expect(settings.value.minHz).toBe(125)
    expect(settings.value.maxHz).toBe(4000)
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_settings', { minHz: 125, maxHz: 4000 })
  })

  it('keeps a change and logs the error when saving fails', async () => {
    const { settings, load, save } = await freshSettings()
    load(storedConfig)
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})
    invoke.mockRejectedValue(new Error('database is locked'))

    await save({ midpointDb: -50 })

    expect(settings.value.midpointDb).toBe(-50)
    expect(consoleError).toHaveBeenCalled()
  })

  it('restores and persists the default settings', async () => {
    const { settings, load, reset } = await freshSettings()
    load({
      ...storedConfig,
      spectrogram_theme: 'green',
      spectrogram_min_hz: 500,
      spectrogram_max_hz: 2000,
      spectrogram_midpoint_db: -55,
    })

    await reset()

    const defaults = { visible: true, theme: 'inferno', minHz: 62.5, maxHz: 8000, midpointDb: -35 }
    expect(settings.value).toEqual(defaults)
    expect(invoke).toHaveBeenCalledWith('set_spectrogram_settings', defaults)
  })
})
