<template>
  <transition name="slide-fade" mode="out-in">
    <div v-if="syncedLines.length > 0 && duration != null && progress != null" class="flex flex-col gap-1 relative">
      <transition name="slide-fade" mode="out-in">
        <div v-if="expanded" class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-brave-95 dark:bg-brave-10 border-t border-brave-90/50 dark:border-brave-10/50 overflow-hidden">
          <div class="relative h-full">
            <div class="flex justify-center items-center h-6 w-full relative z-10">
              <button class="text-xl text-brave-30 dark:text-brave-80 w-full flex justify-center" type="button" @click="expand"><DragHorizontal /></button>
            </div>

            <div id="full-lyrics-container" class="h-full text-center transition" :style="{ transform: fullViewTransform }">
              <p
                v-for="(line, index) in syncedLines"
                :key="index"
                class="transition"
                :class="{
                  'font-bold text-brave-50 dark:text-brave-95': currentIndex === index,
                  'text-brave-50 hover:text-brave-40 hover:cursor-pointer dark:text-brave-80 dark:hover:text-brave-90': currentIndex !== index
                }"
                @click="onLineClick(line)"
              >
                <template v-if="hasWordSync(syncedLines, index)">
                  <span
                    v-for="(word, wordIndex) in getLineWords(syncedLines, index)"
                    :key="wordIndex"
                    class="whitespace-pre-wrap"
                    :class="{ 'text-yellow-500 dark:text-yellow-400': currentIndex === index && isWordPlaying(syncedLines, index, wordIndex, progress * 1000) }"
                  >{{ word.text }}</span>
                </template>
                <template v-else>{{ line.text }}</template>
              </p>
            </div>

            <button
              class="z-10 absolute bottom-2 right-2 flex items-center gap-1 px-3 py-1 rounded text-xs font-bold bg-brave-90 text-brave-20 dark:bg-brave-15 dark:text-brave-95 hover:bg-brave-80 dark:hover:bg-brave-20 shadow"
              type="button"
              @click.stop="onCopy"
              :aria-label="copied ? 'Copied' : 'Copy'"
            >
              <ContentCopy class="w-4 h-4" />
              <span>{{ copied ? 'Copied' : 'Copy' }}</span>
            </button>

            <div class="absolute top-0 left-0 w-full h-10 bg-gradient-to-b from-brave-95 dark:from-brave-10 pointer-events-none"></div>
            <div class="absolute bottom-0 left-0 w-full h-10 bg-gradient-to-t from-brave-95 dark:from-brave-10 pointer-events-none"></div>
          </div>
        </div>
      </transition>

      <div
        class="mini-viewer transition cursor-pointer top-0 left-0 w-full h-12 bg-brave-95 dark:bg-brave-10 border-t border-brave-90/50 dark:border-brave-10/50 flex flex-col"
        :class="{ 'invisible opacity-0': expanded }"
        @click="expand"
      >
        <div class="flex justify-center items-center h-4 w-full">
          <button class="text-xl text-brave-30 dark:text-brave-95 w-full flex justify-center" type="button"><DragHorizontal /></button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div class="flex w-full justify-center items-center text-brave-30 dark:text-brave-95 text-sm grow font-bold" :key="currentLyrics">
            <template v-if="hasWordSync(syncedLines, currentIndex)">
              <span
                v-for="(word, wordIndex) in getLineWords(syncedLines, currentIndex)"
                :key="wordIndex"
                class="whitespace-pre-wrap"
                :class="{ 'text-yellow-500 dark:text-yellow-400': isWordPlaying(syncedLines, currentIndex, wordIndex, progress * 1000) }"
              >{{ word.text }}</span>
            </template>
            <template v-else>{{ currentLyrics }}</template>
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
import { computed } from '@vue/reactivity'
import { parseLyricsfile } from '@/utils/lyricsfile.js'

const props = defineProps(['duration', 'progress', 'lyricsfile'])
const emit = defineEmits(['lyricsClicked'])

const currentIndex = ref(null)
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

const currentLyrics = computed(() => {
  if (currentIndex.value === null || currentIndex.value < 0 || currentIndex.value >= syncedLines.value.length) {
    return '…'
  }
  const line = syncedLines.value[currentIndex.value]
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

const findCurrentLineIndex = (progressMs) => {
  const lines = syncedLines.value
  if (!lines || lines.length === 0) {
    return -1
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const nextLine = lines[i + 1]
    const lineStart = line.start_ms
    const lineEnd = nextLine ? nextLine.start_ms : Infinity

    if (progressMs >= lineStart && progressMs < lineEnd) {
      return i
    }
  }

  return -1
}

const expand = () => {
  expanded.value = !expanded.value
  nextTick(() => {
    updateCurrentLineElementOffset(currentIndex.value)
  })
}

const updateCurrentLineElementOffset = (newCurrentIndex) => {
  if (newCurrentIndex === null) {
    newCurrentIndex = -1
  }

  if (newCurrentIndex === -1) {
    currentLineElementOffset.value = null
    return
  }

  const fullLyricsContainerEl = document.getElementById('full-lyrics-container')
  if (fullLyricsContainerEl) {
    const currentLyricsLine = Array.from(fullLyricsContainerEl.children)[newCurrentIndex]
    currentLineElementOffset.value = currentLyricsLine.offsetTop
  }
}

const fullViewTransform = computed(() => {
  if (!currentLineElementOffset.value) {
    return `translateY(calc(50% - 2.5em))`
  }

  return `translateY(calc(50% - 2.5em - ${currentLineElementOffset.value}px))`
})

const onLineClick = (line) => {
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

watch(() => props.progress, (newProgress) => {
  if (!syncedLines.value || syncedLines.value.length === 0) {
    currentIndex.value = -1
    return
  }

  const progressMs = newProgress * 1000
  currentIndex.value = findCurrentLineIndex(progressMs)
}, { immediate: true })

watch(currentIndex, (newCurrentIndex) => {
  updateCurrentLineElementOffset(newCurrentIndex)
})
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
