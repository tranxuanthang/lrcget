import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const isHotkeyState = ref(true)
const themeModeState = ref(true)
const lrclibInstanceState = ref('')
const spectrogramVisibleState = ref(true)

export function useGlobalState() {
  const disableHotkey = () => {
    console.log('disabled hotkey!')
    isHotkeyState.value = false
  }
  const enableHotkey = () => {
    console.log('enabled hotkey!')
    isHotkeyState.value = true
  }
  const isHotkey = computed(() => isHotkeyState.value)

  const setThemeMode = mode => {
    themeModeState.value = mode
  }
  const setLrclibInstance = instance => {
    lrclibInstanceState.value = instance
  }

  const setSpectrogramVisible = visible => {
    spectrogramVisibleState.value = Boolean(visible)
  }

  const toggleSpectrogramVisible = async () => {
    const newValue = !spectrogramVisibleState.value
    spectrogramVisibleState.value = newValue
    try {
      await invoke('set_spectrogram_visible', { visible: newValue })
    } catch (err) {
      console.error('Failed to persist spectrogram visibility:', err)
      spectrogramVisibleState.value = !newValue
    }
  }

  const lrclibInstance = computed(() => lrclibInstanceState.value)

  const themeMode = computed(() => themeModeState.value)

  const spectrogramVisible = computed(() => spectrogramVisibleState.value)

  return {
    isHotkey,
    disableHotkey,
    enableHotkey,
    setThemeMode,
    themeMode,
    setLrclibInstance,
    lrclibInstance,
    setSpectrogramVisible,
    toggleSpectrogramVisible,
    spectrogramVisible,
  }
}
