export function useEditLyricsV2SyncedHotkeys({
  activeTab,
  isSyncedLineEditing,
  selectedLineExists,
  selectedSyncedLineIndex,
  selectedSyncedLineIndices,
  syncedLines,
  selectSyncedLine,
  clearSyncedLineSelection,
  syncLineToCurrentProgress,
  rewindLineBy100,
  forwardLineBy100,
  playLine,
}) {
  const isKeyboardTargetEditable = event => {
    const element = event.target

    if (!(element instanceof HTMLElement)) {
      return false
    }

    const tag = element.tagName.toLowerCase()
    return element.isContentEditable || tag === 'input' || tag === 'textarea' || tag === 'select'
  }

  const hasMultiSelection = () => selectedSyncedLineIndices.value.length >= 2

  const clearSelectionIfNeeded = () => {
    if (hasMultiSelection()) {
      clearSyncedLineSelection()
    }
  }

  const handleSyncedEditorKeyboardShortcuts = event => {
    if (
      activeTab.value !== 'synced' ||
      isSyncedLineEditing.value ||
      !selectedLineExists.value ||
      isKeyboardTargetEditable(event)
    ) {
      return
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault()
      clearSelectionIfNeeded()
      selectSyncedLine(Math.max(0, selectedSyncedLineIndex.value - 1))
      return
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault()
      clearSelectionIfNeeded()
      selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
      return
    }

    if (event.key === ' ') {
      event.preventDefault()
      clearSelectionIfNeeded()
      syncLineToCurrentProgress(selectedSyncedLineIndex.value)
      return
    }

    if (event.key === 'Enter') {
      event.preventDefault()
      clearSelectionIfNeeded()
      syncLineToCurrentProgress(selectedSyncedLineIndex.value)
      selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
      return
    }

    if (event.key === 'ArrowLeft') {
      event.preventDefault()
      clearSelectionIfNeeded()
      rewindLineBy100(selectedSyncedLineIndex.value)
      return
    }

    if (event.key === 'ArrowRight') {
      event.preventDefault()
      clearSelectionIfNeeded()
      forwardLineBy100(selectedSyncedLineIndex.value)
      return
    }

    if (event.key.toLowerCase() === 'p') {
      event.preventDefault()
      clearSelectionIfNeeded()
      playLine(selectedSyncedLineIndex.value)
    }
  }

  const bindSyncedHotkeys = () => {
    document.addEventListener('keydown', handleSyncedEditorKeyboardShortcuts)
  }

  const unbindSyncedHotkeys = () => {
    document.removeEventListener('keydown', handleSyncedEditorKeyboardShortcuts)
  }

  return {
    bindSyncedHotkeys,
    unbindSyncedHotkeys,
  }
}
