export function useEditLyricsV2WordTimingHotkeys({
  isWordSyncAvailable,
  selectedBoundaryIndex,
  words,
  selectedLineIndex,
  allLines,
  syncSelectedBoundaryAtProgress,
  onSelectNextLine,
}) {
  const handleWordTimingKeyDown = event => {
    if (!isWordSyncAvailable.value) {
      return
    }

    const target = event.target
    if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') {
      return
    }

    switch (event.key.toLowerCase()) {
      case 'z':
        event.preventDefault()
        syncSelectedBoundaryAtProgress()
        break
      case 'x': {
        event.preventDefault()
        const isAtLastBoundary = selectedBoundaryIndex.value >= words.value.length - 1
        syncSelectedBoundaryAtProgress()

        if (!isAtLastBoundary) {
          break
        }

        const nextLineIndex = selectedLineIndex.value + 1
        if (nextLineIndex < allLines.value.length) {
          onSelectNextLine(nextLineIndex)
        }
        break
      }
    }
  }

  const bindWordTimingHotkeys = () => {
    window.addEventListener('keydown', handleWordTimingKeyDown)
  }

  const unbindWordTimingHotkeys = () => {
    window.removeEventListener('keydown', handleWordTimingKeyDown)
  }

  return {
    bindWordTimingHotkeys,
    unbindWordTimingHotkeys,
  }
}
