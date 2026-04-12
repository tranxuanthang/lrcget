export function useEditLyricsV2SyncedHotkeys({
  activeTab,
  isSyncedLineEditing,
  selectedLineExists,
  selectedSyncedLineIndex,
  syncedLines,
  selectSyncedLine,
  syncLineToCurrentProgress,
  rewindLineBy100,
  forwardLineBy100,
}) {
  const isKeyboardTargetEditable = event => {
    const element = event.target

    if (!(element instanceof HTMLElement)) {
      return false
    }

    const tag = element.tagName.toLowerCase()
    return element.isContentEditable || tag === 'input' || tag === 'textarea' || tag === 'select'
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
      selectSyncedLine(Math.max(0, selectedSyncedLineIndex.value - 1))
      return
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault()
      selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
      return
    }

    if (event.key === ' ') {
      event.preventDefault()
      syncLineToCurrentProgress(selectedSyncedLineIndex.value)
      return
    }

    if (event.key === 'Enter') {
      event.preventDefault()
      syncLineToCurrentProgress(selectedSyncedLineIndex.value)
      selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
      return
    }

    if (event.key === 'ArrowLeft') {
      event.preventDefault()
      rewindLineBy100(selectedSyncedLineIndex.value)
      return
    }

    if (event.key === 'ArrowRight') {
      event.preventDefault()
      forwardLineBy100(selectedSyncedLineIndex.value)
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
