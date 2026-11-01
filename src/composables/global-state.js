import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  DEFAULT_SPECTROGRAM_THEME_ID,
  normalizeSpectrogramThemeId,
} from '@/utils/spectrogram-themes.js'
import {
  DEFAULT_SPECTROGRAM_MAX_HZ,
  DEFAULT_SPECTROGRAM_MIDPOINT_DB,
  DEFAULT_SPECTROGRAM_MIN_HZ,
  normalizeSpectrogramMaxHz,
  normalizeSpectrogramMidpointDb,
  normalizeSpectrogramMinHz,
} from '@/utils/spectrogram-settings.js'

const isHotkeyState = ref(true)
const themeModeState = ref(true)
const lrclibInstanceState = ref('')
const spectrogramVisibleState = ref(true)
const spectrogramThemeState = ref(DEFAULT_SPECTROGRAM_THEME_ID)
const spectrogramMinHzState = ref(DEFAULT_SPECTROGRAM_MIN_HZ)
const spectrogramMaxHzState = ref(DEFAULT_SPECTROGRAM_MAX_HZ)
const spectrogramMidpointDbState = ref(DEFAULT_SPECTROGRAM_MIDPOINT_DB)
let lastSavedSpectrogramMinHz = DEFAULT_SPECTROGRAM_MIN_HZ
let lastSavedSpectrogramMaxHz = DEFAULT_SPECTROGRAM_MAX_HZ
let lastSavedSpectrogramMidpointDb = DEFAULT_SPECTROGRAM_MIDPOINT_DB
// Values most recently sent to the backend. Saves skip only values already requested,
// and only the latest request may roll back on failure, so overlapping saves neither
// drop a change nor undo a newer one.
let requestedSpectrogramMinHz = DEFAULT_SPECTROGRAM_MIN_HZ
let requestedSpectrogramMaxHz = DEFAULT_SPECTROGRAM_MAX_HZ
let requestedSpectrogramMidpointDb = DEFAULT_SPECTROGRAM_MIDPOINT_DB
let spectrogramRangeRequestId = 0
let spectrogramMidpointRequestId = 0

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

  const loadSpectrogramDisplay = ({ minHz, maxHz, midpointDb }) => {
    spectrogramMinHzState.value = normalizeSpectrogramMinHz(minHz)
    spectrogramMaxHzState.value = normalizeSpectrogramMaxHz(maxHz)
    spectrogramMidpointDbState.value = normalizeSpectrogramMidpointDb(midpointDb)
    lastSavedSpectrogramMinHz = requestedSpectrogramMinHz = spectrogramMinHzState.value
    lastSavedSpectrogramMaxHz = requestedSpectrogramMaxHz = spectrogramMaxHzState.value
    lastSavedSpectrogramMidpointDb = requestedSpectrogramMidpointDb =
      spectrogramMidpointDbState.value
  }

  const setSpectrogramFrequencyRange = (minHz, maxHz) => {
    spectrogramMinHzState.value = normalizeSpectrogramMinHz(minHz)
    spectrogramMaxHzState.value = normalizeSpectrogramMaxHz(maxHz)
  }

  const saveSpectrogramFrequencyRange = async () => {
    const minHz = spectrogramMinHzState.value
    const maxHz = spectrogramMaxHzState.value
    if (minHz === requestedSpectrogramMinHz && maxHz === requestedSpectrogramMaxHz) return
    requestedSpectrogramMinHz = minHz
    requestedSpectrogramMaxHz = maxHz
    const requestId = ++spectrogramRangeRequestId
    try {
      await invoke('set_spectrogram_frequency_range', { minHz, maxHz })
      lastSavedSpectrogramMinHz = minHz
      lastSavedSpectrogramMaxHz = maxHz
    } catch (err) {
      console.error('Failed to persist spectrogram frequency range:', err)
      if (requestId !== spectrogramRangeRequestId) return
      requestedSpectrogramMinHz = spectrogramMinHzState.value = lastSavedSpectrogramMinHz
      requestedSpectrogramMaxHz = spectrogramMaxHzState.value = lastSavedSpectrogramMaxHz
    }
  }

  const setSpectrogramMidpointDb = midpointDb => {
    spectrogramMidpointDbState.value = normalizeSpectrogramMidpointDb(midpointDb)
  }

  const saveSpectrogramMidpointDb = async () => {
    const midpointDb = spectrogramMidpointDbState.value
    if (midpointDb === requestedSpectrogramMidpointDb) return
    requestedSpectrogramMidpointDb = midpointDb
    const requestId = ++spectrogramMidpointRequestId
    try {
      await invoke('set_spectrogram_midpoint', { midpointDb })
      lastSavedSpectrogramMidpointDb = midpointDb
    } catch (err) {
      console.error('Failed to persist spectrogram midpoint:', err)
      if (requestId !== spectrogramMidpointRequestId) return
      requestedSpectrogramMidpointDb = spectrogramMidpointDbState.value =
        lastSavedSpectrogramMidpointDb
    }
  }

  const resetSpectrogramSettings = async () => {
    setSpectrogramFrequencyRange(DEFAULT_SPECTROGRAM_MIN_HZ, DEFAULT_SPECTROGRAM_MAX_HZ)
    setSpectrogramMidpointDb(DEFAULT_SPECTROGRAM_MIDPOINT_DB)
    await Promise.all([
      selectSpectrogramTheme(DEFAULT_SPECTROGRAM_THEME_ID),
      saveSpectrogramFrequencyRange(),
      saveSpectrogramMidpointDb(),
    ])
  }

  const lrclibInstance = computed(() => lrclibInstanceState.value)

  const themeMode = computed(() => themeModeState.value)

  const spectrogramVisible = computed(() => spectrogramVisibleState.value)

  const spectrogramTheme = computed(() => spectrogramThemeState.value)

  const spectrogramMinHz = computed(() => spectrogramMinHzState.value)

  const spectrogramMaxHz = computed(() => spectrogramMaxHzState.value)

  const spectrogramMidpointDb = computed(() => spectrogramMidpointDbState.value)

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
    loadSpectrogramDisplay,
    setSpectrogramFrequencyRange,
    saveSpectrogramFrequencyRange,
    spectrogramMinHz,
    spectrogramMaxHz,
    setSpectrogramMidpointDb,
    saveSpectrogramMidpointDb,
    spectrogramMidpointDb,
    resetSpectrogramSettings,
  }
}
