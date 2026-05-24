import { wordTimingShortcutBindings } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

export function useEditLyricsV2WordTimingHotkeys({
  isWordSyncAvailable,
  selectedBoundaryIndex,
  words,
  syncSelectedBoundaryAtProgress,
  syncSelectedBoundaryAtProgressNoAdvance,
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
