// Frequency stops are octaves of 1 kHz.
export const SPECTROGRAM_AXIS_STOPS_HZ = Object.freeze([
  62.5, 125, 250, 500, 1000, 2000, 4000, 8000, 16000,
])

export const DEFAULT_SPECTROGRAM_MIN_HZ = 62.5
export const DEFAULT_SPECTROGRAM_MAX_HZ = 8000

// Where the middle color of the ramp sits, in dB below the loudest displayed bin.
export const SPECTROGRAM_MIDPOINT_SHALLOWEST_DB = -15
export const SPECTROGRAM_MIDPOINT_DEEPEST_DB = -55
export const SPECTROGRAM_MIDPOINT_STEP_DB = 5
export const DEFAULT_SPECTROGRAM_MIDPOINT_DB = -35

// Both handles move over every stop; the top must stay this many stops (octaves) above
// the bottom.
export const SPECTROGRAM_MIN_STOP_GAP = 2

const LAST_STOP_INDEX = SPECTROGRAM_AXIS_STOPS_HZ.length - 1
const DEFAULT_MIN_STOP_INDEX = SPECTROGRAM_AXIS_STOPS_HZ.indexOf(DEFAULT_SPECTROGRAM_MIN_HZ)
const DEFAULT_MAX_STOP_INDEX = SPECTROGRAM_AXIS_STOPS_HZ.indexOf(DEFAULT_SPECTROGRAM_MAX_HZ)

const isValidHz = hz => Number.isFinite(hz) && hz > 0

const nearestStop = (stops, hz) =>
  stops.reduce((best, stop) =>
    Math.abs(Math.log(stop / hz)) < Math.abs(Math.log(best / hz)) ? stop : best
  )

export const spectrogramStopIndex = hz =>
  SPECTROGRAM_AXIS_STOPS_HZ.indexOf(nearestStop(SPECTROGRAM_AXIS_STOPS_HZ, hz))

export const clampSpectrogramMinStop = (index, maxIndex) =>
  Math.max(0, Math.min(index, maxIndex - SPECTROGRAM_MIN_STOP_GAP))

export const clampSpectrogramMaxStop = (index, minIndex) =>
  Math.min(LAST_STOP_INDEX, Math.max(index, minIndex + SPECTROGRAM_MIN_STOP_GAP))

// Snaps both ends to stops. A pair closer than the minimum gap keeps its bottom and raises
// its top, lowering the bottom only when the top runs out of stops.
export const normalizeSpectrogramFrequencyRange = (minHz, maxHz) => {
  const minIndex = isValidHz(minHz) ? spectrogramStopIndex(minHz) : DEFAULT_MIN_STOP_INDEX
  const maxIndex = clampSpectrogramMaxStop(
    isValidHz(maxHz) ? spectrogramStopIndex(maxHz) : DEFAULT_MAX_STOP_INDEX,
    minIndex
  )
  return {
    minHz: SPECTROGRAM_AXIS_STOPS_HZ[clampSpectrogramMinStop(minIndex, maxIndex)],
    maxHz: SPECTROGRAM_AXIS_STOPS_HZ[maxIndex],
  }
}

// A file can't contain frequencies above half its sample rate, so the displayed top is
// capped there; the bottom drops too if needed to keep the minimum gap in octaves.
export const displayedSpectrogramRange = (minHz, maxHz, sampleRate) => {
  const nyquistHz = Number.isFinite(sampleRate) && sampleRate > 0 ? sampleRate / 2 : Infinity
  const displayedMaxHz = Math.min(maxHz, nyquistHz)
  return {
    minHz: Math.min(minHz, displayedMaxHz / 2 ** SPECTROGRAM_MIN_STOP_GAP),
    maxHz: displayedMaxHz,
  }
}

export const normalizeSpectrogramMidpointDb = db => {
  if (!Number.isFinite(db)) return DEFAULT_SPECTROGRAM_MIDPOINT_DB
  const stepped = Math.round(db / SPECTROGRAM_MIDPOINT_STEP_DB) * SPECTROGRAM_MIDPOINT_STEP_DB
  return Math.min(
    SPECTROGRAM_MIDPOINT_SHALLOWEST_DB,
    Math.max(SPECTROGRAM_MIDPOINT_DEEPEST_DB, stepped)
  )
}

export const formatSpectrogramFrequency = hz =>
  hz < 1000 ? `${Math.round(hz)} Hz` : `${Number((hz / 1000).toFixed(1))} kHz`

export const formatSpectrogramAxisStop = hz => (hz < 1000 ? `${Math.round(hz)}` : `${hz / 1000}k`)

export const formatSpectrogramFrequencyRange = (minHz, maxHz) =>
  `${formatSpectrogramFrequency(minHz)} – ${formatSpectrogramFrequency(maxHz)}`

// Appends what this file actually shows when its sample rate caps the chosen top:
// "125 Hz – 16 kHz (11 kHz)", or both ends when the bottom had to drop as well.
export const formatSpectrogramRangeReadout = (range, displayedRange) => {
  const chosen = formatSpectrogramFrequencyRange(range.minHz, range.maxHz)
  if (!displayedRange || displayedRange.maxHz >= range.maxHz) return chosen
  const shown =
    displayedRange.minHz < range.minHz
      ? formatSpectrogramFrequencyRange(displayedRange.minHz, displayedRange.maxHz)
      : formatSpectrogramFrequency(displayedRange.maxHz)
  return `${chosen} (${shown})`
}

export const formatSpectrogramMidpoint = db => `${db < 0 ? '−' : ''}${Math.abs(db)} dB`
