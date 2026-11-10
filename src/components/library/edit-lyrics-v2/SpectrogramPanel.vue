<template>
  <div
    class="relative w-full h-24 overflow-hidden rounded border border-neutral-300 dark:border-neutral-600 bg-black cursor-pointer"
  >
    <canvas ref="canvasEl" class="absolute inset-0 w-full h-full block" />

    <div
      v-if="errorMessage"
      class="absolute inset-0 flex items-center justify-center text-[10px] text-neutral-300 dark:text-neutral-400 italic px-2 text-center bg-black/60"
    >
      {{ errorMessage }}
    </div>

    <div
      v-else-if="isLoading"
      class="absolute inset-0 flex items-center justify-center text-[10px] text-neutral-300 italic bg-black/40"
    >
      Computing spectrogram…
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import _debounce from 'lodash/debounce'
import { useSpectrogramSettings } from '@/composables/spectrogram-settings.js'
import { SPECTROGRAM_LUT_SIZE, getSpectrogramLut } from '@/utils/spectrogram-themes.js'
import {
  buildLevelToLutOffset,
  buildRowBinMap,
  computeSpectrogram,
  findPeakDb,
  sampleColumn,
  spectrogramFftSize,
} from '@/utils/spectrogram-render.js'
import { displayedSpectrogramRange } from '@/utils/spectrogram-settings.js'

const props = defineProps({
  filePath: { type: String, default: null },
  startMs: { type: Number, default: 0 },
  endMs: { type: Number, default: 0 },
})

const emit = defineEmits(['sample-rate'])

const { settings } = useSpectrogramSettings()

const canvasEl = ref(null)
const isLoading = ref(false)
const errorMessage = ref(null)

const cacheKey = computed(() =>
  props.filePath ? `${props.filePath}|${props.startMs}|${props.endMs}` : null
)

const isValidRange = computed(
  () =>
    props.filePath &&
    Number.isFinite(props.startMs) &&
    Number.isFinite(props.endMs) &&
    props.endMs > props.startMs &&
    props.endMs - props.startMs < 5 * 60 * 1000
)

// Least-recently-used FFT results per audio slice, so revisiting a line skips decoding.
const cache = new Map()
const CACHE_MAX = 50

const DYNAMIC_RANGE_DB = 70
const LEVEL_RESOLUTION = 1024

const renderToCanvas = (canvas, { frames, binCount, fftSize, sampleRate }) => {
  if (!canvas || frames.length === 0) return

  const { theme, minHz, maxHz, midpointDb } = settings.value
  const lut = getSpectrogramLut(theme)

  const cssWidth = canvas.clientWidth
  const cssHeight = canvas.clientHeight
  const dpr = window.devicePixelRatio || 1
  const pxWidth = Math.max(1, Math.floor(cssWidth * dpr))
  const pxHeight = Math.max(1, Math.floor(cssHeight * dpr))
  canvas.width = pxWidth
  canvas.height = pxHeight

  const ctx = canvas.getContext('2d')
  const imgData = ctx.createImageData(pxWidth, pxHeight)
  const data = imgData.data

  const range = displayedSpectrogramRange(minHz, maxHz, sampleRate)
  const rowMap = buildRowBinMap({
    rows: pxHeight,
    minHz: range.minHz,
    maxHz: range.maxHz,
    binHz: sampleRate / fftSize,
    binCount,
  })
  const levelToLutOffset = buildLevelToLutOffset({
    midpointDb,
    dynamicRangeDb: DYNAMIC_RANGE_DB,
    lutSize: SPECTROGRAM_LUT_SIZE,
    resolution: LEVEL_RESOLUTION,
  })

  // Only displayed bins set the peak, so loud energy outside the panel can't dim what is shown.
  let peakDb = findPeakDb(frames, rowMap.lowestBin, rowMap.highestBin)
  if (!Number.isFinite(peakDb)) peakDb = 0
  const floorDb = peakDb - DYNAMIC_RANGE_DB

  const column = new Float32Array(pxHeight)
  let sampledFrameIdx = -1
  for (let x = 0; x < pxWidth; x++) {
    const frameIdx = Math.min(frames.length - 1, Math.floor((x / pxWidth) * frames.length))
    if (frameIdx !== sampledFrameIdx) {
      sampleColumn(frames[frameIdx], rowMap, column)
      sampledFrameIdx = frameIdx
    }
    for (let y = 0; y < pxHeight; y++) {
      const level = Math.max(0, Math.min(1, (column[y] - floorDb) / DYNAMIC_RANGE_DB))
      const lutIdx = levelToLutOffset[Math.round(level * (LEVEL_RESOLUTION - 1))]
      const offset = (y * pxWidth + x) * 4
      data[offset] = lut[lutIdx]
      data[offset + 1] = lut[lutIdx + 1]
      data[offset + 2] = lut[lutIdx + 2]
      data[offset + 3] = 255
    }
  }
  ctx.putImageData(imgData, 0, 0)
}

let activeRequestKey = null

const drawLoadedSpectrogram = spectrogram => {
  renderToCanvas(canvasEl.value, spectrogram)
  emit('sample-rate', spectrogram.sampleRate)
}

// Display settings and size don't change the FFT frames, so these redraw from the cache.
const redraw = () => {
  const cached = cache.get(cacheKey.value)
  if (cached) renderToCanvas(canvasEl.value, cached)
}

const refresh = async () => {
  errorMessage.value = null

  if (!isValidRange.value) {
    if (canvasEl.value) {
      const ctx = canvasEl.value.getContext('2d')
      ctx.clearRect(0, 0, canvasEl.value.width, canvasEl.value.height)
    }
    return
  }

  const key = cacheKey.value
  activeRequestKey = key

  const cached = cache.get(key)
  if (cached) {
    cache.delete(key)
    cache.set(key, cached)
    drawLoadedSpectrogram(cached)
    return
  }

  isLoading.value = true
  try {
    const { samples, sampleRate } = await invoke('get_audio_slice', {
      filePath: props.filePath,
      startMs: Math.round(props.startMs),
      endMs: Math.round(props.endMs),
    })
    if (activeRequestKey !== key) return

    if (!samples || samples.length < spectrogramFftSize(sampleRate)) {
      errorMessage.value = 'Audio slice too short'
      return
    }
    const result = computeSpectrogram(samples, sampleRate)

    if (cache.size >= CACHE_MAX) {
      cache.delete(cache.keys().next().value)
    }
    cache.set(key, result)
    drawLoadedSpectrogram(result)
  } catch (err) {
    if (activeRequestKey === key) {
      errorMessage.value = typeof err === 'string' ? err : err?.message || 'Failed to load audio'
    }
  } finally {
    if (activeRequestKey === key) {
      isLoading.value = false
    }
  }
}

const scheduleRefresh = _debounce(refresh, 120)

watch(() => [props.filePath, props.startMs, props.endMs], scheduleRefresh)

watch(settings, redraw)

let resizeObserver = null
onMounted(() => {
  refresh()
  resizeObserver = new ResizeObserver(redraw)
  resizeObserver.observe(canvasEl.value)
})

onUnmounted(() => {
  scheduleRefresh.cancel()
  resizeObserver.disconnect()
})
</script>
