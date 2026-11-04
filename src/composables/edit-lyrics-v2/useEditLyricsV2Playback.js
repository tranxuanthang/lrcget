import { computed, onScopeDispose, ref, watch } from 'vue'
const playbackSourceKey = source => {
  if (source?.type === 'library' && source.id != null) return `library:${source.id}`
  return source?.file_path ? `file:${source.file_path}` : ''
}

export function useEditLyricsV2Playback({
  audioSource,
  syncedLines,
  progress,
  playingTrack,
  status,
  playTrack,
  resume,
  seek,
  duration,
  selectedLineIndex,
  markerPreview,
  loopAvailable = ref(true),
  onError = () => {},
}) {
  const loopEnabled = ref(false)
  const loopLeadSeconds = ref(0)
  const loopTailSeconds = ref(0)
  let awaitingLoopSeek = false
  let initializingLoop = false
  let initializingPlay = false
  let loopOperation = 0
  let playOperation = 0
  let disposed = false
  const stopLoop = () => {
    loopOperation++
    loopEnabled.value = false
    awaitingLoopSeek = false
    initializingLoop = false
  }
  const failLoop = (error, operation) => {
    if (operation !== loopOperation) return
    stopLoop()
    onError(error)
  }
  const boundedContext = value =>
    Number.isFinite(Number(value)) ? Math.min(5, Math.max(0, Number(value))) : 0
  const loopRange = computed(() => {
    const index = selectedLineIndex?.value
    const line = syncedLines.value[index]
    if (!Number.isFinite(line?.start_ms)) return null
    const preview = markerPreview?.value
    const usePreview = preview?.line === line && preview?.lineIndex === index
    const startMs = usePreview ? preview.startMs : line.start_ms
    const endMs = usePreview
      ? preview.endMs
      : Number.isFinite(line.end_ms)
        ? line.end_ms
        : (syncedLines.value[index + 1]?.start_ms ??
          (Number.isFinite(duration?.value) && duration.value > 0 ? duration.value * 1000 : null))
    if (!Number.isFinite(startMs) || startMs < 0 || !Number.isFinite(endMs) || endMs <= startMs)
      return null
    const start = Math.max(0, startMs / 1000 - boundedContext(loopLeadSeconds.value))
    const end = Math.min(
      duration?.value || Infinity,
      endMs / 1000 + boundedContext(loopTailSeconds.value)
    )
    return end > start ? { start, end } : null
  })
  // Helper to check if the playing track matches the audio source
  const isPlayingCorrectTrack = () => {
    if (!playingTrack.value || !audioSource.value) {
      return false
    }
    return audioSource.value.type === 'library'
      ? playingTrack.value.id === audioSource.value.id
      : playingTrack.value.file_path === audioSource.value.file_path
  }

  const playLineAtOffset = async (lineIndex, offsetMs = 0) => {
    const source = audioSource.value
    const line = syncedLines.value[lineIndex]
    if (!source || !line || !Number.isFinite(offsetMs) || disposed) return
    // Loop is a mode: navigation moves its range without changing the toggle.
    if (loopEnabled.value && selectedLineIndex) selectedLineIndex.value = lineIndex
    const operation = ++playOperation
    initializingPlay = true
    const sourceKey = playbackSourceKey(source)
    const lineStartMs = line.start_ms
    const baseStartMs = Number.isFinite(lineStartMs)
      ? lineStartMs
      : isPlayingCorrectTrack() && Number.isFinite(progress.value)
        ? progress.value * 1000
        : 0
    const seekTo = Math.max(0, baseStartMs + offsetMs) / 1000
    try {
      if (!isPlayingCorrectTrack()) await playTrack(source)
      else if (status.value === 'paused') await resume()
      // A delayed resume/load must not seek another recording or an outdated lyric.
      if (
        disposed ||
        operation !== playOperation ||
        sourceKey !== playbackSourceKey(audioSource.value) ||
        !isPlayingCorrectTrack() ||
        syncedLines.value[lineIndex] !== line
      )
        return
      await seek(seekTo)
    } catch (error) {
      if (!disposed && operation === playOperation) onError(error)
    } finally {
      if (operation === playOperation) initializingPlay = false
    }
  }

  const playLine = async lineIndex => {
    return playLineAtOffset(lineIndex)
  }

  const resumeOrPlay = async () => {
    if (disposed) return
    try {
      if (status.value !== 'stopped' && isPlayingCorrectTrack()) await resume()
      else if (audioSource.value) await playTrack(audioSource.value)
    } catch (error) {
      if (!disposed) onError(error)
    }
  }

  const toggleLoop = async () => {
    if (loopEnabled.value) {
      stopLoop()
      return
    }
    if (disposed || !audioSource.value || !loopAvailable.value || !loopRange.value) return
    const operation = ++loopOperation
    loopEnabled.value = true
    initializingLoop = true
    awaitingLoopSeek = false
    try {
      if (!isPlayingCorrectTrack()) await playTrack(audioSource.value)
      else if (status.value === 'paused') await resume()
      // A canceled/changed editor must not seek a different recording after an await.
      if (
        operation !== loopOperation ||
        !loopEnabled.value ||
        !loopAvailable.value ||
        !isPlayingCorrectTrack()
      )
        return
      if (loopRange.value) await seek(loopRange.value.start)
    } catch (error) {
      failLoop(error, operation)
    } finally {
      if (operation === loopOperation) initializingLoop = false
    }
  }

  watch(loopRange, () => {
    awaitingLoopSeek = false
  })
  watch([progress, status, loopRange, playingTrack, loopAvailable], () => {
    if (!loopEnabled.value || initializingLoop || initializingPlay) return
    if (!isPlayingCorrectTrack()) {
      stopLoop()
      return
    }
    // Untimed/reversed selections and inactive editor tabs suspend repetition,
    // retaining the user's choice until usable boundaries return.
    if (!loopAvailable.value || !loopRange.value) {
      awaitingLoopSeek = false
      return
    }
    if (status.value !== 'playing' && status.value !== 'stopped') return
    const { start, end } = loopRange.value
    if (progress.value >= start && progress.value < end) awaitingLoopSeek = false
    if ((progress.value >= end || progress.value < start) && !awaitingLoopSeek) {
      awaitingLoopSeek = true
      const operation = loopOperation
      // Both synchronous and asynchronous player failures stop the loop instead of retrying forever.
      try {
        Promise.resolve(seek(start)).catch(error => failLoop(error, operation))
      } catch (error) {
        failLoop(error, operation)
      }
    }
  })

  if (selectedLineIndex) {
    watch(
      selectedLineIndex,
      () => {
        // Invalidate an old asynchronous start/replay, keeping the new range armed.
        loopOperation++
        playOperation++
        initializingLoop = false
        initializingPlay = false
        awaitingLoopSeek = false
      },
      { flush: 'sync' }
    )
  }
  watch(
    audioSource,
    () => {
      playOperation++
      initializingPlay = false
      stopLoop()
    },
    { deep: true, flush: 'sync' }
  )
  onScopeDispose(() => {
    disposed = true
    playOperation++
    stopLoop()
  })

  return {
    loopEnabled,
    loopRange,
    loopLeadSeconds,
    loopTailSeconds,
    canLoop: computed(() => loopAvailable.value && !!loopRange.value),
    toggleLoop,
    playLine,
    playLineAtOffset,
    resumeOrPlay,
  }
}
