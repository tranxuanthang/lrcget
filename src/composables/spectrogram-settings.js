import { readonly, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  DEFAULT_SPECTROGRAM_THEME_ID,
  normalizeSpectrogramThemeId,
} from '@/utils/spectrogram-themes.js'
import {
  DEFAULT_SPECTROGRAM_MAX_HZ,
  DEFAULT_SPECTROGRAM_MIDPOINT_DB,
  DEFAULT_SPECTROGRAM_MIN_HZ,
  normalizeSpectrogramFrequencyRange,
  normalizeSpectrogramMidpointDb,
} from '@/utils/spectrogram-settings.js'

const DEFAULT_SETTINGS = Object.freeze({
  visible: true,
  theme: DEFAULT_SPECTROGRAM_THEME_ID,
  minHz: DEFAULT_SPECTROGRAM_MIN_HZ,
  maxHz: DEFAULT_SPECTROGRAM_MAX_HZ,
  midpointDb: DEFAULT_SPECTROGRAM_MIDPOINT_DB,
})

const settings = ref({ ...DEFAULT_SETTINGS })

export function useSpectrogramSettings() {
  const load = config => {
    settings.value = {
      visible: Boolean(config.spectrogram_visible),
      theme: normalizeSpectrogramThemeId(config.spectrogram_theme),
      ...normalizeSpectrogramFrequencyRange(config.spectrogram_min_hz, config.spectrogram_max_hz),
      midpointDb: normalizeSpectrogramMidpointDb(config.spectrogram_midpoint_db),
    }
  }

  // Unlike useExportPreferences, applies changes immediately so sliders preview them live.
  const save = async changes => {
    settings.value = { ...settings.value, ...changes }
    try {
      await invoke('set_spectrogram_settings', changes)
    } catch (err) {
      console.error('Failed to save spectrogram settings:', err)
    }
  }

  const reset = () => save(DEFAULT_SETTINGS)

  return { settings: readonly(settings), load, save, reset }
}
