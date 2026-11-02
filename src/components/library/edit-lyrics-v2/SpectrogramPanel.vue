<template>
  <div
    ref="containerEl"
    class="relative w-full overflow-hidden rounded border border-neutral-300 dark:border-neutral-600 bg-black cursor-pointer"
    :style="{ height: `${height}px` }"
    @click="handleSeekClick"
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
import FFT from 'fft.js'
import {
  DEFAULT_SPECTROGRAM_THEME_ID,
  SPECTROGRAM_LUT_SIZE,
  getSpectrogramLut,
} from '@/utils/spectrogram-themes.js'
import {
  buildLevelToLutOffset,
  buildRowBinMap,
  findPeakDb,
  sampleColumn,
  spectrogramFftSize,
} from '@/utils/spectrogram-render.js'
import {
  DEFAULT_SPECTROGRAM_MAX_HZ,
  DEFAULT_SPECTROGRAM_MIDPOINT_DB,
  DEFAULT_SPECTROGRAM_MIN_HZ,
  displayedSpectrogramRange,
} from '@/utils/spectrogram-settings.js'

const props = defineProps({
  filePath: { type: String, default: null },
  startMs: { type: Number, default: 0 },
  endMs: { type: Number, default: 0 },
  height: { type: Number, default: 96 },
  theme: { type: String, default: DEFAULT_SPECTROGRAM_THEME_ID },
  minHz: { type: Number, default: DEFAULT_SPECTROGRAM_MIN_HZ },
  maxHz: { type: Number, default: DEFAULT_SPECTROGRAM_MAX_HZ },
  midpointDb: { type: Number, default: DEFAULT_SPECTROGRAM_MIDPOINT_DB },
})

const canvasEl = ref(null)
const containerEl = ref(null)
const isLoading = ref(false)
const errorMessage = ref(null)

const emit = defineEmits(['seek', 'sample-rate'])

const handleSeekClick = event => {
  if (errorMessage.value) return
  if (!isValidRange.value) return
  const target = containerEl.value
  if (!(target instanceof HTMLElement)) return
  const rect = target.getBoundingClientRect()
  if (rect.width <= 0) return
  const fraction = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width))
  const timeMs = props.startMs + fraction * (props.endMs - props.startMs)
  emit('seek', timeMs / 1000)
}

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

// Module-scope result cache: same line revisited = no recompute.
// Bounded to ~50 entries to avoid unbounded memory growth.
const cache = new Map()
const CACHE_MAX = 50

const HOPS_PER_WINDOW = 8
const DYNAMIC_RANGE_DB = 70
const LEVEL_RESOLUTION = 1024

const computeSpectrogram = (samples, sampleRate) => {
  const fftSize = spectrogramFftSize(sampleRate)
  const hop = fftSize / HOPS_PER_WINDOW
  const win = new Float32Array(fftSize)
  for (let i = 0; i < fftSize; i++) {
    win[i] = 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (fftSize - 1))
  }

  const totalFrames = Math.max(1, Math.floor((samples.length - fftSize) / hop) + 1)
  const binCount = fftSize / 2

  // fft.js radix-4 real-input transform. realTransform() fills the left half of
  // `spectrum` (the non-redundant bins 0..binCount-1) with interleaved [re, im]
  // pairs — exactly the bins we render
  const fft = new FFT(fftSize)
  const input = new Float32Array(fftSize)
  const spectrum = fft.createComplexArray()

  const frames = new Array(totalFrames)

  for (let frame = 0; frame < totalFrames; frame++) {
    const offset = frame * hop
    for (let i = 0; i < fftSize; i++) {
      input[i] = (samples[offset + i] || 0) * win[i]
    }
    fft.realTransform(spectrum, input)
    const mags = new Float32Array(binCount)
    for (let b = 0; b < binCount; b++) {
      const re = spectrum[2 * b]
      const im = spectrum[2 * b + 1]
      const mag = Math.sqrt(re * re + im * im)
      mags[b] = mag > 1e-9 ? 20 * Math.log10(mag) : -180
    }
    frames[frame] = mags
  }
  return { frames, binCount, fftSize, sampleRate }
}

const renderToCanvas = (canvas, { frames, binCount, fftSize, sampleRate }) => {
  if (!canvas || frames.length === 0) return

  const lut = getSpectrogramLut(props.theme)

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

  const range = displayedSpectrogramRange(props.minHz, props.maxHz, sampleRate)
  const rowMap = buildRowBinMap({
    rows: pxHeight,
    minHz: range.minHz,
    maxHz: range.maxHz,
    binHz: sampleRate / fftSize,
    binCount,
  })
  const levelToLutOffset = buildLevelToLutOffset({
    midpointDb: props.midpointDb,
    dynamicRangeDb: DYNAMIC_RANGE_DB,
    lutSize: SPECTROGRAM_LUT_SIZE,
    resolution: LEVEL_RESOLUTION,
  })

  // Peak-relative normalization: the loudest displayed bin gets the top color and
  // DYNAMIC_RANGE_DB below it is black. Only displayed bins count, so loud energy
  // above or below the panel can't dim what is shown.
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
    drawLoadedSpectrogram(cached)
    return
  }

  isLoading.value = true
  try {
    const response = await invoke('get_audio_slice', {
      filePath: props.filePath,
      startMs: Math.round(props.startMs),
      endMs: Math.round(props.endMs),
    })
    if (activeRequestKey !== key) return

    const samples = response.samples
    const sampleRate = response.sampleRate > 0 ? response.sampleRate : 44100
    if (!samples || samples.length < spectrogramFftSize(sampleRate)) {
      errorMessage.value = 'Audio slice too short'
      return
    }
    const result = computeSpectrogram(samples, sampleRate)

    if (cache.size >= CACHE_MAX) {
      cache.delete(cache.keys().next().value)
    }
    cache.set(key, result)

    if (activeRequestKey !== key) return
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

let refreshTimer = null
const scheduleRefresh = () => {
  if (refreshTimer) clearTimeout(refreshTimer)
  refreshTimer = setTimeout(refresh, 120)
}

watch(
  () => [props.filePath, props.startMs, props.endMs],
  () => scheduleRefresh(),
  { immediate: false }
)

// FFT frames don't depend on the display settings, so a theme, frequency range or
// midpoint change redraws the cached result in place instead of re-fetching audio
// and recomputing the transform.
watch(
  () => [props.theme, props.minHz, props.maxHz, props.midpointDb],
  () => {
    const cached = cacheKey.value ? cache.get(cacheKey.value) : null
    if (cached) renderToCanvas(canvasEl.value, cached)
  }
)

let resizeObserver = null
onMounted(() => {
  refresh()
  if (canvasEl.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      const cached = cacheKey.value ? cache.get(cacheKey.value) : null
      if (cached) renderToCanvas(canvasEl.value, cached)
    })
    resizeObserver.observe(canvasEl.value)
  }
})

onUnmounted(() => {
  if (refreshTimer) clearTimeout(refreshTimer)
  if (resizeObserver) resizeObserver.disconnect()
})
</script>
