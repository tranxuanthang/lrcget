export const shortcutCategories = [
  {
    id: 'global',
    label: 'Global',
    shortcuts: [
      { keys: ['Ctrl', 'S'], description: 'Save lyrics' },
    ],
  },
  {
    id: 'plain',
    label: 'Plain Editor',
    shortcuts: [
      { keys: ['Ctrl', '+'], description: 'Zoom in' },
      { keys: ['Ctrl', '-'], description: 'Zoom out' },
      { keys: ['Ctrl', '0'], description: 'Reset zoom' },
      { keys: ['Ctrl', 'Scroll'], description: 'Zoom in/out' },
    ],
  },
  {
    id: 'synced',
    label: 'Synced Editor',
    shortcuts: [
      { keys: ['↑'], description: 'Select previous line' },
      { keys: ['↓'], description: 'Select next line' },
      { keys: ['Space'], description: 'Sync line to current playback' },
      { keys: ['Enter'], description: 'Sync line & move to next' },
      { keys: ['←'], description: 'Rewind 100 ms & play' },
      { keys: ['→'], description: 'Forward 100 ms & play' },
    ],
  },
  {
    id: 'wordTiming',
    label: 'Word Timing',
    shortcuts: [
      { keys: ['Z'], description: 'Sync word at current playback' },
      { keys: ['X'], description: 'Sync word & advance to next line' },
    ],
  },
]

export function getShortcutsForTab(activeTab) {
  const result = [...shortcutCategories.find(c => c.id === 'global').shortcuts]

  if (activeTab === 'plain') {
    result.push(...shortcutCategories.find(c => c.id === 'plain').shortcuts)
  } else if (activeTab === 'synced') {
    result.push(...shortcutCategories.find(c => c.id === 'synced').shortcuts)
    result.push(...shortcutCategories.find(c => c.id === 'wordTiming').shortcuts)
  }

  return result
}
