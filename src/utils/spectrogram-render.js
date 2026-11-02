// The FFT window scales with the sample rate (to the nearest power of two) so every file
// gets roughly the bin width (~43 Hz) and window length (~23 ms) that 1024 samples give at
// 44.1 kHz. A fixed window would blur onsets in low-rate files and smear the lowest
// octaves into one or two bins in high-rate ones.
export const spectrogramFftSize = sampleRate => {
  const scaled = (1024 * sampleRate) / 44100
  if (!Number.isFinite(scaled) || scaled <= 0) return 1024
  return Math.min(8192, Math.max(256, 2 ** Math.round(Math.log2(scaled))))
}

// FFT bins are evenly spaced in Hz but the panel's rows are evenly spaced in log Hz,
// so near the top one row covers many bins and near the bottom one bin covers
// several rows. Rows spanning at least a full bin show the loudest bin they cover,
// so narrow peaks can't fall between rows; narrower rows interpolate between the
// two bins around their center frequency instead of repeating one bin as a block.
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

// Remaps a level between the floor (0) and the peak (1) so the middle of the color
// ramp lands `midpointDb` below the peak. Schlick's bias curve keeps both ends
// fixed and has finite slope at each, so moving the midpoint never clips colors;
// a midpoint halfway down the dynamic range leaves levels unchanged.
export const shapeLevel = (level, midpointDb, dynamicRangeDb) => {
  const midpointLevel = 1 + midpointDb / dynamicRangeDb
  const bias = (2 * midpointLevel - 1) / (1 - midpointLevel)
  return level / (bias * (1 - level) + 1)
}

// Precomputes the color lookup table offset (index * 3) for each quantized level,
// keeping the per-pixel cost of the curve to one array read.
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
