<template>
  <transition name="slide-fade" mode="out-in">
    <div v-if="lyrics && duration && progress" class="flex flex-col gap-1 relative">
      <transition name="slide-fade" mode="out-in">
        <div v-if="expanded" class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-brave-95 dark:bg-brave-10 border-t border-brave-90/50 dark:border-brave-10/50 overflow-hidden">
          <div class="relative h-full">
            <div class="flex justify-between items-center h-6 w-full relative z-10 px-2">
              <button class="text-xs text-brave-30 dark:text-brave-80 p-1" type="button" @click="toggleRomanized">
                {{ showRomanized ? 'Hide Romanized' : 'Show Romanized' }}
              </button>
              <button class="text-xl text-brave-30 dark:text-brave-80 flex justify-center" type="button" @click="expand"><DragHorizontal /></button>
              <div class="w-1/3"></div> <!-- Spacer for centering expand button -->
            </div>

            <div id="full-lyrics-container" class="h-full text-center transition overflow-y-auto" :style="{ transform: fullViewTransform }">
              <div
                v-for="(line, index) in parsedLyrics"
                :key="index"
                class="transition py-1"
                :class="{
                  'font-bold text-brave-50 dark:text-brave-95': currentIndex === index,
                  'text-brave-50 hover:text-brave-40 hover:cursor-pointer dark:text-brave-80 dark:hover:text-brave-90': currentIndex !== index
                }"
                @click="onLineClick(line)"
              >
                <div v-if="showRomanized && getRomanizedLine(line.content)" class="text-xs text-gray-400 dark:text-gray-500">
                  {{ getRomanizedLine(line.content) }}
                </div>
                <p>{{ line.content }}</p>
              </div>
            </div>

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
          <!-- Mini view romanized toggle could go here if desired, for now, it's controlled by the expanded view's toggle -->
          <button class="text-xl text-brave-30 dark:text-brave-95 w-full flex justify-center" type="button"><DragHorizontal /></button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div class="flex w-full justify-center items-center text-brave-30 dark:text-brave-95 text-sm grow px-1 text-center" :key="displayMiniLyrics">
            {{ displayMiniLyrics }}
          </div>
        </transition>
      </div>
    </div>
  </transition>
</template>

<script setup>
import { DragHorizontal } from 'mdue'
import { Lrc, Runner } from 'lrc-kit'
import { ref, onMounted, watch, nextTick, computed } from 'vue'
import { getRomanizedText, detectLanguage } from '../../utils/romanization.js'

const props = defineProps(['lyrics', 'duration', 'progress'])
const emit = defineEmits(['lyricsClicked'])

const runner = ref(null)
const parsedLyrics = ref(null)
const currentIndex = ref(null)
const currentRawLyrics = ref('…') // Stores the raw current lyric line
const expanded = ref(false)
const showRomanized = ref(false)
const currentLineElementOffset = ref(null)

const expand = () => {
  expanded.value = !expanded.value
  nextTick(() => {
    updateCurrentLineElementOffset(currentIndex.value)
  })
}

const toggleRomanized = () => {
  showRomanized.value = !showRomanized.value
  // Force re-evaluation of currentLineElementOffset if view changes
  nextTick(() => {
    updateCurrentLineElementOffset(currentIndex.value)
  })
}

const getRomanizedLine = (text) => {
  if (!text || typeof text !== 'string') return ''
  const lang = detectLanguage(text)
  if (lang !== 'unknown') {
    return getRomanizedText(text, lang)
  }
  return ''
}

const displayMiniLyrics = computed(() => {
  if (showRomanized.value && currentRawLyrics.value && currentRawLyrics.value !== '…') {
    const romanized = getRomanizedLine(currentRawLyrics.value)
    return romanized || currentRawLyrics.value // Show original if romanization is empty
  }
  return currentRawLyrics.value
})

const updateCurrentLineElementOffset = (newCurrentIndex) => {
  if (newCurrentIndex === null) {
    newCurrentIndex = -1
  }

  if (newCurrentIndex === -1 || !expanded.value) { // also check if expanded
    currentLineElementOffset.value = null
    return
  }

  const fullLyricsContainerEl = document.getElementById('full-lyrics-container')
  if (fullLyricsContainerEl && fullLyricsContainerEl.children.length > newCurrentIndex && newCurrentIndex >=0) {
    const currentLyricsLineDiv = fullLyricsContainerEl.children[newCurrentIndex]
    if (currentLyricsLineDiv) {
      currentLineElementOffset.value = currentLyricsLineDiv.offsetTop
    } else {
      currentLineElementOffset.value = null
    }
  } else {
    currentLineElementOffset.value = null
  }
}

const fullViewTransform = computed(() => {
  if (!currentLineElementOffset.value || !expanded.value) {
    // Default centering when no specific line is focused or not expanded
    // Use a fixed offset or calculate based on container height if possible
    return `translateY(calc(50% - 2.5em))` // Adjust 2.5em based on typical line height
  }
  // Adjust based on the current line's offset
  return `translateY(calc(50% - ${currentLineElementOffset.value}px - 1.25em))` // Adjust 1.25em to center the line
})

const onLineClick = (line) => {
  emit('lyricsClicked', line)
}

onMounted(() => {
  if (props.lyrics) {
    const parsed = Lrc.parse(props.lyrics)
    runner.value = new Runner(parsed)
    parsedLyrics.value = runner.value.getLyrics()
  }
})

watch(() => props.lyrics, (newLyrics) => {
  if (!newLyrics) {
    parsedLyrics.value = []
    runner.value = null
    return
  }
  const parsed = Lrc.parse(newLyrics)
  runner.value = new Runner(parsed)
  parsedLyrics.value = runner.value.getLyrics()
  // Reset current index and lyrics on new song
  currentIndex.value = null 
  currentRawLyrics.value = '…'
  updateCurrentLineElementOffset(-1)
})

watch(() => props.progress, (newProgress) => {
  if (!runner.value || !props.lyrics) {
    currentRawLyrics.value = '…' // Ensure "…" is shown if no runner or lyrics
    return
  }

  runner.value.timeUpdate(newProgress)
  let resultCurrentIndex = runner.value.curIndex()

  if (resultCurrentIndex === null || resultCurrentIndex < 0 || resultCurrentIndex >= parsedLyrics.value.length) {
    resultCurrentIndex = -1
  }
  
  if (currentIndex.value !== resultCurrentIndex) {
    currentIndex.value = resultCurrentIndex
    if (expanded.value) { // Only update offset if expanded
        nextTick(() => updateCurrentLineElementOffset(resultCurrentIndex));
    }
  }

  if (currentIndex.value === -1) {
    currentRawLyrics.value = '…'
    return
  }

  try {
    // Ensure parsedLyrics is populated and index is valid
    if (parsedLyrics.value && parsedLyrics.value.length > currentIndex.value && currentIndex.value !== -1) {
      const currentLyricsObj = parsedLyrics.value[currentIndex.value] // Use parsedLyrics directly
      currentRawLyrics.value = currentLyricsObj.content
    } else {
      currentRawLyrics.value = '…' // Fallback if something is wrong
    }
  } catch (error) {
    console.error("Error accessing lyric line:", error)
    currentRawLyrics.value = '…'
  }
})

watch(currentIndex, (newCurrentIndex, oldCurrentIndex) => {
    if (newCurrentIndex !== oldCurrentIndex && expanded.value) {
        nextTick(() => { // Ensure DOM has updated
            updateCurrentLineElementOffset(newCurrentIndex);
        });
    }
})

watch(expanded, (isExpanded) => {
  if (isExpanded) {
    nextTick(() => {
      updateCurrentLineElementOffset(currentIndex.value)
    })
  }
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

/* Ensure the full lyrics container allows for scrolling if content overflows */
#full-lyrics-container {
  overflow-y: auto; /* Changed from hidden to auto or scroll */
  /* padding for top and bottom fades */
  padding-top: 2.5rem; 
  padding-bottom: 2.5rem;
}
</style>
