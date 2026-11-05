import { describe, expect, it } from 'vitest'
import {
  buildLevelToLutOffset,
  buildRowBinMap,
  computeSpectrogram,
  findPeakDb,
  sampleColumn,
  shapeLevel,
  spectrogramFftSize,
} from './spectrogram-render.js'

describe('spectrogramFftSize', () => {
  it('scales the window with the sample rate to the nearest power of two', () => {
    expect(spectrogramFftSize(44100)).toBe(1024)
    expect(spectrogramFftSize(48000)).toBe(1024)
    expect(spectrogramFftSize(22050)).toBe(512)
    expect(spectrogramFftSize(16000)).toBe(512)
    expect(spectrogramFftSize(96000)).toBe(2048)
    expect(spectrogramFftSize(192000)).toBe(4096)
  })

  it('keeps the bin width close to 43 Hz for common sample rates', () => {
    for (const sampleRate of [22050, 44100, 48000, 88200, 96000, 176400, 192000]) {
      const binHz = sampleRate / spectrogramFftSize(sampleRate)
      expect(binHz).toBeGreaterThan(40)
      expect(binHz).toBeLessThan(50)
    }
  })

  it('stays within bounds and falls back to 1024 for unusable sample rates', () => {
    expect(spectrogramFftSize(1000)).toBe(256)
    expect(spectrogramFftSize(10_000_000)).toBe(8192)
    for (const value of [0, -44100, NaN, undefined]) {
      expect(spectrogramFftSize(value)).toBe(1024)
    }
  })
})

describe('computeSpectrogram', () => {
  const SAMPLE_RATE = 44100
  const FFT_SIZE = spectrogramFftSize(SAMPLE_RATE)

  it('peaks at the bin a pure sine is centered on, in every frame', () => {
    const bin = 23
    const hz = (bin * SAMPLE_RATE) / FFT_SIZE
    const samples = Float32Array.from({ length: FFT_SIZE * 4 }, (_, i) =>
      Math.sin((2 * Math.PI * hz * i) / SAMPLE_RATE)
    )
    const { frames } = computeSpectrogram(samples, SAMPLE_RATE)
    expect(frames.length).toBeGreaterThan(1)
    for (const frame of frames) {
      expect(frame.indexOf(Math.max(...frame))).toBe(bin)
    }
  })

  it('keeps silence at the -180 dB floor in every bin of every frame', () => {
    const { frames, binCount } = computeSpectrogram(new Float32Array(FFT_SIZE * 4), SAMPLE_RATE)
    const floor = new Float32Array(binCount).fill(-180)
    expect(frames.length).toBeGreaterThan(1)
    for (const frame of frames) {
      expect(frame).toEqual(floor)
    }
  })
})

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

describe('shapeLevel', () => {
  const RANGE_DB = 70

  it('keeps the floor black and the peak at the top color for every midpoint', () => {
    for (const midpointDb of [-55, -35, -15]) {
      expect(shapeLevel(0, midpointDb, RANGE_DB)).toBe(0)
      expect(shapeLevel(1, midpointDb, RANGE_DB)).toBeCloseTo(1, 10)
    }
  })

  it('puts the middle color exactly midpointDb below the peak', () => {
    for (const midpointDb of [-55, -45, -25, -15]) {
      const midpointLevel = 1 + midpointDb / RANGE_DB
      expect(shapeLevel(midpointLevel, midpointDb, RANGE_DB)).toBeCloseTo(0.5, 10)
    }
  })

  it('leaves levels unchanged when the midpoint is halfway down the range', () => {
    for (const level of [0.1, 0.37, 0.8]) {
      expect(shapeLevel(level, -35, RANGE_DB)).toBeCloseTo(level, 10)
    }
  })

  it('brightens quiet levels for deeper midpoints and darkens them for shallower ones', () => {
    expect(shapeLevel(0.3, -55, RANGE_DB)).toBeGreaterThan(0.3)
    expect(shapeLevel(0.3, -15, RANGE_DB)).toBeLessThan(0.3)
  })
})

describe('buildLevelToLutOffset', () => {
  it('maps quantized levels onto RGB offsets across the whole lookup table, in order', () => {
    const offsets = buildLevelToLutOffset({
      midpointDb: -45,
      dynamicRangeDb: 70,
      lutSize: 256,
      resolution: 1024,
    })
    expect(offsets).toHaveLength(1024)
    expect(offsets[0]).toBe(0)
    expect(offsets[1023]).toBe(255 * 3)
    for (let i = 1; i < offsets.length; i++) {
      expect(offsets[i]).toBeGreaterThanOrEqual(offsets[i - 1])
      expect(offsets[i] % 3).toBe(0)
    }
  })

  it('stays inside the lookup table even for a midpoint outside the dynamic range', () => {
    const offsets = buildLevelToLutOffset({
      midpointDb: -100,
      dynamicRangeDb: 70,
      lutSize: 256,
      resolution: 1024,
    })
    for (const offset of offsets) {
      expect(offset).toBeGreaterThanOrEqual(0)
      expect(offset).toBeLessThanOrEqual(255 * 3)
    }
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
