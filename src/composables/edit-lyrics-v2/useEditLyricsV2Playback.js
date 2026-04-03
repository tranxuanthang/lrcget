export function useEditLyricsV2Playback({
  editingTrack,
  syncedLines,
  progress,
  playingTrack,
  status,
  playTrack,
  resume,
  seek
}) {
  const playLine = async (lineIndex) => {
    if (!editingTrack.value) {
      return
    }

    const lineStartMs = syncedLines.value[lineIndex]?.start_ms
    const seekTo = Number.isFinite(lineStartMs) ? lineStartMs / 1000 : progress.value

    if (!playingTrack.value || playingTrack.value.id !== editingTrack.value.id) {
      await playTrack(editingTrack.value)
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

    if (editingTrack.value) {
      playTrack(editingTrack.value)
    }
  }

  return {
    playLine,
    resumeOrPlay
  }
}
