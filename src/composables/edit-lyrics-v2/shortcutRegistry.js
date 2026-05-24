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

const matchesShortcut = (event, { key, code, ctrl = false, alt = false, shift = false }) => {
  if (!hasExactModifiers(event, { ctrl, alt, shift })) {
    return false
  }

  if (code && event.code === code) {
    return true
  }

  return normalizeKey(event.key) === normalizeKey(key)
}

export const globalShortcutBindings = [
  {
    id: 'saveLyrics',
    keys: ['Ctrl', 'S'],
    description: 'Save lyrics',
    matches: event => matchesShortcut(event, { key: 's', ctrl: true }),
  },
  {
    id: 'openShortcutModal',
    keys: ['Ctrl', '/'],
    description: 'Open keyboard shortcuts',
    matches: event =>
      hasExactModifiers(event, { ctrl: true }) &&
      (normalizeKey(event.key) === '/' || normalizeKey(event.key) === '?' || event.code === 'Slash'),
  },
]

export const plainEditorShortcutBindings = [
  {
    id: 'zoomIn',
    keys: ['Ctrl', '+'],
    description: 'Zoom in',
    matches: event =>
      hasExactModifiers(event, { ctrl: true }) &&
      (normalizeKey(event.key) === '+' || normalizeKey(event.key) === '='),
  },
  {
    id: 'zoomOut',
    keys: ['Ctrl', '-'],
    description: 'Zoom out',
    matches: event =>
      hasExactModifiers(event, { ctrl: true }) &&
      (normalizeKey(event.key) === '-' || normalizeKey(event.key) === '_'),
  },
  {
    id: 'zoomReset',
    keys: ['Ctrl', '0'],
    description: 'Reset zoom',
    matches: event => matchesShortcut(event, { key: '0', ctrl: true }),
  },
]

export const plainEditorDisplayOnlyShortcuts = [
  { keys: ['Ctrl', 'Scroll'], description: 'Zoom in/out' },
]

export const syncedEditorShortcutBindings = [
  {
    id: 'selectPreviousLine',
    keys: ['↑'],
    description: 'Select previous line',
    matches: event => matchesShortcut(event, { key: 'ArrowUp' }),
  },
  {
    id: 'selectNextLine',
    keys: ['↓'],
    description: 'Select next line',
    matches: event => matchesShortcut(event, { key: 'ArrowDown' }),
  },
  {
    id: 'syncLineToPlayback',
    keys: ['Space'],
    description: 'Sync line to current playback',
    matches: event => matchesShortcut(event, { key: ' ' }),
  },
  {
    id: 'syncLineEndToPlayback',
    keys: ['Ctrl', 'Space'],
    description: 'Sync selected line end to current playback',
    matches: event => matchesShortcut(event, { key: ' ', ctrl: true }),
  },
  {
    id: 'syncLineAndAdvance',
    keys: ['Enter'],
    description: 'Sync line, sync previous line end & move to next',
    matches: event => matchesShortcut(event, { key: 'Enter' }),
  },
  {
    id: 'syncLineAndAdvanceNoPreviousEndSync',
    keys: ['Ctrl', 'Enter'],
    description: 'Sync line & move to next (skip previous line end sync)',
    matches: event => matchesShortcut(event, { key: 'Enter', ctrl: true }),
  },
  {
    id: 'rewindLine',
    keys: ['←'],
    description: 'Rewind 100 ms & play',
    matches: event => matchesShortcut(event, { key: 'ArrowLeft' }),
  },
  {
    id: 'forwardLine',
    keys: ['→'],
    description: 'Forward 100 ms & play',
    matches: event => matchesShortcut(event, { key: 'ArrowRight' }),
  },
  {
    id: 'replaySelectedLine',
    keys: ['P'],
    description: 'Replay selected line',
    matches: event => matchesShortcut(event, { key: 'p', ctrl: false, shift: false }) && !event.altKey,
  },
  {
    id: 'deleteSelectedLine',
    keys: ['Backspace'],
    description: 'Delete selected line',
    matches: event => matchesShortcut(event, { key: 'Backspace' }),
  },
]

export const wordTimingShortcutBindings = [
  {
    id: 'syncSelectedSeparatorAndAdvance',
    keys: ['Z'],
    description: 'Sync selected separator & move to next',
    matches: event => matchesShortcut(event, { key: 'z' }),
  },
  {
    id: 'resetSeparatorCursor',
    keys: ['X'],
    description: 'Reset separator cursor to second divider',
    matches: event => matchesShortcut(event, { key: 'x' }),
  },
]

export const shortcutCategories = [
  {
    id: 'global',
    label: 'Global',
    shortcuts: globalShortcutBindings.map(({ keys, description }) => ({ keys, description })),
  },
  {
    id: 'plain',
    label: 'Plain Editor',
    shortcuts: [
      ...plainEditorShortcutBindings.map(({ keys, description }) => ({ keys, description })),
      ...plainEditorDisplayOnlyShortcuts,
    ],
  },
  {
    id: 'synced',
    label: 'Synced Editor',
    shortcuts: syncedEditorShortcutBindings.map(({ keys, description }) => ({ keys, description })),
  },
  {
    id: 'wordTiming',
    label: 'Word Timing',
    shortcuts: wordTimingShortcutBindings.map(({ keys, description }) => ({ keys, description })),
  },
]
