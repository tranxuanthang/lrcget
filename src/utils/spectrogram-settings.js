// Frequency stops are octaves of 1 kHz. The bottom handle and the top handle move
// over separate halves of the axis, so every bottom/top combination is valid and
// the band is never narrower than two octaves.
export const SPECTROGRAM_AXIS_STOPS_HZ = Object.freeze([
  62.5, 125, 250, 500, 1000, 2000, 4000, 8000, 16000,
])
export const SPECTROGRAM_MIN_HZ_STOPS = Object.freeze([62.5, 125, 250, 500])
export const SPECTROGRAM_MAX_HZ_STOPS = Object.freeze([2000, 4000, 8000, 16000])

export const DEFAULT_SPECTROGRAM_MIN_HZ = 62.5
export const DEFAULT_SPECTROGRAM_MAX_HZ = 8000

// Where the middle color of the ramp sits, in dB below the loudest displayed bin.
export const SPECTROGRAM_MIDPOINT_SHALLOWEST_DB = -15
export const SPECTROGRAM_MIDPOINT_DEEPEST_DB = -55
export const SPECTROGRAM_MIDPOINT_STEP_DB = 5
export const DEFAULT_SPECTROGRAM_MIDPOINT_DB = -35

const nearestStop = (stops, hz) =>
  stops.reduce((best, stop) =>
    Math.abs(Math.log(stop / hz)) < Math.abs(Math.log(best / hz)) ? stop : best
  )

export const normalizeSpectrogramMinHz = hz =>
  Number.isFinite(hz) && hz > 0
    ? nearestStop(SPECTROGRAM_MIN_HZ_STOPS, hz)
    : DEFAULT_SPECTROGRAM_MIN_HZ

export const normalizeSpectrogramMaxHz = hz =>
  Number.isFinite(hz) && hz > 0
    ? nearestStop(SPECTROGRAM_MAX_HZ_STOPS, hz)
    : DEFAULT_SPECTROGRAM_MAX_HZ

export const normalizeSpectrogramMidpointDb = db => {
  if (!Number.isFinite(db)) return DEFAULT_SPECTROGRAM_MIDPOINT_DB
  const stepped = Math.round(db / SPECTROGRAM_MIDPOINT_STEP_DB) * SPECTROGRAM_MIDPOINT_STEP_DB
  return Math.min(
    SPECTROGRAM_MIDPOINT_SHALLOWEST_DB,
    Math.max(SPECTROGRAM_MIDPOINT_DEEPEST_DB, stepped)
  )
}

export const formatSpectrogramFrequency = hz =>
  hz < 1000 ? `${Math.round(hz)} Hz` : `${hz / 1000} kHz`

export const formatSpectrogramAxisStop = hz => (hz < 1000 ? `${Math.round(hz)}` : `${hz / 1000}k`)

export const formatSpectrogramFrequencyRange = (minHz, maxHz) =>
  `${formatSpectrogramFrequency(minHz)} – ${formatSpectrogramFrequency(maxHz)}`

export const formatSpectrogramMidpoint = db => `${db < 0 ? '−' : ''}${Math.abs(db)} dB`
