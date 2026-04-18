export function useEditLyricsV2Playback({
  audioSource,
  syncedLines,
  progress,
  playingTrack,
  status,
  playTrack,
  resume,
  seek,
}) {
  // Helper to check if the playing track matches the audio source
  const isPlayingCorrectTrack = () => {
    if (!playingTrack.value || !audioSource.value) {
      return false
    }
    return audioSource.value.type === 'library'
      ? playingTrack.value.id === audioSource.value.id
      : playingTrack.value.file_path === audioSource.value.file_path
  }

  const playLine = async lineIndex => {
    if (!audioSource.value) {
      return
    }

    const lineStartMs = syncedLines.value[lineIndex]?.start_ms
    const seekTo = Number.isFinite(lineStartMs) ? lineStartMs / 1000 : progress.value

    if (!isPlayingCorrectTrack()) {
      await playTrack(audioSource.value)
    } else if (status.value === 'paused') {
      resume()
    }

    seek(seekTo)
  }

  const resumeOrPlay = () => {
    if (status.value === 'paused') {
      resume()
      return
    }

    if (audioSource.value) {
      playTrack(audioSource.value)
    }
  }

  return {
    playLine,
    resumeOrPlay,
  }
}
