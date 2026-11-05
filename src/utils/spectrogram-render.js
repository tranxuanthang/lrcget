import FFT from 'fft.js'

const HOPS_PER_WINDOW = 8

// Keeps the bin width and window length of a 1024-point FFT at 44.1 kHz for any sample rate.
export const spectrogramFftSize = sampleRate => {
  const scaled = (1024 * sampleRate) / 44100
  if (!Number.isFinite(scaled) || scaled <= 0) return 1024
  return Math.min(8192, Math.max(256, 2 ** Math.round(Math.log2(scaled))))
}

// Short-time FFT (Hann window, hop of 1/8 window) as dB magnitudes per frame.
export const computeSpectrogram = (samples, sampleRate) => {
  const fftSize = spectrogramFftSize(sampleRate)
  const hop = fftSize / HOPS_PER_WINDOW
  const win = new Float32Array(fftSize)
  for (let i = 0; i < fftSize; i++) {
    win[i] = 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (fftSize - 1))
  }

  const totalFrames = Math.max(1, Math.floor((samples.length - fftSize) / hop) + 1)
  const binCount = fftSize / 2

  // realTransform() fills only the non-redundant bins 0..binCount-1, as interleaved [re, im].
  const fft = new FFT(fftSize)
  const input = new Float32Array(fftSize)
  const spectrum = fft.createComplexArray()

  const frames = new Array(totalFrames)

  for (let frame = 0; frame < totalFrames; frame++) {
    const offset = frame * hop
    for (let i = 0; i < fftSize; i++) {
      input[i] = (samples[offset + i] || 0) * win[i]
    }
    fft.realTransform(spectrum, input)
    const mags = new Float32Array(binCount)
    for (let b = 0; b < binCount; b++) {
      const re = spectrum[2 * b]
      const im = spectrum[2 * b + 1]
      const mag = Math.sqrt(re * re + im * im)
      mags[b] = mag > 1e-9 ? 20 * Math.log10(mag) : -180
    }
    frames[frame] = mags
  }
  return { frames, binCount, fftSize, sampleRate }
}

// Maps log-spaced rows onto FFT bins: rows spanning at least a bin take their loudest bin so no
// peak falls between rows, and narrower rows interpolate between the two bins around them.
export const buildRowBinMap = ({ rows, minHz, maxHz, binHz, binCount }) => {
  const logMinHz = Math.log(minHz)
  const logSpan = Math.log(maxHz) - logMinHz
  const binAt = position => Math.exp(logMinHz + position * logSpan) / binHz
  const clampBin = bin => Math.max(0, Math.min(binCount - 1, bin))

  const firstBin = new Int32Array(rows)
  const lastBin = new Int32Array(rows)
  // Interpolation weight toward lastBin, or -1 for rows that take the loudest bin.
  const weight = new Float32Array(rows)

  for (let y = 0; y < rows; y++) {
    const bottom = binAt((rows - 1 - y) / rows)
    const top = binAt((rows - y) / rows)

    if (top - bottom >= 1) {
      firstBin[y] = clampBin(Math.ceil(bottom))
      lastBin[y] = clampBin(Math.ceil(top) - 1)
      weight[y] = -1
      continue
    }

    const center = binAt((rows - 0.5 - y) / rows)
    const lower = Math.floor(center)
    firstBin[y] = clampBin(lower)
    lastBin[y] = clampBin(lower + 1)
    weight[y] = firstBin[y] === lastBin[y] ? 0 : center - lower
  }

  return {
    rows,
    firstBin,
    lastBin,
    weight,
    lowestBin: rows > 0 ? firstBin[rows - 1] : 0,
    highestBin: rows > 0 ? lastBin[0] : 0,
  }
}

// Fills `out` (one value per row, top row first) with the dB level each row shows.
export const sampleColumn = (frame, rowMap, out) => {
  const { rows, firstBin, lastBin, weight } = rowMap

  for (let y = 0; y < rows; y++) {
    const first = firstBin[y]
    const last = lastBin[y]

    if (weight[y] < 0) {
      let loudest = frame[first]
      for (let bin = first + 1; bin <= last; bin++) {
        if (frame[bin] > loudest) loudest = frame[bin]
      }
      out[y] = loudest
    } else {
      out[y] = frame[first] + (frame[last] - frame[first]) * weight[y]
    }
  }

  return out
}

// Schlick's bias curve keeps 0 and 1 fixed and puts the middle color `midpointDb` below the peak.
export const shapeLevel = (level, midpointDb, dynamicRangeDb) => {
  const midpointLevel = 1 + midpointDb / dynamicRangeDb
  const bias = (2 * midpointLevel - 1) / (1 - midpointLevel)
  return level / (bias * (1 - level) + 1)
}

// Precomputes each quantized level's LUT offset so the curve costs one array read per pixel.
export const buildLevelToLutOffset = ({ midpointDb, dynamicRangeDb, lutSize, resolution }) => {
  const offsets = new Uint16Array(resolution)
  for (let i = 0; i < resolution; i++) {
    const shaped = shapeLevel(i / (resolution - 1), midpointDb, dynamicRangeDb)
    offsets[i] = Math.round(Math.min(1, Math.max(0, shaped)) * (lutSize - 1)) * 3
  }
  return offsets
}

export const findPeakDb = (frames, lowestBin, highestBin) => {
  let peakDb = -Infinity
  for (const frame of frames) {
    for (let bin = lowestBin; bin <= highestBin; bin++) {
      if (frame[bin] > peakDb) peakDb = frame[bin]
    }
  }
  return peakDb
}
