// Match Configuration's existing three choices, including plain-flag precedence.
export function downloadModeFromConfig(config) {
  if (config.skip_tracks_with_plain_lyrics) return 'skipPlain'
  if (config.skip_tracks_with_synced_lyrics) return 'skipSynced'
  return 'all'
}

export function downloadFlagsForMode(mode) {
  return {
    skipTracksWithSyncedLyrics: mode === 'skipSynced' || mode === 'skipPlain',
    skipTracksWithPlainLyrics: mode === 'skipPlain',
  }
}
