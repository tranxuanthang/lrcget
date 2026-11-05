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
      data-row="0"
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
        :tabindex="theme.id === settings.theme ? 0 : -1"
        :value="theme.id"
        :checked="theme.id === settings.theme"
        :aria-label="theme.label"
        :title="theme.label"
        :style="{ backgroundColor: theme.swatchColor }"
        @change="save({ theme: theme.id })"
      />
    </div>
    <span class="readout col-start-3 row-start-1">{{ selectedThemeLabel }}</span>

    <span
      id="spectrogram-settings-frequency-label"
      class="row-label col-start-1 row-start-2 self-start"
      :class="{ 'row-label-active': activeRow === 'frequency' }"
      data-row="1"
    >
      Frequency
    </span>
    <VueSlider
      ref="frequencySlider"
      class="slider col-start-2 row-start-2 self-start"
      :model-value="[settings.minHz, settings.maxHz]"
      :data="SPECTROGRAM_AXIS_STOPS_HZ"
      :marks="axisStopMark"
      :min-range="SPECTROGRAM_MIN_STOP_GAP"
      v-bind="SLIDER_OPTIONS"
      :tooltip-formatter="formatSpectrogramFrequency"
      :dot-attrs="{ 'aria-labelledby': 'spectrogram-settings-frequency-label' }"
      @change="handleFrequencyChange"
    />
    <span class="readout col-start-3 row-start-2 self-start">{{ frequencyReadout }}</span>

    <span
      id="spectrogram-settings-sensitivity-label"
      class="row-label col-start-1 row-start-3"
      :class="{ 'row-label-active': activeRow === 'sensitivity' }"
      data-row="2"
    >
      Sensitivity
    </span>
    <VueSlider
      ref="sensitivitySlider"
      class="slider col-start-2 row-start-3"
      :model-value="-settings.midpointDb"
      :min="-SPECTROGRAM_MIDPOINT_SHALLOWEST_DB"
      :max="-SPECTROGRAM_MIDPOINT_DEEPEST_DB"
      :interval="SPECTROGRAM_MIDPOINT_STEP_DB"
      :process="false"
      v-bind="SLIDER_OPTIONS"
      :tooltip-formatter="value => formatSpectrogramMidpoint(-value)"
      :dot-attrs="{ 'aria-labelledby': 'spectrogram-settings-sensitivity-label' }"
      @change="handleSensitivityChange"
    />
    <span class="readout col-start-3 row-start-3">
      {{ formatSpectrogramMidpoint(settings.midpointDb) }}
    </span>

    <button
      ref="defaultsButton"
      type="button"
      tabindex="0"
      class="defaults-button button button-normal col-start-4 row-start-1"
      title="Restore default spectrogram settings"
      @click="reset"
    >
      <Restore class="w-3.5 h-3.5" />
      <span>Defaults</span>
    </button>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import VueSlider from 'vue-3-slider-component'
import Restore from '~icons/mdi/restore'
import { useSpectrogramSettings } from '@/composables/spectrogram-settings.js'
import { globalShortcutBindings } from '@/composables/edit-lyrics-v2/shortcutRegistry.js'
import { SPECTROGRAM_THEMES } from '@/utils/spectrogram-themes.js'
import {
  SPECTROGRAM_AXIS_STOPS_HZ,
  SPECTROGRAM_MIDPOINT_DEEPEST_DB,
  SPECTROGRAM_MIDPOINT_SHALLOWEST_DB,
  SPECTROGRAM_MIDPOINT_STEP_DB,
  SPECTROGRAM_MIN_STOP_GAP,
  clampSpectrogramMaxStop,
  clampSpectrogramMinStop,
  displayedSpectrogramRange,
  formatSpectrogramAxisStop,
  formatSpectrogramFrequency,
  formatSpectrogramMidpoint,
  formatSpectrogramRangeReadout,
  normalizeSpectrogramMidpointDb,
  spectrogramStopIndex,
} from '@/utils/spectrogram-settings.js'

const props = defineProps({
  sampleRate: { type: Number, default: null },
})

const emit = defineEmits(['close'])

const { settings, save, reset } = useSpectrogramSettings()

// The drawer handles arrow keys itself; the sliders' own handler listens on the whole document.
const SLIDER_OPTIONS = { contained: true, duration: 0, useKeyboard: false, tooltip: 'none' }
const LAST_STOP_INDEX = SPECTROGRAM_AXIS_STOPS_HZ.length - 1

const axisStopMark = hz => ({ label: formatSpectrogramAxisStop(hz) })

const drawerElement = ref(null)
const frequencySlider = ref(null)
const sensitivitySlider = ref(null)
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
  () => SPECTROGRAM_THEMES.find(theme => theme.id === settings.value.theme)?.label ?? ''
)

const frequencyReadout = computed(() => {
  const { minHz, maxHz } = settings.value
  const displayedRange = props.sampleRate
    ? displayedSpectrogramRange(minHz, maxHz, props.sampleRate)
    : null
  return formatSpectrogramRangeReadout({ minHz, maxHz }, displayedRange)
})

const sliderHandles = slider => [...(slider?.$el.querySelectorAll('[role="slider"]') ?? [])]
const checkedSwatch = () => swatchElements.get(settings.value.theme)
const isSwatch = element => [...swatchElements.values()].includes(element)

// Rows in Up/Down order; Tab visits every control, ending on Defaults even though it sits top right.
const rows = () => [
  { name: 'color', controls: [checkedSwatch()] },
  { name: 'frequency', controls: sliderHandles(frequencySlider.value) },
  { name: 'sensitivity', controls: sliderHandles(sensitivitySlider.value) },
  { name: 'defaults', controls: [defaultsButton.value] },
]

const rowIndexOf = element =>
  isSwatch(element) ? 0 : rows().findIndex(row => row.controls.includes(element))

const focusControl = element => element?.focus({ preventScroll: true })

const focusFirstControl = () => {
  keyboardActive.value = false
  focusControl(checkedSwatch())
}

const cycleFocus = (controls, currentIndex, direction) => {
  const nextIndex =
    currentIndex < 0
      ? direction > 0
        ? 0
        : controls.length - 1
      : (currentIndex + direction + controls.length) % controls.length
  focusControl(controls[nextIndex])
}

const saveFrequencyStops = (minIndex, maxIndex) =>
  save({ minHz: SPECTROGRAM_AXIS_STOPS_HZ[minIndex], maxHz: SPECTROGRAM_AXIS_STOPS_HZ[maxIndex] })

// Like a slider drag, the moved handle pushes the other one to keep the minimum gap.
const stepFrequency = (handleIndex, step) => {
  const minIndex = spectrogramStopIndex(settings.value.minHz)
  const maxIndex = spectrogramStopIndex(settings.value.maxHz)
  if (handleIndex === 0) {
    const nextMin = clampSpectrogramMinStop(minIndex + step, LAST_STOP_INDEX)
    saveFrequencyStops(nextMin, clampSpectrogramMaxStop(maxIndex, nextMin))
  } else {
    const nextMax = clampSpectrogramMaxStop(maxIndex + step, 0)
    saveFrequencyStops(clampSpectrogramMinStop(minIndex, nextMax), nextMax)
  }
}

const stepSensitivity = step =>
  save({
    midpointDb: normalizeSpectrogramMidpointDb(
      settings.value.midpointDb - step * SPECTROGRAM_MIDPOINT_STEP_DB
    ),
  })

const stepColor = step => {
  const currentIndex = SPECTROGRAM_THEMES.findIndex(theme => theme.id === settings.value.theme)
  const nextIndex = Math.max(0, Math.min(SPECTROGRAM_THEMES.length - 1, currentIndex + step))
  const nextTheme = SPECTROGRAM_THEMES[nextIndex]
  if (nextIndex !== currentIndex) save({ theme: nextTheme.id })
  focusControl(swatchElements.get(nextTheme.id))
}

// Pointer changes focus the moved handle so the keyboard can take over.
const handleFrequencyChange = ([minHz, maxHz], handleIndex) => {
  save({ minHz, maxHz })
  focusControl(sliderHandles(frequencySlider.value)[handleIndex])
}

const handleSensitivityChange = value => {
  save({ midpointDb: -value })
  focusControl(sliderHandles(sensitivitySlider.value)[0])
}

const handleKeydown = event => {
  if (globalShortcutBindings.some(binding => binding.matches(event))) return

  // Editor hotkeys act on non-input targets and the modal's focus trap releases on Escape,
  // so no key may leave the drawer.
  event.stopPropagation()
  keyboardActive.value = true

  if (event.ctrlKey || event.altKey || event.metaKey) return

  const target = event.target
  switch (event.key) {
    case 'Tab': {
      event.preventDefault()
      const controls = rows().flatMap(row => row.controls)
      const currentIndex = isSwatch(target) ? 0 : controls.indexOf(target)
      cycleFocus(controls, currentIndex, event.shiftKey ? -1 : 1)
      return
    }
    case 'ArrowUp':
    case 'ArrowDown':
      event.preventDefault()
      cycleFocus(
        rows().map(row => row.controls[0]),
        rowIndexOf(target),
        event.key === 'ArrowDown' ? 1 : -1
      )
      return
    case 'ArrowLeft':
    case 'ArrowRight': {
      const step = event.key === 'ArrowRight' ? 1 : -1
      const frequencyHandleIndex = sliderHandles(frequencySlider.value).indexOf(target)
      if (isSwatch(target)) stepColor(step)
      else if (frequencyHandleIndex >= 0) stepFrequency(frequencyHandleIndex, step)
      else if (sliderHandles(sensitivitySlider.value).includes(target)) stepSensitivity(step)
      else return
      event.preventDefault()
      return
    }
    case 'Escape': {
      event.preventDefault()
      const [bottomHandle, topHandle] = sliderHandles(frequencySlider.value)
      if (target === topHandle) focusControl(bottomHandle)
      else emit('close')
    }
  }
}

const handleMouseDown = event => {
  const rowLabel = event.target.closest('[data-row]')
  if (rowLabel) {
    event.preventDefault()
    focusControl(rows()[Number(rowLabel.dataset.row)].controls[0])
    return
  }
  if (event.target.closest('input, button, [role="slider"]')) return

  // Clicks on empty parts of the drawer keep keyboard focus in it instead of dropping it.
  event.preventDefault()
  if (!drawerElement.value?.contains(document.activeElement)) {
    focusControl(lastFocusedControl ?? checkedSwatch())
  }
}

// WebKit on macOS doesn't focus form controls on click, so focus the clicked control explicitly.
const handleClick = event => {
  const control = event.target.closest('input, button, [role="slider"]')
  if (control && document.activeElement !== control) focusControl(control)
}

const handleFocusIn = event => {
  const row = rows()[rowIndexOf(event.target)]
  if (row) lastFocusedControl = event.target
  activeRow.value = row?.name ?? null
}

const handleFocusOut = event => {
  if (!drawerElement.value?.contains(event.relatedTarget)) activeRow.value = null
}

// Some editor controls cancel pointerdown, which keeps the browser from moving focus out.
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
  @apply grid items-center px-3 py-2 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900;
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

.slider {
  @apply cursor-pointer;
}

.slider :deep(.vue-slider-rail) {
  @apply bg-neutral-300 dark:bg-neutral-600;
}

.slider :deep(.vue-slider-process) {
  @apply bg-hoa-1100;
}

.slider :deep(.vue-slider-mark-step) {
  @apply hidden;
}

.slider :deep(.vue-slider-mark-label) {
  @apply mt-2 text-[10px] leading-3 text-neutral-500 dark:text-neutral-400 select-none;
}

.slider :deep(.vue-slider-dot) {
  outline: none;
}

.slider :deep(.vue-slider-dot-handle) {
  @apply border-2 border-solid border-hoa-1100 bg-white shadow;
}

.slider :deep(.vue-slider-dot:focus-visible .vue-slider-dot-handle),
.drawer[data-keyboard-active] .slider :deep(.vue-slider-dot:focus .vue-slider-dot-handle) {
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
