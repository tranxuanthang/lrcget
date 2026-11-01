import { describe, expect, it } from 'vitest'
import { buildRowBinMap, findPeakDb, sampleColumn } from './spectrogram-render.js'

const BIN_HZ = 44100 / 1024
const BIN_COUNT = 512

const buildMap = (minHz, maxHz, rows = 96) =>
  buildRowBinMap({ rows, minHz, maxHz, binHz: BIN_HZ, binCount: BIN_COUNT })

const silentFrame = () => new Float32Array(BIN_COUNT).fill(-100)

describe('buildRowBinMap', () => {
  it('pools every bin a row covers when the row spans more than one bin', () => {
    const map = buildMap(62.5, 16000)
    expect(map.weight[0]).toBe(-1)
    expect(map.firstBin[0]).toBe(351)
    expect(map.lastBin[0]).toBe(371)
  })

  it('tiles pooled rows so no bin falls between two rows', () => {
    const map = buildMap(62.5, 16000)
    for (let y = map.rows - 1; y > 0; y--) {
      if (map.weight[y] < 0 && map.weight[y - 1] < 0) {
        expect(map.firstBin[y - 1]).toBe(map.lastBin[y] + 1)
      }
    }
  })

  it('interpolates between neighboring bins when a row is narrower than one bin', () => {
    const map = buildMap(500, 2000)
    const bottom = map.rows - 1
    expect(map.firstBin[bottom]).toBe(11)
    expect(map.lastBin[bottom]).toBe(12)
    expect(map.weight[bottom]).toBeGreaterThan(0)
    expect(map.weight[bottom]).toBeLessThan(1)
  })

  it('keeps every row inside the bin range when the band reaches Nyquist', () => {
    const map = buildMap(62.5, 22050)
    expect(map.highestBin).toBe(BIN_COUNT - 1)
    for (let y = 0; y < map.rows; y++) {
      expect(map.firstBin[y]).toBeGreaterThanOrEqual(map.lowestBin)
      expect(map.lastBin[y]).toBeLessThanOrEqual(map.highestBin)
      expect(map.lastBin[y]).toBeGreaterThanOrEqual(map.firstBin[y])
    }
  })
})

describe('sampleColumn', () => {
  it('shows the loudest bin of a pooled row', () => {
    const map = buildMap(62.5, 16000)
    const frame = silentFrame()
    frame[360] = -10
    const column = sampleColumn(frame, map, new Float32Array(map.rows))
    expect(column[0]).toBe(-10)
  })

  it('blends the two bins around an interpolated row', () => {
    const map = buildMap(500, 2000)
    const bottom = map.rows - 1
    const frame = silentFrame()
    frame[11] = -40
    frame[12] = -20
    const column = sampleColumn(frame, map, new Float32Array(map.rows))
    expect(column[bottom]).toBeCloseTo(-40 + 20 * map.weight[bottom], 5)
  })
})

describe('findPeakDb', () => {
  it('ignores energy outside the displayed band', () => {
    const map = buildMap(500, 2000)
    const loudBass = silentFrame()
    loudBass[1] = 0
    loudBass[20] = -12
    const quiet = silentFrame()
    quiet[30] = -30
    expect(findPeakDb([loudBass, quiet], map.lowestBin, map.highestBin)).toBe(-12)
  })
})
