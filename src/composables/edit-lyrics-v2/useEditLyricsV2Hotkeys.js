export function useEditLyricsV2Hotkeys({ activeTab, saveLyrics, changeFontSizeBy, resetFontSize }) {
  const isCtrlPressed = event => event.ctrlKey || event.metaKey

  const handleKeyboardShortcuts = event => {
    if (!isCtrlPressed(event)) {
      return
    }

    const key = event.key.toLowerCase()

    if (key === 's') {
      event.preventDefault()
      saveLyrics()
      return
    }

    if (activeTab.value !== 'plain') {
      return
    }

    if (key === '+' || key === '=') {
      event.preventDefault()
      changeFontSizeBy(1)
      return
    }

    if (key === '-' || key === '_') {
      event.preventDefault()
      changeFontSizeBy(-1)
      return
    }

    if (key === '0') {
      event.preventDefault()
      resetFontSize()
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
