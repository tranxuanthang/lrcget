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

const props = defineProps({
  filePath: { type: String, default: null },
  startMs: { type: Number, default: 0 },
  endMs: { type: Number, default: 0 },
  height: { type: Number, default: 96 },
  theme: { type: String, default: DEFAULT_SPECTROGRAM_THEME_ID },
})

const canvasEl = ref(null)
const containerEl = ref(null)
const isLoading = ref(false)
const errorMessage = ref(null)

const emit = defineEmits(['seek'])

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

const FFT_SIZE = 1024
const HOP = 128
const DYNAMIC_RANGE_DB = 70

// Log-frequency display range. Covers the speech-relevant band (fundamental
// + first three formants) so it fills the panel vertically rather than being
// squashed into the bottom ~5% the way a linear Y axis does.
const DISPLAY_MIN_HZ = 50
const DISPLAY_MAX_HZ = 8000

const computeSpectrogram = (samples, sampleRate) => {
  const win = new Float32Array(FFT_SIZE)
  for (let i = 0; i < FFT_SIZE; i++) {
    win[i] = 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (FFT_SIZE - 1))
  }

  const totalFrames = Math.max(1, Math.floor((samples.length - FFT_SIZE) / HOP) + 1)
  const binCount = FFT_SIZE / 2

  // fft.js radix-4 real-input transform. realTransform() fills the left half of
  // `spectrum` (the non-redundant bins 0..binCount-1) with interleaved [re, im]
  // pairs — exactly the bins we render — so completeSpectrum() is unnecessary.
  const fft = new FFT(FFT_SIZE)
  const input = new Float32Array(FFT_SIZE)
  const spectrum = fft.createComplexArray()

  const frames = new Array(totalFrames)

  for (let frame = 0; frame < totalFrames; frame++) {
    const offset = frame * HOP
    for (let i = 0; i < FFT_SIZE; i++) {
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
  return { frames, binCount, sampleRate }
}

const renderToCanvas = (canvas, frames, binCount, sampleRate) => {
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

  // Peak-relative normalization: take the loudest bin across the slice as the
  // ceiling, display DYNAMIC_RANGE_DB below that. Local min/max wasted the
  // contrast budget on the noise floor; peak-relative keeps contrast where
  // speech actually lives.
  let peakDb = -Infinity
  for (let f = 0; f < frames.length; f++) {
    for (let b = 0; b < binCount; b++) {
      if (frames[f][b] > peakDb) peakDb = frames[f][b]
    }
  }
  if (!Number.isFinite(peakDb)) peakDb = 0
  const floorDb = peakDb - DYNAMIC_RANGE_DB

  // Precompute the bin index for each Y pixel using a log-frequency mapping.
  // Pixel 0 sits at the top (max freq), pxHeight-1 at the bottom (min freq).
  const effectiveSampleRate = sampleRate > 0 ? sampleRate : 44100
  const nyquistHz = effectiveSampleRate / 2
  const logMin = Math.log(DISPLAY_MIN_HZ)
  const logMax = Math.log(Math.min(DISPLAY_MAX_HZ, nyquistHz))
  const logRange = logMax - logMin
  const yToBin = new Int32Array(pxHeight)
  for (let y = 0; y < pxHeight; y++) {
    const yNorm = pxHeight > 1 ? (pxHeight - 1 - y) / (pxHeight - 1) : 0
    const freqHz = Math.exp(logMin + yNorm * logRange)
    const binFloat = (freqHz * FFT_SIZE) / effectiveSampleRate
    yToBin[y] = Math.max(0, Math.min(binCount - 1, Math.round(binFloat)))
  }

  for (let x = 0; x < pxWidth; x++) {
    const frameIdx = Math.min(frames.length - 1, Math.floor((x / pxWidth) * frames.length))
    const frame = frames[frameIdx]
    for (let y = 0; y < pxHeight; y++) {
      const db = frame[yToBin[y]]
      const t = Math.max(0, Math.min(1, (db - floorDb) / DYNAMIC_RANGE_DB))
      const lutIdx = Math.round(t * (SPECTROGRAM_LUT_SIZE - 1)) * 3
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
    renderToCanvas(canvasEl.value, cached.frames, cached.binCount, cached.sampleRate)
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
    if (!samples || samples.length < FFT_SIZE) {
      errorMessage.value = 'Audio slice too short'
      return
    }
    const result = computeSpectrogram(samples, response.sampleRate)

    if (cache.size >= CACHE_MAX) {
      cache.delete(cache.keys().next().value)
    }
    cache.set(key, result)

    if (activeRequestKey !== key) return
    renderToCanvas(canvasEl.value, result.frames, result.binCount, result.sampleRate)
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

// FFT frames are theme-independent, so a theme switch recolours the cached
// result in place instead of re-fetching audio and recomputing the transform.
watch(
  () => props.theme,
  () => {
    const cached = cacheKey.value ? cache.get(cacheKey.value) : null
    if (cached) renderToCanvas(canvasEl.value, cached.frames, cached.binCount, cached.sampleRate)
  }
)

let resizeObserver = null
onMounted(() => {
  refresh()
  if (canvasEl.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      const cached = cacheKey.value ? cache.get(cacheKey.value) : null
      if (cached) renderToCanvas(canvasEl.value, cached.frames, cached.binCount, cached.sampleRate)
    })
    resizeObserver.observe(canvasEl.value)
  }
})

onUnmounted(() => {
  if (refreshTimer) clearTimeout(refreshTimer)
  if (resizeObserver) resizeObserver.disconnect()
})
</script>
