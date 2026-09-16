import { describe, expect, it } from 'vitest'
import { buildPlaybackSource } from './useEditLyricsV2VocalStem.js'

const librarySource = {
  type: 'library',
  id: 42,
  file_path: '/music/song.flac',
  title: 'Song',
  album_name: 'Album',
  artist_name: 'Artist',
  album_artist_name: 'Artist',
  duration: 180,
}

describe('buildPlaybackSource', () => {
  it('keeps the original source in song mode', () => {
    expect(buildPlaybackSource(librarySource, 'song', '/cache/vocals.wav')).toBe(librarySource)
  })

  it('plays vocals.wav as a file source in vocals mode', () => {
    expect(buildPlaybackSource(librarySource, 'vocals', '/cache/vocals.wav')).toEqual({
      type: 'file',
      file_path: '/cache/vocals.wav',
      title: 'Song',
      album_name: 'Album',
      artist_name: 'Artist',
      album_artist_name: 'Artist',
      duration: 180,
    })
  })

  it('falls back to the original source when no stem exists', () => {
    expect(buildPlaybackSource(librarySource, 'vocals', null)).toBe(librarySource)
  })
})
