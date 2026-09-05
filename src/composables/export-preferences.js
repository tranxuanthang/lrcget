import { readonly, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const preferences = ref(null)

export function useExportPreferences() {
  const refresh = async () => {
    preferences.value = await invoke('get_config')
    return preferences.value
  }

  // Only fields owned and visible in the submitting popup are written.
  const save = async changes => {
    preferences.value = await invoke('set_export_preferences', changes)
    return preferences.value
  }

  return { preferences: readonly(preferences), refresh, save }
}
