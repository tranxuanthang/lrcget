import { describe, expect, it } from 'vitest'
import {
  DEFAULT_SPECTROGRAM_MAX_HZ,
  DEFAULT_SPECTROGRAM_MIDPOINT_DB,
  DEFAULT_SPECTROGRAM_MIN_HZ,
  SPECTROGRAM_AXIS_STOPS_HZ,
  SPECTROGRAM_MAX_HZ_STOPS,
  SPECTROGRAM_MIN_HZ_STOPS,
  formatSpectrogramAxisStop,
  formatSpectrogramFrequencyRange,
  formatSpectrogramMidpoint,
  normalizeSpectrogramMaxHz,
  normalizeSpectrogramMidpointDb,
  normalizeSpectrogramMinHz,
} from './spectrogram-settings.js'

describe('spectrogram settings', () => {
  it('splits the octave axis into separate halves for the bottom and top handles', () => {
    expect(SPECTROGRAM_AXIS_STOPS_HZ).toEqual([
      ...SPECTROGRAM_MIN_HZ_STOPS,
      1000,
      ...SPECTROGRAM_MAX_HZ_STOPS,
    ])
    expect(Math.max(...SPECTROGRAM_MIN_HZ_STOPS) * 4).toBeLessThanOrEqual(
      Math.min(...SPECTROGRAM_MAX_HZ_STOPS)
    )
    expect(SPECTROGRAM_MIN_HZ_STOPS).toContain(DEFAULT_SPECTROGRAM_MIN_HZ)
    expect(SPECTROGRAM_MAX_HZ_STOPS).toContain(DEFAULT_SPECTROGRAM_MAX_HZ)
  })

  it('snaps stored frequencies to the nearest stop within each handle range', () => {
    expect(normalizeSpectrogramMinHz(50)).toBe(62.5)
    expect(normalizeSpectrogramMinHz(300)).toBe(250)
    expect(normalizeSpectrogramMinHz(1000)).toBe(500)
    expect(normalizeSpectrogramMaxHz(20000)).toBe(16000)
    expect(normalizeSpectrogramMaxHz(1000)).toBe(2000)
    expect(normalizeSpectrogramMaxHz(8000)).toBe(8000)
  })

  it('falls back to the defaults for missing or invalid frequencies', () => {
    for (const value of [undefined, null, NaN, 0, -100, 'abc']) {
      expect(normalizeSpectrogramMinHz(value)).toBe(DEFAULT_SPECTROGRAM_MIN_HZ)
      expect(normalizeSpectrogramMaxHz(value)).toBe(DEFAULT_SPECTROGRAM_MAX_HZ)
    }
  })

  it('rounds the midpoint to 5 dB steps between -55 and -15 dB', () => {
    expect(normalizeSpectrogramMidpointDb(-37)).toBe(-35)
    expect(normalizeSpectrogramMidpointDb(-80)).toBe(-55)
    expect(normalizeSpectrogramMidpointDb(0)).toBe(-15)
    expect(normalizeSpectrogramMidpointDb(undefined)).toBe(DEFAULT_SPECTROGRAM_MIDPOINT_DB)
    expect(normalizeSpectrogramMidpointDb(NaN)).toBe(DEFAULT_SPECTROGRAM_MIDPOINT_DB)
  })

  it('formats axis stops, the range readout and the midpoint readout', () => {
    expect(SPECTROGRAM_AXIS_STOPS_HZ.map(formatSpectrogramAxisStop)).toEqual([
      '63',
      '125',
      '250',
      '500',
      '1k',
      '2k',
      '4k',
      '8k',
      '16k',
    ])
    expect(formatSpectrogramFrequencyRange(125, 8000)).toBe('125 Hz – 8 kHz')
    expect(formatSpectrogramFrequencyRange(62.5, 16000)).toBe('63 Hz – 16 kHz')
    expect(formatSpectrogramMidpoint(-35)).toBe('−35 dB')
  })
})
