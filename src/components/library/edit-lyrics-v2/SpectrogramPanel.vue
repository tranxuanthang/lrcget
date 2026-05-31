<template>
  <div
    class="relative w-full overflow-hidden rounded border border-neutral-300 dark:border-neutral-600 bg-black"
    :style="{ height: `${height}px` }"
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

const props = defineProps({
  filePath: { type: String, default: null },
  startMs: { type: Number, default: 0 },
  endMs: { type: Number, default: 0 },
  height: { type: Number, default: 96 },
})

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

// In-place radix-2 Cooley-Tukey FFT. Length must be a power of two.
const fftInPlace = (real, imag) => {
  const n = real.length
  let j = 0
  for (let i = 1; i < n; i++) {
    let bit = n >> 1
    while (j & bit) {
      j ^= bit
      bit >>= 1
    }
    j ^= bit
    if (i < j) {
      let t = real[i]
      real[i] = real[j]
      real[j] = t
      t = imag[i]
      imag[i] = imag[j]
      imag[j] = t
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const half = len >> 1
    const step = (-2 * Math.PI) / len
    for (let i = 0; i < n; i += len) {
      for (let k = 0; k < half; k++) {
        const angle = step * k
        const cos = Math.cos(angle)
        const sin = Math.sin(angle)
        const idx = i + k + half
        const tr = real[idx] * cos - imag[idx] * sin
        const ti = real[idx] * sin + imag[idx] * cos
        real[idx] = real[i + k] - tr
        imag[idx] = imag[i + k] - ti
        real[i + k] = real[i + k] + tr
        imag[i + k] = imag[i + k] + ti
      }
    }
  }
}

const computeSpectrogram = (samples, sampleRate) => {
  const win = new Float32Array(FFT_SIZE)
  for (let i = 0; i < FFT_SIZE; i++) {
    win[i] = 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (FFT_SIZE - 1))
  }

  const totalFrames = Math.max(1, Math.floor((samples.length - FFT_SIZE) / HOP) + 1)
  const binCount = FFT_SIZE / 2

  const frames = new Array(totalFrames)
  const real = new Float32Array(FFT_SIZE)
  const imag = new Float32Array(FFT_SIZE)

  for (let frame = 0; frame < totalFrames; frame++) {
    const offset = frame * HOP
    for (let i = 0; i < FFT_SIZE; i++) {
      real[i] = (samples[offset + i] || 0) * win[i]
      imag[i] = 0
    }
    fftInPlace(real, imag)
    const mags = new Float32Array(binCount)
    for (let b = 0; b < binCount; b++) {
      const mag = Math.sqrt(real[b] * real[b] + imag[b] * imag[b])
      mags[b] = mag > 1e-9 ? 20 * Math.log10(mag) : -180
    }
    frames[frame] = mags
  }
  return { frames, binCount, sampleRate }
}

// 5-stop inferno approximation (black → purple → red → orange → pale yellow).
// Higher visual contrast in the mid-range than viridis, which matters for
// speech detail. Cheap enough to recompute per pixel.
const inferno = t => {
  const stops = [
    [0.0, 0, 0, 4],
    [0.25, 101, 21, 110],
    [0.5, 212, 72, 66],
    [0.75, 250, 193, 39],
    [1.0, 252, 255, 164],
  ]
  if (t <= 0) return [stops[0][1], stops[0][2], stops[0][3]]
  if (t >= 1) return [stops[4][1], stops[4][2], stops[4][3]]
  for (let i = 1; i < stops.length; i++) {
    if (t <= stops[i][0]) {
      const lo = stops[i - 1]
      const hi = stops[i]
      const local = (t - lo[0]) / (hi[0] - lo[0])
      return [
        Math.round(lo[1] + (hi[1] - lo[1]) * local),
        Math.round(lo[2] + (hi[2] - lo[2]) * local),
        Math.round(lo[3] + (hi[3] - lo[3]) * local),
      ]
    }
  }
  return [stops[4][1], stops[4][2], stops[4][3]]
}

const renderToCanvas = (canvas, frames, binCount, sampleRate) => {
  if (!canvas || frames.length === 0) return

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
      const [r, g, b] = inferno(t)
      const offset = (y * pxWidth + x) * 4
      data[offset] = r
      data[offset + 1] = g
      data[offset + 2] = b
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
