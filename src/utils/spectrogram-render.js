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

export const findPeakDb = (frames, lowestBin, highestBin) => {
  let peakDb = -Infinity
  for (const frame of frames) {
    for (let bin = lowestBin; bin <= highestBin; bin++) {
      if (frame[bin] > peakDb) peakDb = frame[bin]
    }
  }
  return peakDb
}
