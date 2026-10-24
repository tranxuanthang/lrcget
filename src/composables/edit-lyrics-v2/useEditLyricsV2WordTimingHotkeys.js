import { wordTimingShortcutBindings } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

export function useEditLyricsV2WordTimingHotkeys({
  isWordSyncAvailable,
  selectedBoundaryIndex,
  words,
  syncSelectedBoundaryAtProgress,
  syncSelectedBoundaryAtProgressNoAdvance,
  rewindSelectedWord,
  forwardSelectedWord,
  replaySelectedSeparator,
  replayPreviousSeparator,
  selectPreviousBoundary,
  selectNextBoundary,
  resetBoundarySelection,
  deleteSelectedBoundaries,
}) {
  const handleWordTimingKeyDown = event => {
    if (!isWordSyncAvailable.value) {
      return
    }

    const target = event.target
    if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') {
      return
    }

    for (const binding of wordTimingShortcutBindings) {
      if (!binding.matches(event)) {
        continue
      }

      event.preventDefault()
      switch (binding.id) {
        case 'selectPreviousSeparator':
          selectPreviousBoundary()
          return
        case 'selectNextSeparator':
          selectNextBoundary()
          return
        case 'syncSelectedSeparatorNoAdvance':
          syncSelectedBoundaryAtProgressNoAdvance()
          return
        case 'syncSelectedSeparatorAndAdvance':
          syncSelectedBoundaryAtProgress()
          return
        case 'rewindSelectedWord':
          rewindSelectedWord()
          return
        case 'forwardSelectedWord':
          forwardSelectedWord()
          return
        case 'replaySelectedSeparator':
          replaySelectedSeparator()
          return
        case 'replaySeparator1Back':
          replayPreviousSeparator(1)
          return
        case 'replaySeparator2Back':
          replayPreviousSeparator(2)
          return
        case 'replaySeparator3Back':
          replayPreviousSeparator(3)
          return
        case 'replaySeparator4Back':
          replayPreviousSeparator(4)
          return
        case 'replaySeparator5Back':
          replayPreviousSeparator(5)
          return
        case 'resetSeparatorCursor':
          resetBoundarySelection()
          return
        case 'deleteSelectedSeparators':
          deleteSelectedBoundaries()
          return
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
