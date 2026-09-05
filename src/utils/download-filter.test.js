import { describe, expect, it } from 'vitest'
import { downloadModeFromConfig, downloadFlagsForMode } from './download-filter.js'

describe('Download lyrics for', () => {
  it.each([
    ['all', false, false],
    ['skipSynced', true, false],
    ['skipPlain', true, true],
  ])('round-trips %s through the existing Configuration flags', (mode, synced, plain) => {
    expect(downloadFlagsForMode(mode)).toEqual({
      skipTracksWithSyncedLyrics: synced,
      skipTracksWithPlainLyrics: plain,
    })
    expect(
      downloadModeFromConfig({
        skip_tracks_with_synced_lyrics: synced,
        skip_tracks_with_plain_lyrics: plain,
      })
    ).toBe(mode)
  })

  it('uses the plain flag first for an unusual persisted combination, like Configuration', () => {
    const mode = downloadModeFromConfig({
      skip_tracks_with_synced_lyrics: false,
      skip_tracks_with_plain_lyrics: true,
    })
    expect(mode).toBe('skipPlain')
    expect(downloadFlagsForMode(mode)).toEqual({
      skipTracksWithSyncedLyrics: true,
      skipTracksWithPlainLyrics: true,
    })
  })
})
