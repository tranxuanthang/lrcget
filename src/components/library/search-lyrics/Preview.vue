<template>
  <BaseModal
    title="Preview"
    content-class="w-full h-[60vh] max-w-screen-md"
    body-class="flex flex-col gap-4 h-full min-h-0"
    @close="emit('close')"
  >
    <div
      class="flex flex-none gap-4 items-center bg-neutral-50 dark:bg-neutral-900 rounded-lg px-4 py-2"
    >
      <button
        v-if="status !== 'playing'"
        class="button button-primary text-white p-2 rounded-full text-xl"
        @click.prevent="resume"
      >
        <Play />
      </button>
      <button
        v-else
        class="button button-primary text-white p-2 rounded-full text-xl"
        @click.prevent="pause"
      >
        <Pause />
      </button>
      <div class="flex-none w-12 text-xs text-neutral-800 dark:text-neutral-400">
        {{ humanDuration(progress) }}
      </div>
      <Seek class="grow" :duration="duration" :progress="progress" @seek="seek" />
      <div class="flex-none w-12 text-xs text-neutral-800 dark:text-neutral-400">
        {{ humanDuration(duration) }}
      </div>
    </div>

    <div v-if="resolvedLyrics.syncedLyrics" class="relative grow h-full overflow-hidden">
      <div
        id="full-lyrics-container"
        class="h-full text-center transition"
        :style="{ transform: fullViewTransform }"
      >
        <p
          v-for="(line, index) in visibleSyncedLines"
          :key="index"
          class="text-neutral-700 dark:text-neutral-400"
          :class="{ 'font-bold dark:text-white': isLinePlaying(line, index) }"
        >
          <template v-if="hasWordSyncLine(line)">
            <span
              v-for="(word, wordIndex) in line.words"
              :key="wordIndex"
              class="whitespace-pre-wrap"
              :class="{
                'text-yellow-500 dark:text-yellow-400':
                  isLinePlaying(line, index) && isWordPlaying(line, wordIndex, progressMs),
              }"
              >{{ word.text }}</span
            >
          </template>
          <template v-else>
            {{ lineText(line) }}
          </template>
        </p>
      </div>

      <div
        class="absolute top-0 left-0 w-full h-10 bg-gradient-to-b from-white dark:from-neutral-900"
      />
      <div
        class="absolute bottom-0 left-0 w-full h-10 bg-gradient-to-t from-white dark:from-neutral-900"
      />
    </div>

    <div
      v-else-if="resolvedLyrics.plainLyrics"
      class="relative grow text-center text-neutral-700 dark:text-neutral-400 whitespace-pre h-full overflow-hidden"
    >
      <div class="grow p-4 h-full overflow-y-auto">
        {{ resolvedLyrics.plainLyrics }}
      </div>
    </div>

    <div
      v-else-if="resolvedLyrics.instrumental"
      class="relative grow text-center text-neutral-700 dark:text-neutral-400 whitespace-pre h-full overflow-hidden"
    >
      <div class="grow p-4 h-full overflow-y-auto italic flex items-center justify-center">
        This track is instrumental
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import Play from '~icons/mdi/play'
import Pause from '~icons/mdi/pause'
import { usePlayer } from '@/composables/player.js'
import Seek from '@/components/now-playing/Seek.vue'
import { normalizeLrclibLyrics, parseLyricsfile, parseLrcLines } from '@/utils/lyricsfile.js'

const props = defineProps(['track', 'lyrics'])
const { playingTrack, status, duration, progress, playTrack, pause, resume, stop, seek } =
  usePlayer()
const emit = defineEmits(['close'])

const parsedLyrics = ref(null)
const currentLineElementOffset = ref(null)

const resolvedLyrics = computed(() => normalizeLrclibLyrics(props.lyrics))
const lyricsfileSyncedLines = computed(() => {
  if (!props.lyrics?.lyricsfile) {
    return []
  }

  return parseLyricsfile(props.lyrics.lyricsfile).syncedLines || []
})

const hasWordSyncedLyrics = computed(() => {
  return lyricsfileSyncedLines.value.some(
    line => Array.isArray(line?.words) && line.words.length > 0
  )
})

const visibleSyncedLines = computed(() => {
  if (hasWordSyncedLyrics.value) {
    return lyricsfileSyncedLines.value
  }

  return parsedLyrics.value || []
})

const progressMs = computed(() => (progress.value ?? 0) * 1000)

const humanDuration = seconds => {
  if (!seconds) {
    seconds = 0
  }

  return new Date(seconds * 1000).toISOString().slice(11, 19)
}

const lineText = line => {
  if (!line) {
    return ''
  }

  if (typeof line.text === 'string') {
    return line.text
  }

  return line.content || ''
}

const hasWordSyncLine = line => {
  return Array.isArray(line?.words) && line.words.length > 0
}

const getCurrentWordIndex = (line, currentTimeMs) => {
  if (!hasWordSyncLine(line)) {
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

const isWordPlaying = (line, wordIndex, currentTimeMs) => {
  return getCurrentWordIndex(line, currentTimeMs) === wordIndex
}

const isLinePlaying = (line, index) => {
  if (!line || !Number.isFinite(line.start_ms)) {
    return false
  }
  const lineStart = line.start_ms
  const lineEnd = Number.isFinite(line.end_ms)
    ? line.end_ms
    : (visibleSyncedLines.value[index + 1]?.start_ms ?? Infinity)
  return progressMs.value >= lineStart && progressMs.value < lineEnd
}

const primaryPlayingLineIndex = computed(() => {
  const lines = visibleSyncedLines.value
  if (!lines || lines.length === 0) {
    return -1
  }

  const playingIndices = []
  for (let i = 0; i < lines.length; i++) {
    if (isLinePlaying(lines[i], i)) {
      playingIndices.push(i)
    }
  }

  if (playingIndices.length === 0) {
    for (let i = lines.length - 1; i >= 0; i--) {
      if (progressMs.value >= lines[i].start_ms) {
        return i
      }
    }
    return -1
  }

  return playingIndices[playingIndices.length - 1]
})

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

onMounted(() => {
  playTrack(props.track)

  if (resolvedLyrics.value.syncedLyrics) {
    parsedLyrics.value = parseLrcLines(resolvedLyrics.value.syncedLyrics)
  }

  nextTick(() => {
    updateCurrentLineElementOffset()
  })
})

onUnmounted(() => {
  stop()
  parsedLyrics.value = null
})

watch(primaryPlayingLineIndex, () => {
  nextTick(() => {
    updateCurrentLineElementOffset()
  })
})
</script>
