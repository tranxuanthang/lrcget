<template>
  <transition name="slide-fade" mode="out-in">
    <div v-if="lyrics && duration && progress" class="flex flex-col gap-1 border-b border-brave-90/50 relative z-10">
      <transition name="slide-fade" mode="out-in">
        <div v-if="expanded" class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-brave-95 border-t border-brave-90/50 overflow-hidden">
          <div class="relative h-full">
            <div class="flex justify-center items-center h-6 w-full relative z-10">
              <button class="text-xl text-brave-30 w-full flex justify-center" type="button" @click="expand"><DragHorizontal /></button>
            </div>

            <div id="full-lyrics-container" class="h-full text-center transition" :style="{ transform: fullViewTransform }">
              <p v-for="(line, index) in parsedLyrics" :key="index" class="text-brave-50" :class="{ 'font-bold': currentIndex === index }">{{ line.content }}</p>
            </div>

            <div class="absolute top-0 left-0 w-full h-10 bg-gradient-to-b from-brave-95"></div>
            <div class="absolute bottom-0 left-0 w-full h-10 bg-gradient-to-t from-brave-95"></div>
          </div>
        </div>
      </transition>

      <div class="mini-viewer transition cursor-pointer top-0 left-0 w-full h-12 bg-brave-95 border-t border-brave-90/50 flex flex-col" :class="{ 'invisible opacity-0': expanded }" @click="expand">
        <div class="flex justify-center items-center h-4 w-full">
          <button class="text-xl text-brave-30 w-full flex justify-center" type="button"><DragHorizontal /></button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div class="flex w-full justify-center items-center text-brave-30 text-sm grow" :key="currentLyrics">{{ currentLyrics }}</div>
        </transition>
      </div>
    </div>
  </transition>
</template>

<script setup>
import { DragHorizontal } from 'mdue'
import { Lrc, Runner } from 'lrc-kit'
import { ref, onMounted, watch, nextTick } from 'vue'
import { computed } from '@vue/reactivity'

const props = defineProps(['lyrics', 'duration', 'progress'])
const runner = ref(null)
const parsedLyrics = ref(null)
const currentIndex = ref(null)
const currentLyrics = ref(null)
const expanded = ref(false)
const currentLineElementOffset = ref(null)
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

onMounted(() => {
  const parsed = Lrc.parse(props.lyrics)

  runner.value = new Runner(parsed)
  parsedLyrics.value = runner.value.getLyrics()
})

watch(() => props.lyrics, (newLyrics) => {
  if (!newLyrics) {
    return
  }
  const parsed = Lrc.parse(newLyrics)

  runner.value = new Runner(parsed)
  parsedLyrics.value = runner.value.getLyrics()
})

watch(() => props.progress, (newProgress) => {
  if (!runner.value || !props.lyrics) {
    return
  }

  runner.value.timeUpdate(newProgress)
  let resultCurrentIndex = runner.value.curIndex()

  if (resultCurrentIndex === null) {
    resultCurrentIndex = -1
  }

  currentIndex.value = resultCurrentIndex

  if (currentIndex.value === -1) {
    currentLyrics.value = '…'
    return
  }

  try {
    const currentLyricsObj = runner.value.getLyric(resultCurrentIndex)
    currentLyrics.value = currentLyricsObj.content
  } catch (error) {
    console.error(error)
  }
})

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
