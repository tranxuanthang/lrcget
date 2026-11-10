import { describe, expect, it } from 'vitest'
import {
  DEFAULT_SPECTROGRAM_MAX_HZ,
  DEFAULT_SPECTROGRAM_MIDPOINT_DB,
  DEFAULT_SPECTROGRAM_MIN_HZ,
  SPECTROGRAM_AXIS_STOPS_HZ,
  SPECTROGRAM_MIN_STOP_GAP,
  clampSpectrogramMaxStop,
  clampSpectrogramMinStop,
  displayedSpectrogramRange,
  formatSpectrogramAxisStop,
  formatSpectrogramFrequency,
  formatSpectrogramFrequencyRange,
  formatSpectrogramMidpoint,
  formatSpectrogramRangeReadout,
  normalizeSpectrogramFrequencyRange,
  normalizeSpectrogramMidpointDb,
  spectrogramStopIndex,
} from './spectrogram-settings.js'

describe('spectrogram frequency range', () => {
  it('starts with a default range at least the minimum gap apart', () => {
    const gap =
      spectrogramStopIndex(DEFAULT_SPECTROGRAM_MAX_HZ) -
      spectrogramStopIndex(DEFAULT_SPECTROGRAM_MIN_HZ)
    expect(gap).toBeGreaterThanOrEqual(SPECTROGRAM_MIN_STOP_GAP)
  })

  it('clamps each handle so the top stays two stops above the bottom', () => {
    expect(clampSpectrogramMinStop(7, 8)).toBe(6)
    expect(clampSpectrogramMinStop(-1, 8)).toBe(0)
    expect(clampSpectrogramMinStop(3, 8)).toBe(3)
    expect(clampSpectrogramMaxStop(3, 2)).toBe(4)
    expect(clampSpectrogramMaxStop(10, 0)).toBe(8)
    expect(clampSpectrogramMaxStop(6, 2)).toBe(6)
  })

  it('snaps a stored range to stops and repairs pairs that are too close', () => {
    expect(normalizeSpectrogramFrequencyRange(50, 8000)).toEqual({ minHz: 62.5, maxHz: 8000 })
    expect(normalizeSpectrogramFrequencyRange(2000, 8000)).toEqual({ minHz: 2000, maxHz: 8000 })
    expect(normalizeSpectrogramFrequencyRange(4000, 4000)).toEqual({ minHz: 4000, maxHz: 16000 })
    expect(normalizeSpectrogramFrequencyRange(16000, 16000)).toEqual({ minHz: 4000, maxHz: 16000 })
    expect(normalizeSpectrogramFrequencyRange(2000, 1000)).toEqual({ minHz: 2000, maxHz: 8000 })
    expect(normalizeSpectrogramFrequencyRange(NaN, undefined)).toEqual({
      minHz: DEFAULT_SPECTROGRAM_MIN_HZ,
      maxHz: DEFAULT_SPECTROGRAM_MAX_HZ,
    })
  })

  it("caps the displayed top at the file's Nyquist frequency and keeps two octaves", () => {
    expect(displayedSpectrogramRange(62.5, 8000, 44100)).toEqual({ minHz: 62.5, maxHz: 8000 })
    expect(displayedSpectrogramRange(125, 16000, 22050)).toEqual({ minHz: 125, maxHz: 11025 })
    expect(displayedSpectrogramRange(4000, 16000, 22050)).toEqual({
      minHz: 2756.25,
      maxHz: 11025,
    })
    expect(displayedSpectrogramRange(62.5, 16000, 16000)).toEqual({ minHz: 62.5, maxHz: 8000 })
    expect(displayedSpectrogramRange(62.5, 16000, NaN)).toEqual({ minHz: 62.5, maxHz: 16000 })
  })

  it('shows what the file displays in parentheses only when its sample rate caps the top', () => {
    const range = { minHz: 125, maxHz: 16000 }
    expect(formatSpectrogramRangeReadout(range, displayedSpectrogramRange(125, 16000, 22050))).toBe(
      '125 Hz – 16 kHz (11 kHz)'
    )
    expect(
      formatSpectrogramRangeReadout(
        { minHz: 4000, maxHz: 16000 },
        displayedSpectrogramRange(4000, 16000, 22050)
      )
    ).toBe('4 kHz – 16 kHz (2.8 kHz – 11 kHz)')
    expect(formatSpectrogramRangeReadout(range, displayedSpectrogramRange(125, 16000, 48000))).toBe(
      '125 Hz – 16 kHz'
    )
    expect(formatSpectrogramRangeReadout(range, null)).toBe('125 Hz – 16 kHz')
  })

  it('rounds kilohertz to one decimal place', () => {
    expect(formatSpectrogramFrequency(11025)).toBe('11 kHz')
    expect(formatSpectrogramFrequency(5512.5)).toBe('5.5 kHz')
    expect(formatSpectrogramFrequency(16000)).toBe('16 kHz')
  })
})

describe('spectrogram settings', () => {
  it('spaces the axis stops one octave apart', () => {
    for (let i = 1; i < SPECTROGRAM_AXIS_STOPS_HZ.length; i++) {
      expect(SPECTROGRAM_AXIS_STOPS_HZ[i]).toBe(SPECTROGRAM_AXIS_STOPS_HZ[i - 1] * 2)
    }
  })

  it('falls back to the default range for missing or invalid frequencies', () => {
    for (const value of [undefined, null, NaN, 0, -100, 'abc']) {
      expect(normalizeSpectrogramFrequencyRange(value, value)).toEqual({
        minHz: DEFAULT_SPECTROGRAM_MIN_HZ,
        maxHz: DEFAULT_SPECTROGRAM_MAX_HZ,
      })
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
