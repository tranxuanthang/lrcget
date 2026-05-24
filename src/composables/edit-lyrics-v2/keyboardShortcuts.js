import { getShortcutCategories } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

export const shortcutCategories = getShortcutCategories()

export const getKeyboardShortcutCategories = () => getShortcutCategories()

export function getShortcutsForTab(activeTab) {
  const categories = getShortcutCategories()
  const result = [...categories.find(c => c.id === 'global').shortcuts]

  if (activeTab === 'plain') {
    result.push(...categories.find(c => c.id === 'plain').shortcuts)
  } else if (activeTab === 'synced') {
    result.push(...categories.find(c => c.id === 'synced').shortcuts)
    result.push(...categories.find(c => c.id === 'wordTiming').shortcuts)
  }

  return result
}
