import {
  globalShortcutBindings,
  plainEditorShortcutBindings,
} from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

export function useEditLyricsV2Hotkeys({
  activeTab,
  saveLyrics,
  changeFontSizeBy,
  resetFontSize,
  openShortcutsModal,
}) {
  const runGlobalAction = actionId => {
    switch (actionId) {
      case 'openShortcutModal':
        openShortcutsModal()
        return
      case 'saveLyrics':
        saveLyrics()
        return
    }
  }

  const runPlainAction = actionId => {
    switch (actionId) {
      case 'zoomIn':
        changeFontSizeBy(1)
        return
      case 'zoomOut':
        changeFontSizeBy(-1)
        return
      case 'zoomReset':
        resetFontSize()
        return
    }
  }

  const handleKeyboardShortcuts = event => {
    for (const binding of globalShortcutBindings) {
      if (!binding.matches(event)) {
        continue
      }

      event.preventDefault()
      runGlobalAction(binding.id)
      return
    }

    if (activeTab.value !== 'plain') {
      return
    }

    for (const binding of plainEditorShortcutBindings) {
      if (!binding.matches(event)) {
        continue
      }

      event.preventDefault()
      runPlainAction(binding.id)
      return
    }
  }

  const bindHotkeys = () => {
    document.addEventListener('keydown', handleKeyboardShortcuts)
  }

  const unbindHotkeys = () => {
    document.removeEventListener('keydown', handleKeyboardShortcuts)
  }

  return {
    bindHotkeys,
    unbindHotkeys,
  }
}
