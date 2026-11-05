import { describe, it, expect } from 'vitest'
import { resolveWordSplitTimeMs } from './word-split.js'

describe('resolveWordSplitTimeMs', () => {
  it('uses the highlighted lane time when it is provided', () => {
    expect(
      resolveWordSplitTimeMs({
        wordStartMs: 1000,
        wordEndMs: 2000,
        highlightTimeMs: 1600,
        ratioTimeMs: 1999,
      })
    ).toBe(1600)
  })

  it('falls back to the width-ratio time when no highlight time is available', () => {
    expect(
      resolveWordSplitTimeMs({
        wordStartMs: 1000,
        wordEndMs: 2000,
        highlightTimeMs: null,
        ratioTimeMs: 1400,
      })
    ).toBe(1400)
  })

  it('clamps to the strict interior so neither segment is zero-length', () => {
    // Below the lower bound is pulled up to wordStartMs + 1
    expect(
      resolveWordSplitTimeMs({ wordStartMs: 1000, wordEndMs: 2000, highlightTimeMs: 500 })
    ).toBe(1001)
    // Above the upper bound is pulled down to wordEndMs - 1
    expect(
      resolveWordSplitTimeMs({ wordStartMs: 1000, wordEndMs: 2000, highlightTimeMs: 5000 })
    ).toBe(1999)
    // Exactly at the end bound is pulled inside
    expect(
      resolveWordSplitTimeMs({ wordStartMs: 1000, wordEndMs: 2000, highlightTimeMs: 2000 })
    ).toBe(1999)
  })

  it('uses the midpoint when neither highlight nor ratio time is available', () => {
    expect(resolveWordSplitTimeMs({ wordStartMs: 1000, wordEndMs: 2000 })).toBe(1500)
  })
})
