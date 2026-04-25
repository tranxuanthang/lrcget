<template>
  <transition name="slide-fade" mode="out-in">
    <div
      v-if="syncedLines.length > 0 && duration != null && progress != null"
      class="flex flex-col gap-1 relative"
    >
      <transition name="slide-fade" mode="out-in">
        <div
          v-if="expanded"
          class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-neutral-50 dark:bg-neutral-900 border-t border-neutral-200/50 dark:border-neutral-800/50 overflow-hidden"
        >
          <div class="relative h-full">
            <div class="flex justify-center items-center h-6 w-full relative z-10">
              <button
                class="text-xl text-neutral-800 dark:text-neutral-400 w-full flex justify-center"
                type="button"
                @click="expand"
              >
                <DragHorizontal />
              </button>
            </div>

            <div
              id="full-lyrics-container"
              class="h-full text-center transition"
              :style="{ transform: fullViewTransform }"
            >
              <p
                v-for="(line, index) in syncedLines"
                :key="index"
                class="transition"
                :class="{
                  'font-bold text-neutral-800 dark:text-neutral-200': isLinePlaying(line, index),
                  'text-neutral-800/70 hover:text-neutral-800/80 hover:cursor-pointer dark:text-neutral-400/70 dark:hover:text-neutral-300/80':
                    !isLinePlaying(line, index),
                }"
                @click="onLineClick(line)"
              >
                <template v-if="hasWordSync(syncedLines, index)">
                  <span
                    v-for="(word, wordIndex) in getLineWords(syncedLines, index)"
                    :key="wordIndex"
                    class="whitespace-pre-wrap"
                    :class="{
                      'text-yellow-500 dark:text-yellow-400':
                        isLinePlaying(line, index) &&
                        isWordPlaying(syncedLines, index, wordIndex, progressMs),
                    }"
                    >{{ word.text }}</span
                  >
                </template>
                <template v-else>
                  {{ line.text }}
                </template>
              </p>
            </div>

            <button
              class="z-10 absolute bottom-2 right-2 flex items-center gap-1 px-3 py-1 rounded text-xs font-bold bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700 shadow"
              type="button"
              :aria-label="copied ? 'Copied' : 'Copy'"
              @click.stop="onCopy"
            >
              <ContentCopy class="w-4 h-4" />
              <span>{{ copied ? 'Copied' : 'Copy' }}</span>
            </button>

            <div
              class="absolute top-0 left-0 w-full h-10 bg-gradient-to-b from-neutral-50 dark:from-neutral-900 pointer-events-none"
            />
            <div
              class="absolute bottom-0 left-0 w-full h-10 bg-gradient-to-t from-neutral-50 dark:from-neutral-900 pointer-events-none"
            />
          </div>
        </div>
      </transition>

      <div
        class="mini-viewer transition cursor-pointer top-0 left-0 w-full h-12 bg-neutral-50 dark:bg-neutral-900 border-t border-neutral-200/50 dark:border-neutral-800/50 flex flex-col"
        :class="{ 'invisible opacity-0': expanded }"
        @click="expand"
      >
        <div class="flex justify-center items-center h-4 w-full">
          <button
            class="text-xl text-neutral-800 dark:text-neutral-400 w-full flex justify-center"
            type="button"
          >
            <DragHorizontal />
          </button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div
            :key="currentLyrics"
            class="flex w-full justify-center items-center text-neutral-800 dark:text-neutral-200 text-sm grow font-bold"
          >
            <template v-if="hasWordSync(syncedLines, primaryPlayingLineIndex)">
              <span
                v-for="(word, wordIndex) in getLineWords(syncedLines, primaryPlayingLineIndex)"
                :key="wordIndex"
                class="whitespace-pre-wrap"
                :class="{
                  'text-yellow-500 dark:text-yellow-400': isWordPlaying(
                    syncedLines,
                    primaryPlayingLineIndex,
                    wordIndex,
                    progressMs
                  ),
                }"
                >{{ word.text }}</span
              >
            </template>
            <template v-else>
              {{ currentLyrics }}
            </template>
          </div>
        </transition>
      </div>
    </div>
  </transition>
</template>

<script setup>
import DragHorizontal from '~icons/mdi/drag-horizontal'
import ContentCopy from '~icons/mdi/content-copy'
import { ref, watch, nextTick } from 'vue'
import { computed } from 'vue'
import { parseLyricsfile } from '@/utils/lyricsfile.js'

const props = defineProps(['duration', 'progress', 'lyricsfile'])
const emit = defineEmits(['lyricsClicked'])

const expanded = ref(false)
const currentLineElementOffset = ref(null)
const copied = ref(false)

const parsedLyricsfile = computed(() => {
  if (!props.lyricsfile) {
    return null
  }
  return parseLyricsfile(props.lyricsfile)
})

const syncedLines = computed(() => {
  if (!parsedLyricsfile.value) {
    return []
  }
  return parsedLyricsfile.value.syncedLines || []
})

const progressMs = computed(() => (props.progress ?? 0) * 1000)

// Check if a line is currently playing based on its own time range
const isLinePlaying = (line, index) => {
  if (!line || !Number.isFinite(line.start_ms)) {
    return false
  }
  const lineStart = line.start_ms
  const lineEnd = Number.isFinite(line.end_ms)
    ? line.end_ms
    : (syncedLines.value[index + 1]?.start_ms ?? Infinity)
  return progressMs.value >= lineStart && progressMs.value < lineEnd
}

// For minimal view: find the primary playing line (prioritize by start time if multiple overlap)
const primaryPlayingLineIndex = computed(() => {
  const lines = syncedLines.value
  if (!lines || lines.length === 0) {
    return -1
  }

  // Find all lines that are currently playing
  const playingIndices = []
  for (let i = 0; i < lines.length; i++) {
    if (isLinePlaying(lines[i], i)) {
      playingIndices.push(i)
    }
  }

  if (playingIndices.length === 0) {
    // No line is playing, find the most recent past line
    for (let i = lines.length - 1; i >= 0; i--) {
      if (progressMs.value >= lines[i].start_ms) {
        return i
      }
    }
    return -1
  }

  // Return the line with the latest start time (most recently started)
  return playingIndices[playingIndices.length - 1]
})

const currentLyrics = computed(() => {
  const index = primaryPlayingLineIndex.value
  if (index === null || index < 0 || index >= syncedLines.value.length) {
    return '…'
  }
  const line = syncedLines.value[index]
  return line?.text || '…'
})

const getCurrentWordIndex = (line, currentTimeMs) => {
  if (!line?.words || !Array.isArray(line.words) || line.words.length === 0) {
    return -1
  }

  for (let i = 0; i < line.words.length; i++) {
    const word = line.words[i]
    const nextWord = line.words[i + 1]
    const wordStart = word.start_ms
    const wordEnd = nextWord ? nextWord.start_ms : Infinity

    if (currentTimeMs >= wordStart && currentTimeMs < wordEnd) {
      return i
    }
  }

  return -1
}

const hasWordSync = (lines, index) => {
  if (!lines || index === null || index < 0) {
    return false
  }
  const line = lines[index]
  return line && line.words && Array.isArray(line.words) && line.words.length > 0
}

const getLineWords = (lines, index) => {
  if (!lines || index === null || index < 0) {
    return []
  }
  const line = lines[index]
  if (!line || !line.words || !Array.isArray(line.words)) {
    return []
  }
  return line.words
}

const isWordPlaying = (lines, lineIndex, wordIndex, progressMs) => {
  if (!lines || lineIndex === null || lineIndex < 0) {
    return false
  }
  const line = lines[lineIndex]
  if (!line || !line.words || !Array.isArray(line.words)) {
    return false
  }
  return getCurrentWordIndex(line, progressMs) === wordIndex
}

// For expanded view scrolling: use the primary playing line for scroll position
const updateCurrentLineElementOffset = () => {
  const newCurrentIndex = primaryPlayingLineIndex.value

  if (newCurrentIndex === -1) {
    currentLineElementOffset.value = null
    return
  }

  const fullLyricsContainerEl = document.getElementById('full-lyrics-container')
  if (fullLyricsContainerEl) {
    const currentLyricsLine = Array.from(fullLyricsContainerEl.children)[newCurrentIndex]
    if (currentLyricsLine) {
      currentLineElementOffset.value = currentLyricsLine.offsetTop
    }
  }
}

const fullViewTransform = computed(() => {
  if (!currentLineElementOffset.value) {
    return `translateY(calc(50% - 2.5em))`
  }

  return `translateY(calc(50% - 2.5em - ${currentLineElementOffset.value}px))`
})

const expand = () => {
  expanded.value = !expanded.value
  nextTick(() => {
    updateCurrentLineElementOffset()
  })
}

const onLineClick = line => {
  emit('lyricsClicked', { timestamp: line.start_ms / 1000 })
}

const onCopy = async () => {
  try {
    const text = syncedLines.value.map(l => l.text).join('\n')
    if (!text) return
    await navigator.clipboard.writeText(text)
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch (e) {
    // swallow
  }
}

watch(
  () => props.progress,
  () => {
    updateCurrentLineElementOffset()
  },
  { immediate: true }
)
</script>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.05s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.05s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

.slide-fade-enter-from {
  transform: translateY(20px);
  opacity: 0;
}
</style>
