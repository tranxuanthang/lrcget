import { shortcutCategories } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

export { shortcutCategories }

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
