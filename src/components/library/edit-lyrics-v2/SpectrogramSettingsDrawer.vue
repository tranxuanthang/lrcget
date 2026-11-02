<template>
  <div
    id="spectrogram-settings-drawer"
    ref="drawerElement"
    role="group"
    aria-label="Spectrogram settings"
    class="drawer"
    :data-keyboard-active="keyboardActive || undefined"
    @keydown="handleKeydown"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @pointerdown="keyboardActive = false"
    @focusin="handleFocusIn"
    @focusout="handleFocusOut"
  >
    <span
      id="spectrogram-settings-color-label"
      class="row-label col-start-1 row-start-1"
      :class="{ 'row-label-active': activeRow === 'color' }"
      data-focuses-color
    >
      Color
    </span>
    <div
      class="col-start-2 row-start-1 flex items-center gap-3"
      role="radiogroup"
      aria-labelledby="spectrogram-settings-color-label"
    >
      <input
        v-for="theme in SPECTROGRAM_THEMES"
        :key="theme.id"
        :ref="element => setSwatchElement(theme.id, element)"
        type="radio"
        name="spectrogram-color"
        class="swatch"
        :tabindex="theme.id === spectrogramTheme ? 0 : -1"
        :value="theme.id"
        :checked="theme.id === spectrogramTheme"
        :aria-label="theme.label"
        :title="theme.label"
        :style="{ backgroundColor: theme.swatchColor }"
        @change="selectSpectrogramTheme(theme.id)"
      />
    </div>
    <span class="readout col-start-3 row-start-1">{{ selectedThemeLabel }}</span>

    <label
      for="spectrogram-settings-min-hz"
      class="row-label col-start-1 row-start-2 self-start"
      :class="{ 'row-label-active': activeRow === 'frequency' }"
    >
      Frequency
    </label>
    <div class="frequency-axis col-start-2 row-start-2">
      <div class="axis-track" />
      <div class="axis-band" :style="bandStyle" />
      <span
        v-for="(hz, index) in SPECTROGRAM_AXIS_STOPS_HZ"
        :key="hz"
        class="axis-stop"
        :class="{ 'axis-stop-marker': index === MIDPOINT_STOP_INDEX }"
        :style="{ left: stopCenter(index) }"
      >
        {{ formatSpectrogramAxisStop(hz) }}
      </span>
      <input
        id="spectrogram-settings-min-hz"
        ref="minHzInput"
        type="range"
        class="range-input frequency-input"
        tabindex="0"
        min="0"
        :max="SPECTROGRAM_MIN_HZ_STOPS.length - 1"
        step="1"
        :value="minHzIndex"
        :style="{ left: '0px', width: HANDLE_INPUT_WIDTH }"
        aria-label="Lowest frequency"
        :aria-valuetext="formatSpectrogramFrequency(spectrogramMinHz)"
        @input="handleMinHzInput"
        @change="saveSpectrogramFrequencyRange"
      />
      <input
        ref="maxHzInput"
        type="range"
        class="range-input frequency-input"
        tabindex="0"
        min="0"
        :max="SPECTROGRAM_MAX_HZ_STOPS.length - 1"
        step="1"
        :value="maxHzIndex"
        :style="{ left: stopOffset(MAX_HZ_FIRST_STOP_INDEX), width: HANDLE_INPUT_WIDTH }"
        aria-label="Highest frequency"
        :aria-valuetext="formatSpectrogramFrequency(spectrogramMaxHz)"
        @input="handleMaxHzInput"
        @change="saveSpectrogramFrequencyRange"
      />
    </div>
    <span class="readout col-start-3 row-start-2 self-start">
      {{ formatSpectrogramFrequencyRange(spectrogramMinHz, spectrogramMaxHz) }}
    </span>

    <label
      for="spectrogram-settings-sensitivity"
      class="row-label col-start-1 row-start-3"
      :class="{ 'row-label-active': activeRow === 'sensitivity' }"
    >
      Sensitivity
    </label>
    <input
      id="spectrogram-settings-sensitivity"
      ref="sensitivityInput"
      type="range"
      class="range-input sensitivity-input col-start-2 row-start-3"
      tabindex="0"
      :min="-SPECTROGRAM_MIDPOINT_SHALLOWEST_DB"
      :max="-SPECTROGRAM_MIDPOINT_DEEPEST_DB"
      :step="SPECTROGRAM_MIDPOINT_STEP_DB"
      :value="-spectrogramMidpointDb"
      :aria-valuetext="formatSpectrogramMidpoint(spectrogramMidpointDb)"
      @input="handleSensitivityInput"
      @change="saveSpectrogramMidpointDb"
    />
    <span class="readout col-start-3 row-start-3">
      {{ formatSpectrogramMidpoint(spectrogramMidpointDb) }}
    </span>

    <button
      ref="defaultsButton"
      type="button"
      tabindex="0"
      class="defaults-button button button-normal col-start-4 row-start-1"
      title="Restore default spectrogram settings"
      @click="resetSpectrogramSettings"
    >
      <Restore class="w-3.5 h-3.5" />
      <span>Defaults</span>
    </button>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import Restore from '~icons/mdi/restore'
import { useGlobalState } from '@/composables/global-state.js'
import { globalShortcutBindings } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'
import { SPECTROGRAM_THEMES } from '@/utils/spectrogram-themes.js'
import {
  SPECTROGRAM_AXIS_STOPS_HZ,
  SPECTROGRAM_MAX_HZ_STOPS,
  SPECTROGRAM_MIDPOINT_DEEPEST_DB,
  SPECTROGRAM_MIDPOINT_SHALLOWEST_DB,
  SPECTROGRAM_MIDPOINT_STEP_DB,
  SPECTROGRAM_MIN_HZ_STOPS,
  formatSpectrogramAxisStop,
  formatSpectrogramFrequency,
  formatSpectrogramFrequencyRange,
  formatSpectrogramMidpoint,
} from '@/utils/spectrogram-settings.js'

const emit = defineEmits(['close'])

const {
  spectrogramTheme,
  selectSpectrogramTheme,
  spectrogramMinHz,
  spectrogramMaxHz,
  setSpectrogramFrequencyRange,
  saveSpectrogramFrequencyRange,
  spectrogramMidpointDb,
  setSpectrogramMidpointDb,
  saveSpectrogramMidpointDb,
  resetSpectrogramSettings,
} = useGlobalState()

// Native range thumbs travel from one radius in from each end of the input, so with
// 14px thumbs stop i of the 9-stop axis is centered at 7px + i/8 of the remaining width.
// Each handle's input spans only its own four stops, so the two never overlap.
const AXIS_INTERVALS = SPECTROGRAM_AXIS_STOPS_HZ.length - 1
const MIDPOINT_STOP_INDEX = SPECTROGRAM_MIN_HZ_STOPS.length
const MAX_HZ_FIRST_STOP_INDEX = SPECTROGRAM_MIN_HZ_STOPS.length + 1
const stopOffset = index => `calc((100% - 14px) * ${index} / ${AXIS_INTERVALS})`
const stopCenter = index => `calc(7px + (100% - 14px) * ${index} / ${AXIS_INTERVALS})`
const HANDLE_INPUT_WIDTH = `calc((100% - 14px) * ${SPECTROGRAM_MIN_HZ_STOPS.length - 1} / ${AXIS_INTERVALS} + 14px)`

const drawerElement = ref(null)
const minHzInput = ref(null)
const maxHzInput = ref(null)
const sensitivityInput = ref(null)
const defaultsButton = ref(null)
const swatchElements = new Map()
const keyboardActive = ref(false)
const activeRow = ref(null)
let lastFocusedControl = null

const setSwatchElement = (themeId, element) => {
  if (element) swatchElements.set(themeId, element)
  else swatchElements.delete(themeId)
}

const selectedThemeLabel = computed(
  () => SPECTROGRAM_THEMES.find(theme => theme.id === spectrogramTheme.value)?.label ?? ''
)
const minHzIndex = computed(() => SPECTROGRAM_MIN_HZ_STOPS.indexOf(spectrogramMinHz.value))
const maxHzIndex = computed(() => SPECTROGRAM_MAX_HZ_STOPS.indexOf(spectrogramMaxHz.value))
const bandStyle = computed(() => {
  const firstStop = minHzIndex.value
  const lastStop = MAX_HZ_FIRST_STOP_INDEX + maxHzIndex.value
  return {
    left: stopCenter(firstStop),
    width: `calc((100% - 14px) * ${lastStop - firstStop} / ${AXIS_INTERVALS})`,
  }
})

const handleMinHzInput = event =>
  setSpectrogramFrequencyRange(
    SPECTROGRAM_MIN_HZ_STOPS[Number(event.target.value)],
    spectrogramMaxHz.value
  )

const handleMaxHzInput = event =>
  setSpectrogramFrequencyRange(
    spectrogramMinHz.value,
    SPECTROGRAM_MAX_HZ_STOPS[Number(event.target.value)]
  )

const handleSensitivityInput = event => setSpectrogramMidpointDb(-Number(event.target.value))

const checkedSwatch = () => swatchElements.get(spectrogramTheme.value)
const isSwatch = element => [...swatchElements.values()].includes(element)

const focusControl = element => element?.focus({ preventScroll: true })

const focusFirstControl = () => {
  keyboardActive.value = false
  focusControl(checkedSwatch())
}

// Tab cycles every control; Up/Down move between rows, landing on each row's default
// control. Defaults is the last row even though it sits at the top right.
const tabStops = () => [
  checkedSwatch(),
  minHzInput.value,
  maxHzInput.value,
  sensitivityInput.value,
  defaultsButton.value,
]

const rows = () => [checkedSwatch(), minHzInput.value, sensitivityInput.value, defaultsButton.value]

const rowIndexOf = element => {
  if (isSwatch(element)) return 0
  if (element === minHzInput.value || element === maxHzInput.value) return 1
  if (element === sensitivityInput.value) return 2
  if (element === defaultsButton.value) return 3
  return -1
}

const ROW_NAMES = ['color', 'frequency', 'sensitivity', null]

const cycleFocus = (controls, currentIndex, direction) => {
  const nextIndex =
    currentIndex < 0
      ? direction > 0
        ? 0
        : controls.length - 1
      : (currentIndex + direction + controls.length) % controls.length
  focusControl(controls[nextIndex])
}

const stepColor = direction => {
  const currentIndex = SPECTROGRAM_THEMES.findIndex(theme => theme.id === spectrogramTheme.value)
  const nextIndex = Math.max(0, Math.min(SPECTROGRAM_THEMES.length - 1, currentIndex + direction))
  const nextTheme = SPECTROGRAM_THEMES[nextIndex]
  if (nextIndex !== currentIndex) selectSpectrogramTheme(nextTheme.id)
  focusControl(swatchElements.get(nextTheme.id))
}

const handleKeydown = event => {
  if (globalShortcutBindings.some(binding => binding.matches(event))) return

  // Editor hotkeys listen on document/window and act on non-input targets such as the
  // Defaults button (Backspace deletes the line, Space/Enter sync it), and the editor
  // modal's focus trap releases itself on Escape, so no key may leave the drawer.
  event.stopPropagation()
  keyboardActive.value = true

  if (event.ctrlKey || event.altKey || event.metaKey) return

  const target = event.target
  switch (event.key) {
    case 'Tab': {
      event.preventDefault()
      const stops = tabStops()
      const currentIndex = isSwatch(target) ? 0 : stops.indexOf(target)
      cycleFocus(stops, currentIndex, event.shiftKey ? -1 : 1)
      return
    }
    case 'ArrowUp':
    case 'ArrowDown':
      event.preventDefault()
      cycleFocus(rows(), rowIndexOf(target), event.key === 'ArrowDown' ? 1 : -1)
      return
    case 'ArrowLeft':
    case 'ArrowRight':
      if (isSwatch(target)) {
        event.preventDefault()
        stepColor(event.key === 'ArrowRight' ? 1 : -1)
      }
      return
    case 'Escape':
      event.preventDefault()
      if (target === maxHzInput.value) focusControl(minHzInput.value)
      else emit('close')
      return
  }
}

const handleMouseDown = event => {
  if (event.target.closest('[data-focuses-color]')) {
    event.preventDefault()
    focusControl(checkedSwatch())
    return
  }
  if (event.target.closest('input, button, label')) return

  // Clicks on empty parts of the drawer keep keyboard focus in it instead of dropping it.
  event.preventDefault()
  if (!drawerElement.value?.contains(document.activeElement)) {
    focusControl(lastFocusedControl ?? checkedSwatch())
  }
}

// WebKit on macOS doesn't move focus to form controls on click, so focus the clicked
// control (or a clicked label's control) to keep "click, then use the keyboard" working.
const handleClick = event => {
  const control = event.target.closest('input, button') ?? event.target.closest('label')?.control
  if (control && document.activeElement !== control) focusControl(control)
}

const handleFocusIn = event => {
  const rowIndex = rowIndexOf(event.target)
  if (rowIndex >= 0) lastFocusedControl = event.target
  activeRow.value = rowIndex >= 0 ? ROW_NAMES[rowIndex] : null
}

const handleFocusOut = event => {
  if (!drawerElement.value?.contains(event.relatedTarget)) activeRow.value = null
}

// Explicit because some editor controls cancel pointerdown, which can stop the browser
// from moving focus out of the drawer on its own.
const handleDocumentPointerDown = event => {
  const drawer = drawerElement.value
  if (!drawer || drawer.contains(event.target)) return
  if (drawer.contains(document.activeElement)) document.activeElement.blur()
}

onMounted(() => {
  document.addEventListener('pointerdown', handleDocumentPointerDown, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', handleDocumentPointerDown, true)
})

defineExpose({ focusFirstControl })
</script>

<style scoped>
.drawer {
  @apply grid items-center h-[7rem] px-3 py-2 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900;
  grid-template-columns: auto 18rem auto 1fr;
  grid-template-rows: 1.5rem 2.25rem 1.25rem;
  column-gap: 1rem;
  row-gap: 0.375rem;
  align-content: center;
}

.row-label {
  @apply text-xs text-neutral-600 dark:text-neutral-400 select-none;
}

.row-label-active {
  @apply text-hoa-1100 dark:text-hoa-500;
}

.readout {
  @apply text-xs tabular-nums whitespace-nowrap text-neutral-700 dark:text-neutral-300;
}

.swatch {
  @apply h-5 w-5 m-0 shrink-0 cursor-pointer appearance-none rounded-full ring-1 ring-black/10 dark:ring-white/15;
}

.swatch:checked {
  @apply ring-2 ring-hoa-1100 ring-offset-2 ring-offset-white dark:ring-offset-neutral-900;
}

.swatch:focus {
  outline: none;
}

.swatch:focus-visible,
.drawer[data-keyboard-active] .swatch:focus {
  @apply outline outline-2 outline-offset-4 outline-hoa-1100/60;
}

.frequency-axis {
  @apply relative h-full;
}

.axis-track {
  @apply absolute inset-x-0 h-0.5 rounded-full bg-neutral-300 dark:bg-neutral-600 pointer-events-none;
  top: 6px;
}

.axis-band {
  @apply absolute h-1 rounded-full bg-hoa-1100 pointer-events-none;
  top: 5px;
}

.axis-stop {
  @apply absolute -translate-x-1/2 whitespace-nowrap text-[10px] leading-3 text-neutral-500 dark:text-neutral-400 pointer-events-none select-none;
  top: 1.25rem;
}

.axis-stop-marker {
  @apply text-neutral-400 dark:text-neutral-500;
}

.range-input {
  @apply m-0 h-3.5 cursor-pointer appearance-none bg-transparent;
}

.range-input:focus {
  outline: none;
}

.frequency-input {
  @apply absolute top-0;
}

.sensitivity-input {
  @apply w-full;
}

.range-input::-webkit-slider-runnable-track {
  height: 14px;
  background: transparent;
}

.sensitivity-input::-webkit-slider-runnable-track {
  @apply h-1 rounded-full bg-neutral-300 dark:bg-neutral-600;
}

.range-input::-moz-range-track {
  height: 14px;
  background: transparent;
}

.sensitivity-input::-moz-range-track {
  @apply h-1 rounded-full bg-neutral-300 dark:bg-neutral-600;
}

.range-input::-webkit-slider-thumb {
  @apply h-3.5 w-3.5 appearance-none rounded-full border-2 border-solid border-hoa-1100 bg-white shadow;
  box-sizing: border-box;
}

.sensitivity-input::-webkit-slider-thumb {
  margin-top: -5px;
}

.range-input::-moz-range-thumb {
  @apply h-3.5 w-3.5 rounded-full border-2 border-solid border-hoa-1100 bg-white shadow;
  box-sizing: border-box;
}

.range-input:focus-visible::-webkit-slider-thumb,
.drawer[data-keyboard-active] .range-input:focus::-webkit-slider-thumb {
  box-shadow: 0 0 0 3px theme('colors.hoa.1100 / 45%');
}

.range-input:focus-visible::-moz-range-thumb,
.drawer[data-keyboard-active] .range-input:focus::-moz-range-thumb {
  box-shadow: 0 0 0 3px theme('colors.hoa.1100 / 45%');
}

.defaults-button {
  @apply flex items-center gap-1 justify-self-end self-center text-xs px-2 py-0.5 rounded;
}

.defaults-button:focus {
  outline: none;
}

.defaults-button:focus-visible,
.drawer[data-keyboard-active] .defaults-button:focus {
  @apply ring-2 ring-hoa-1100/60;
}
</style>
