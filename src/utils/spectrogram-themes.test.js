import { describe, expect, it } from 'vitest'
import { rgb } from 'd3-color'
import { interpolateInferno } from 'd3-scale-chromatic'
import {
  DEFAULT_SPECTROGRAM_THEME_ID,
  SPECTROGRAM_LUT_SIZE,
  SPECTROGRAM_THEMES,
  getSpectrogramLut,
  getSpectrogramTheme,
  isSpectrogramThemeId,
  normalizeSpectrogramThemeId,
} from './spectrogram-themes.js'

const lastIndex = SPECTROGRAM_LUT_SIZE - 1

const channelsAt = (lut, index) => [lut[index * 3], lut[index * 3 + 1], lut[index * 3 + 2]]

describe('spectrogram themes', () => {
  it('exposes inferno, blue and green in picker order, defaulting to inferno', () => {
    expect(SPECTROGRAM_THEMES.map(theme => theme.id)).toEqual(['inferno', 'blue', 'green'])
    expect(DEFAULT_SPECTROGRAM_THEME_ID).toBe('inferno')
    expect(isSpectrogramThemeId(DEFAULT_SPECTROGRAM_THEME_ID)).toBe(true)
  })

  it('builds a complete lookup table for every theme through the shared builder', () => {
    for (const theme of SPECTROGRAM_THEMES) {
      expect(theme.lut).toBeInstanceOf(Uint8Array)
      expect(theme.lut).toHaveLength(SPECTROGRAM_LUT_SIZE * 3)
    }
  })

  it('reproduces d3 inferno byte-for-byte after unifying onto rgb() parsing', () => {
    const lut = getSpectrogramLut('inferno')
    for (let i = 0; i < SPECTROGRAM_LUT_SIZE; i++) {
      const expected = rgb(interpolateInferno(i / lastIndex))
      expect(channelsAt(lut, i)).toEqual([expected.r, expected.g, expected.b])
    }
  })

  it('declares a hex swatch colour for every theme', () => {
    for (const theme of SPECTROGRAM_THEMES) {
      expect(theme.swatchColor).toMatch(/^#[0-9a-f]{6}$/)
    }
  })

  it('anchors every theme at a near-black floor rising to a brighter peak', () => {
    const sum = channels => channels.reduce((total, channel) => total + channel, 0)
    for (const theme of SPECTROGRAM_THEMES) {
      const floor = sum(channelsAt(theme.lut, 0))
      const peak = sum(channelsAt(theme.lut, lastIndex))
      expect(floor).toBeLessThan(16)
      expect(peak).toBeGreaterThan(floor)
    }
  })

  it('keeps swatches distinct so unlabeled options stay tellable apart', () => {
    const swatches = SPECTROGRAM_THEMES.map(theme => theme.swatchColor)
    expect(new Set(swatches).size).toBe(swatches.length)
  })

  it('falls back to the default for unknown, empty or missing ids', () => {
    for (const value of ['nope', '', null, undefined, 0, {}]) {
      expect(normalizeSpectrogramThemeId(value)).toBe(DEFAULT_SPECTROGRAM_THEME_ID)
      expect(getSpectrogramTheme(value).id).toBe(DEFAULT_SPECTROGRAM_THEME_ID)
    }
  })

  it('preserves known ids', () => {
    for (const theme of SPECTROGRAM_THEMES) {
      expect(normalizeSpectrogramThemeId(theme.id)).toBe(theme.id)
      expect(getSpectrogramTheme(theme.id)).toBe(theme)
    }
  })
})
