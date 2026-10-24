import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  DEFAULT_SPECTROGRAM_THEME_ID,
  normalizeSpectrogramThemeId,
} from '@/utils/spectrogram-themes.js'

const isHotkeyState = ref(true)
const themeModeState = ref(true)
const lrclibInstanceState = ref('')
const spectrogramVisibleState = ref(true)
const spectrogramThemeState = ref(DEFAULT_SPECTROGRAM_THEME_ID)

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

  const persistSpectrogramVisible = async visible => {
    const previousValue = spectrogramVisibleState.value
    if (previousValue === visible) return
    spectrogramVisibleState.value = visible
    try {
      await invoke('set_spectrogram_visible', { visible })
    } catch (err) {
      console.error('Failed to persist spectrogram visibility:', err)
      spectrogramVisibleState.value = previousValue
    }
  }

  const toggleSpectrogramVisible = () => persistSpectrogramVisible(!spectrogramVisibleState.value)

  const showSpectrogram = () => persistSpectrogramVisible(true)

  const setSpectrogramTheme = themeId => {
    spectrogramThemeState.value = normalizeSpectrogramThemeId(themeId)
  }

  const selectSpectrogramTheme = async themeId => {
    const nextTheme = normalizeSpectrogramThemeId(themeId)
    const previousTheme = spectrogramThemeState.value
    if (nextTheme === previousTheme) return
    spectrogramThemeState.value = nextTheme
    try {
      await invoke('set_spectrogram_theme', { theme: nextTheme })
    } catch (err) {
      console.error('Failed to persist spectrogram theme:', err)
      spectrogramThemeState.value = previousTheme
    }
  }

  const lrclibInstance = computed(() => lrclibInstanceState.value)

  const themeMode = computed(() => themeModeState.value)

  const spectrogramVisible = computed(() => spectrogramVisibleState.value)

  const spectrogramTheme = computed(() => spectrogramThemeState.value)

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
    showSpectrogram,
    spectrogramVisible,
    setSpectrogramTheme,
    selectSpectrogramTheme,
    spectrogramTheme,
  }
}
