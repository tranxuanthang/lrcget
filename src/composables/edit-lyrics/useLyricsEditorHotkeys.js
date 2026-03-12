export function useLyricsEditorHotkeys(config) {
  let keydownEvent = null

  const createHotkeyHandler = (hotkeyConfig) => (event) => {
    for (const { keys, handler } of hotkeyConfig) {
      const [modifier, key] = keys.split('+')
      const expectedModifier = modifier.toLowerCase()
      const expectedKey = key.toLowerCase()

      const modifierMatch = {
        alt: event.altKey,
        ctrl: event.ctrlKey || event.metaKey
      }[expectedModifier]

      if (modifierMatch && event.key.toLowerCase() === expectedKey) {
        event.preventDefault()
        handler()
        break
      }
    }
  }

  const bindHotkeys = () => {
    if (keydownEvent) {
      return
    }

    keydownEvent = createHotkeyHandler(config)
    document.addEventListener('keydown', keydownEvent)
  }

  const unbindHotkeys = () => {
    if (!keydownEvent) {
      return
    }

    document.removeEventListener('keydown', keydownEvent)
    keydownEvent = null
  }

  return {
    bindHotkeys,
    unbindHotkeys
  }
}
