const SHORTCUT_OVERRIDES_STORAGE_KEY = 'lrcget.edit-lyrics-v2.shortcut-overrides'

const normalizeKey = key => (typeof key === 'string' && key.length === 1 ? key.toLowerCase() : key)

const isCtrlPressed = event => event.ctrlKey || event.metaKey

const hasExactModifiers = (event, { ctrl = false, alt = false, shift = false }) => {
  if (isCtrlPressed(event) !== ctrl) {
    return false
  }

  if (event.altKey !== alt) {
    return false
  }

  if (event.shiftKey !== shift) {
    return false
  }

  return true
}

const isShortcutKeys = keys => {
  return Array.isArray(keys) && keys.length > 0 && keys.every(key => typeof key === 'string' && key.trim())
}

const normalizeShortcutKeys = keys => {
  if (!isShortcutKeys(keys)) {
    return null
  }

  return keys.map(key => key.trim())
}

const loadShortcutOverrides = () => {
  if (typeof window === 'undefined') {
    return {}
  }

  try {
    const raw = window.localStorage.getItem(SHORTCUT_OVERRIDES_STORAGE_KEY)
    if (!raw) {
      return {}
    }

    const parsed = JSON.parse(raw)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {}
    }

    const normalized = {}
    for (const [shortcutId, keys] of Object.entries(parsed)) {
      const normalizedKeys = normalizeShortcutKeys(keys)
      if (normalizedKeys) {
        normalized[shortcutId] = normalizedKeys
      }
    }

    return normalized
  } catch {
    return {}
  }
}

const persistShortcutOverrides = overrides => {
  if (typeof window === 'undefined') {
    return
  }

  try {
    window.localStorage.setItem(SHORTCUT_OVERRIDES_STORAGE_KEY, JSON.stringify(overrides))
  } catch {
    // Ignore storage failures and keep in-memory overrides.
  }
}

let shortcutOverrides = loadShortcutOverrides()

export const getShortcutOverrides = () => ({ ...shortcutOverrides })

export const resolveShortcutKeys = (shortcutId, fallbackKeys = []) => {
  const overrideKeys = shortcutOverrides[shortcutId]
  const normalizedOverrideKeys = normalizeShortcutKeys(overrideKeys)
  if (normalizedOverrideKeys) {
    return normalizedOverrideKeys
  }

  const normalizedFallbackKeys = normalizeShortcutKeys(fallbackKeys)
  return normalizedFallbackKeys || []
}

export const setShortcutOverride = (shortcutId, keys) => {
  if (typeof shortcutId !== 'string' || !shortcutId.trim()) {
    return false
  }

  const normalizedKeys = normalizeShortcutKeys(keys)
  if (!normalizedKeys) {
    return false
  }

  shortcutOverrides = {
    ...shortcutOverrides,
    [shortcutId]: normalizedKeys,
  }

  persistShortcutOverrides(shortcutOverrides)
  return true
}

export const resetShortcutOverride = shortcutId => {
  if (typeof shortcutId !== 'string' || !shortcutId.trim()) {
    return false
  }

  if (!(shortcutId in shortcutOverrides)) {
    return true
  }

  const { [shortcutId]: _removed, ...nextOverrides } = shortcutOverrides
  shortcutOverrides = nextOverrides
  persistShortcutOverrides(shortcutOverrides)
  return true
}

export const resetAllShortcutOverrides = () => {
  shortcutOverrides = {}
  persistShortcutOverrides(shortcutOverrides)
}

const shortcutTokenToEventKey = {
  '↑': 'ArrowUp',
  '↓': 'ArrowDown',
  '←': 'ArrowLeft',
  '→': 'ArrowRight',
  Space: ' ',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Delete: 'Delete',
}

const matchesMainShortcutKey = (event, keyToken) => {
  if (keyToken === '/') {
    return normalizeKey(event.key) === '/' || normalizeKey(event.key) === '?' || event.code === 'Slash'
  }

  if (keyToken === '+') {
    return normalizeKey(event.key) === '+' || normalizeKey(event.key) === '=' || event.code === 'Equal'
  }

  if (keyToken === '-') {
    return normalizeKey(event.key) === '-' || normalizeKey(event.key) === '_' || event.code === 'Minus'
  }

  const eventKey = shortcutTokenToEventKey[keyToken] || keyToken
  return normalizeKey(event.key) === normalizeKey(eventKey)
}

const matchesShortcutKeys = (event, shortcutKeys) => {
  const normalizedKeys = normalizeShortcutKeys(shortcutKeys)
  if (!normalizedKeys) {
    return false
  }

  const ctrl = normalizedKeys.includes('Ctrl')
  const alt = normalizedKeys.includes('Alt')
  const shift = normalizedKeys.includes('Shift')
  const mainKey = normalizedKeys.find(key => key !== 'Ctrl' && key !== 'Alt' && key !== 'Shift')

  if (!mainKey) {
    return false
  }

  if (!hasExactModifiers(event, { ctrl, alt, shift })) {
    return false
  }

  return matchesMainShortcutKey(event, mainKey)
}

export const shortcutKeysFromKeyboardEvent = event => {
  const keys = []

  if (isCtrlPressed(event)) {
    keys.push('Ctrl')
  }

  if (event.altKey) {
    keys.push('Alt')
  }

  if (event.shiftKey) {
    keys.push('Shift')
  }

  const baseKey =
    event.key === ' '
      ? 'Space'
      : event.key === 'ArrowUp'
        ? '↑'
        : event.key === 'ArrowDown'
          ? '↓'
          : event.key === 'ArrowLeft'
            ? '←'
            : event.key === 'ArrowRight'
              ? '→'
              : event.key

  const ignoredBaseKeys = ['Control', 'Meta', 'Alt', 'Shift']
  if (!ignoredBaseKeys.includes(baseKey)) {
    if (baseKey.length === 1) {
      keys.push(baseKey.toUpperCase())
    } else {
      keys.push(baseKey)
    }
  }

  const hasNonModifierKey = keys.some(key => key !== 'Ctrl' && key !== 'Alt' && key !== 'Shift')
  return hasNonModifierKey ? keys : []
}

const createShortcutBinding = ({ id, defaultKeys, description }) => ({
  id,
  description,
  get defaultKeys() {
    return normalizeShortcutKeys(defaultKeys) || []
  },
  get keys() {
    return resolveShortcutKeys(id, defaultKeys)
  },
  matches: event => matchesShortcutKeys(event, resolveShortcutKeys(id, defaultKeys)),
})

export const globalShortcutBindings = [
  createShortcutBinding({
    id: 'saveLyrics',
    defaultKeys: ['Ctrl', 'S'],
    description: 'Save lyrics',
  }),
  createShortcutBinding({
    id: 'openShortcutModal',
    defaultKeys: ['Ctrl', '/'],
    description: 'Open keyboard shortcuts',
  }),
]

export const plainEditorShortcutBindings = [
  createShortcutBinding({
    id: 'zoomIn',
    defaultKeys: ['Ctrl', '+'],
    description: 'Zoom in',
  }),
  createShortcutBinding({
    id: 'zoomOut',
    defaultKeys: ['Ctrl', '-'],
    description: 'Zoom out',
  }),
  createShortcutBinding({
    id: 'zoomReset',
    defaultKeys: ['Ctrl', '0'],
    description: 'Reset zoom',
  }),
]

export const plainEditorDisplayOnlyShortcuts = [{ keys: ['Ctrl', 'Scroll'], description: 'Zoom in/out' }]

export const syncedEditorShortcutBindings = [
  createShortcutBinding({
    id: 'selectPreviousLine',
    defaultKeys: ['↑'],
    description: 'Select previous line',
  }),
  createShortcutBinding({
    id: 'selectNextLine',
    defaultKeys: ['↓'],
    description: 'Select next line',
  }),
  createShortcutBinding({
    id: 'syncLineToPlayback',
    defaultKeys: ['Space'],
    description: 'Sync line to current playback',
  }),
  createShortcutBinding({
    id: 'syncLineEndToPlayback',
    defaultKeys: ['Shift', 'Space'],
    description: 'Sync selected line end to current playback',
  }),
  createShortcutBinding({
    id: 'syncLineEndAndAdvance',
    defaultKeys: ['N'],
    description: 'Sync selected line end & move to next',
  }),
  createShortcutBinding({
    id: 'syncLineAndAdvance',
    defaultKeys: ['Enter'],
    description: 'Sync line, sync previous line end & move to next',
  }),
  createShortcutBinding({
    id: 'syncLineAndAdvanceNoPreviousEndSync',
    defaultKeys: ['Shift', 'Enter'],
    description: 'Sync line & move to next (skip line end sync)',
  }),
  createShortcutBinding({
    id: 'rewindLine',
    defaultKeys: ['←'],
    description: 'Rewind 100 ms & play',
  }),
  createShortcutBinding({
    id: 'forwardLine',
    defaultKeys: ['→'],
    description: 'Forward 100 ms & play',
  }),
  createShortcutBinding({
    id: 'replaySelectedLine',
    defaultKeys: ['P'],
    description: 'Replay selected line',
  }),
  createShortcutBinding({
    id: 'replayPreviousLine',
    defaultKeys: ['Shift', 'P'],
    description: 'Replay previous line',
  }),
  createShortcutBinding({
    id: 'deleteSelectedLine',
    defaultKeys: ['Backspace'],
    description: 'Delete selected line',
  }),
]

export const wordTimingShortcutBindings = [
  createShortcutBinding({
    id: 'selectPreviousSeparator',
    defaultKeys: ['A'],
    description: 'Select previous separator',
  }),
  createShortcutBinding({
    id: 'selectNextSeparator',
    defaultKeys: ['D'],
    description: 'Select next separator',
  }),
  createShortcutBinding({
    id: 'syncSelectedSeparatorNoAdvance',
    defaultKeys: ['S'],
    description: 'Sync selected separator (stay selected)',
  }),
  createShortcutBinding({
    id: 'syncSelectedSeparatorAndAdvance',
    defaultKeys: ['Z'],
    description: 'Sync selected separator & move to next',
  }),
  createShortcutBinding({
    id: 'resetSeparatorCursor',
    defaultKeys: ['X'],
    description: 'Reset separator cursor to second divider',
  }),
  createShortcutBinding({
    id: 'deleteSelectedSeparators',
    defaultKeys: ['Delete'],
    description: 'Delete selected separators to merge adjacent words',
  }),
]

export const getShortcutCategories = () => [
  {
    id: 'global',
    label: 'Global',
    shortcuts: globalShortcutBindings.map(({ id, keys, defaultKeys, description }) => ({
      id,
      keys,
      defaultKeys,
      description,
    })),
  },
  {
    id: 'plain',
    label: 'Plain Editor',
    shortcuts: [
      ...plainEditorShortcutBindings.map(({ id, keys, defaultKeys, description }) => ({
        id,
        keys,
        defaultKeys,
        description,
      })),
      ...plainEditorDisplayOnlyShortcuts,
    ],
  },
  {
    id: 'synced',
    label: 'Synced Editor',
    shortcuts: syncedEditorShortcutBindings.map(({ id, keys, defaultKeys, description }) => ({
      id,
      keys,
      defaultKeys,
      description,
    })),
  },
  {
    id: 'wordTiming',
    label: 'Word Timing',
    shortcuts: wordTimingShortcutBindings.map(({ id, keys, defaultKeys, description }) => ({
      id,
      keys,
      defaultKeys,
      description,
    })),
  },
]

export const shortcutCategories = getShortcutCategories()

export const formatShortcutKeys = keys => {
  if (!Array.isArray(keys) || keys.length === 0) {
    return ''
  }

  return keys.join('+')
}

export const findShortcutById = (bindings, shortcutId) => {
  if (!Array.isArray(bindings) || typeof shortcutId !== 'string') {
    return null
  }

  return bindings.find(binding => binding.id === shortcutId) || null
}

export const withShortcutTitle = (title, bindings, shortcutId) => {
  const baseTitle = typeof title === 'string' ? title : ''
  const binding = findShortcutById(bindings, shortcutId)
  const shortcutLabel = formatShortcutKeys(binding?.keys)

  if (!shortcutLabel) {
    return baseTitle
  }

  return `${baseTitle} (${shortcutLabel})`
}
