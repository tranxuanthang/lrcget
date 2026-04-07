<template>
  <div
    class="word-timing-lane relative flex flex-col px-4 py-3 rounded-lg overflow-hidden"
    :class="hasSelectedLine ? 'bg-brave-95 dark:bg-brave-10 min-h-[5rem]' : 'bg-brave-98 dark:bg-brave-5 min-h-[3.5rem]'
"
  >
    <!-- Empty state -->
    <div v-if="!hasSelectedLine" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Select a lyric line to edit word timings
      </span>
    </div>

    <!-- Word timing timeline -->
    <template v-else>
      <!-- Header with line info -->
      <div class="flex items-center justify-between mb-2 shrink-0">
        <div class="flex items-center gap-3 text-xs text-brave-40 dark:text-brave-60">
          <span class="font-mono bg-brave-90 dark:bg-brave-20 px-2 py-0.5 rounded">
            {{ formatTimestampMs(selectedLine.start_ms) }} - {{ formatTimestampMs(lineEndMs) }}
          </span>
          <span class="truncate max-w-xs">{{ selectedLine.text || '(empty)' }}</span>
        </div>

        <button
          class="button button-normal text-xs px-2 py-1 rounded"
          title="Distribute word timings evenly"
          @click="handleDistributeEvenly"
        >
          Distribute evenly
        </button>
      </div>

      <!-- Timeline with word segments -->
      <div
        ref="timelineElement"
        class="timeline-container relative flex-1 min-h-[2rem] bg-brave-98 dark:bg-brave-5 rounded border border-brave-80 dark:border-brave-25 overflow-hidden"
        @click="handleTimelineClick"
      >
        <!-- Timeline grid lines (every 500ms) -->
        <div class="absolute inset-0 pointer-events-none">
          <template v-for="n in gridLinesCount" :key="n">
            <div
              class="absolute top-0 bottom-0 w-px bg-brave-80 dark:bg-brave-30 opacity-50"
              :style="{ left: `${(n / gridLinesCount) * 100}%` }"
            />
          </template>
        </div>

        <!-- Word segments -->
        <SyncedWordTimingSegment
          v-for="(word, index) in displayedWords"
          :key="index"
          :word="word"
          :word-index="index"
          :start-ms="word.start_ms"
          :end-ms="getWordEndMs(index)"
          :line-start-ms="selectedLine.start_ms"
          :line-end-ms="lineEndMs"
          :timeline-width="timelineWidth"
          :progress-ms="progressMs"
        />

        <button
          v-for="index in boundaryIndexes"
          :key="`boundary-${index}`"
          type="button"
          class="boundary-handle absolute top-0 bottom-0 z-30 -ml-2 w-4 cursor-ew-resize"
          :class="dragState?.rightWordIndex === index ? 'is-active' : ''"
          :style="{ left: `${timeToPercent(displayedWords[index].start_ms)}%` }"
          :title="`Adjust start of ${displayedWords[index].text}`"
          @pointerdown="startBoundaryDrag(index, $event)"
        >
          <span class="boundary-line" />
        </button>

        <div
          v-if="dragState"
          class="boundary-overlay absolute inset-y-0 z-20 pointer-events-none"
          :style="{ left: `${timeToPercent(dragState.currentStartMs)}%` }"
        >
          <div class="boundary-overlay-line" />
          <div class="boundary-overlay-badge">
            {{ formatTimestampMs(dragState.currentStartMs) }}
          </div>
        </div>

        <!-- Current playhead indicator -->
        <div
          v-if="progressMs >= selectedLine.start_ms && progressMs <= lineEndMs"
          class="absolute top-0 bottom-0 w-0.5 bg-brave-40 dark:bg-brave-60 z-20 pointer-events-none"
          :style="{ left: `${playheadPercent}%` }"
        >
          <div class="absolute -top-1 -left-1.5 w-4 h-4 rounded-full bg-brave-40 dark:bg-brave-60" />
        </div>
      </div>

      <!-- Words info -->
      <div class="flex items-center justify-between mt-2 text-xs text-brave-50 dark:text-brave-70">
        <span>{{ words.length }} word{{ words.length === 1 ? '' : 's' }}</span>
        <span v-if="dragState" class="text-brave-40 dark:text-brave-60">
          Dragging: {{ displayedWords[dragState.rightWordIndex]?.text }}
        </span>
      </div>
    </template>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import SyncedWordTimingSegment from '@/components/library/edit-lyrics-v2/SyncedWordTimingSegment.vue'
import { formatTimestampMs } from '@/utils/lyricsfile.js'
import { ensureLineWords, distributeWordTimings } from '@/utils/word-tokenizer.js'

const props = defineProps({
  selectedLine: {
    type: Object,
    default: null
  },
  hasSelectedLine: {
    type: Boolean,
    default: false
  },
  progressMs: {
    type: Number,
    default: 0
  },
  allLines: {
    type: Array,
    default: () => []
  },
  selectedLineIndex: {
    type: Number,
    default: -1
  }
})

const emit = defineEmits(['update:words', 'word-timing-edited'])

const timelineElement = ref(null)
const timelineWidth = ref(0)
const dragState = ref(null)

const lineEndMs = computed(() => {
  if (!props.selectedLine) return 0

  if (props.selectedLineIndex >= 0 && props.selectedLineIndex + 1 < props.allLines.length) {
    const nextLine = props.allLines[props.selectedLineIndex + 1]
    if (Number.isFinite(nextLine?.start_ms)) {
      return nextLine.start_ms
    }
  }

  if (Number.isFinite(props.selectedLine.start_ms)) {
    return props.selectedLine.start_ms + 2000
  }

  return 2000
})

const words = computed(() => {
  if (!props.selectedLine) return []

  const lineWithWords = ensureLineWords(
    props.selectedLine,
    props.allLines,
    props.selectedLineIndex
  )

  return lineWithWords.words || []
})

const displayedWords = computed(() => {
  const currentWords = words.value

  if (!dragState.value) {
    return currentWords
  }

  return currentWords.map((word, index) => {
    if (index !== dragState.value.rightWordIndex) {
      return word
    }

    return {
      ...word,
      start_ms: dragState.value.currentStartMs
    }
  })
})

const playheadPercent = computed(() => {
  if (!props.selectedLine || lineEndMs.value <= props.selectedLine.start_ms) {
    return 0
  }

  const duration = lineEndMs.value - props.selectedLine.start_ms
  const elapsed = props.progressMs - props.selectedLine.start_ms

  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
})

const gridLinesCount = computed(() => {
  if (!props.selectedLine) return 0

  const duration = lineEndMs.value - props.selectedLine.start_ms
  return Math.max(0, Math.floor(duration / 500) - 1)
})

const updateTimelineWidth = () => {
  if (timelineElement.value) {
    timelineWidth.value = timelineElement.value.clientWidth
  }
}

const timeToPercent = (timeMs) => {
  if (!props.selectedLine || lineEndMs.value <= props.selectedLine.start_ms) {
    return 0
  }

  const duration = lineEndMs.value - props.selectedLine.start_ms
  const elapsed = timeMs - props.selectedLine.start_ms
  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
}

const clientXToTime = (clientX) => {
  if (!timelineElement.value || !props.selectedLine || lineEndMs.value <= props.selectedLine.start_ms) {
    return props.selectedLine?.start_ms || 0
  }

  const rect = timelineElement.value.getBoundingClientRect()
  const width = rect.width
  if (width <= 0) {
    return props.selectedLine.start_ms
  }

  const clampedX = Math.max(0, Math.min(width, clientX - rect.left))
  const duration = lineEndMs.value - props.selectedLine.start_ms
  return Math.round(props.selectedLine.start_ms + (clampedX / width) * duration)
}

const boundaryIndexes = computed(() => {
  if (displayedWords.value.length < 2) {
    return []
  }

  return Array.from({ length: displayedWords.value.length - 1 }, (_, index) => index + 1)
})

const getWordEndMs = (index) => {
  if (index >= displayedWords.value.length - 1) {
    return lineEndMs.value
  }

  return displayedWords.value[index + 1].start_ms
}

const getBoundaryConstraint = (rightWordIndex) => {
  const currentWords = words.value
  const previousStartMs = currentWords[rightWordIndex - 1]?.start_ms ?? props.selectedLine?.start_ms ?? 0
  const nextStartMs = currentWords[rightWordIndex + 1]?.start_ms
  const minStartMs = previousStartMs + 1
  const maxStartMs = Number.isFinite(nextStartMs)
    ? nextStartMs - 1
    : lineEndMs.value - 1

  return {
    minStartMs,
    maxStartMs: Math.max(minStartMs, maxStartMs)
  }
}

const updateDragPosition = (clientX) => {
  if (!dragState.value) {
    return
  }

  const { minStartMs, maxStartMs } = getBoundaryConstraint(dragState.value.rightWordIndex)
  const nextStartMs = clientXToTime(clientX)
  dragState.value = {
    ...dragState.value,
    currentStartMs: Math.max(minStartMs, Math.min(maxStartMs, nextStartMs))
  }
}

const commitDraggedBoundary = () => {
  if (!dragState.value) {
    return
  }

  const { rightWordIndex, currentStartMs, initialStartMs } = dragState.value
  const hasChanged = currentStartMs !== initialStartMs
  
  if (hasChanged) {
    emit('update:words', {
      lineIndex: props.selectedLineIndex,
      words: words.value.map((word, index) => {
        if (index !== rightWordIndex) {
          return word
        }

        return {
          ...word,
          start_ms: currentStartMs
        }
      })
    })
    
    // Emit event to trigger auto-replay from the beginning of the edited line
    emit('word-timing-edited', {
      lineIndex: props.selectedLineIndex,
      startMs: props.selectedLine?.start_ms
    })
  }
}

const stopBoundaryDrag = () => {
  document.removeEventListener('pointermove', handlePointerMove)
  document.removeEventListener('pointerup', handlePointerUp)
  document.removeEventListener('pointercancel', handlePointerUp)
}

const handlePointerMove = (event) => {
  updateDragPosition(event.clientX)
}

const handlePointerUp = () => {
  commitDraggedBoundary()
  dragState.value = null
  stopBoundaryDrag()
}

const startBoundaryDrag = (rightWordIndex, event) => {
  event.preventDefault()
  event.stopPropagation()

  if (!props.selectedLine || rightWordIndex <= 0 || rightWordIndex >= words.value.length) {
    return
  }

  const initialStartMs = words.value[rightWordIndex].start_ms
  dragState.value = {
    rightWordIndex,
    initialStartMs,
    currentStartMs: initialStartMs
  }

  updateDragPosition(event.clientX)
  document.addEventListener('pointermove', handlePointerMove)
  document.addEventListener('pointerup', handlePointerUp)
  document.addEventListener('pointercancel', handlePointerUp)
}

watch(() => props.selectedLine, () => {
  stopBoundaryDrag()
  dragState.value = null
  nextTick(() => {
    updateTimelineWidth()
  })
}, { immediate: true })

const handleDistributeEvenly = () => {
  const currentWords = words.value
  if (currentWords.length === 0) return

  const newWords = distributeWordTimings(
    currentWords,
    props.selectedLine.start_ms,
    lineEndMs.value
  )

  emit('update:words', {
    lineIndex: props.selectedLineIndex,
    words: newWords
  })
  
  // Emit event to trigger auto-replay from the beginning of the edited line
  emit('word-timing-edited', {
    lineIndex: props.selectedLineIndex,
    startMs: props.selectedLine?.start_ms
  })
}

const handleTimelineClick = (event) => {
  event.stopPropagation()
}

watch(() => props.hasSelectedLine, (hasLine) => {
  if (hasLine) {
    nextTick(() => {
      updateTimelineWidth()
    })
  }
}, { immediate: true })

watch(() => props.allLines, () => {
  if (dragState.value) {
    dragState.value = null
    stopBoundaryDrag()
  }
}, { deep: true })

onMounted(() => {
  window.addEventListener('resize', updateTimelineWidth)
})

onUnmounted(() => {
  stopBoundaryDrag()
  window.removeEventListener('resize', updateTimelineWidth)
})
</script>

<style scoped>
.word-timing-lane {
  transition: min-height 0.2s ease;
}

.timeline-container {
  position: relative;
}

.boundary-handle {
  background: transparent;
}

.boundary-line {
  position: absolute;
  left: 50%;
  top: 0;
  bottom: 0;
  width: 2px;
  transform: translateX(-50%);
  background: rgba(148, 163, 184, 0.9);
  transition: background-color 0.12s ease, width 0.12s ease, box-shadow 0.12s ease;
}

.boundary-handle:hover .boundary-line,
.boundary-handle.is-active .boundary-line {
  background: rgba(56, 189, 248, 0.95);
  width: 3px;
  box-shadow: 0 0 0 1px rgba(56, 189, 248, 0.18);
}

.boundary-overlay-line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 3px;
  transform: translateX(-50%);
  background: rgba(56, 189, 248, 0.95);
  box-shadow: 0 0 0 1px rgba(56, 189, 248, 0.18);
}

.boundary-overlay-badge {
  position: absolute;
  top: -0.375rem;
  left: 0;
  transform: translate(-50%, -100%);
  padding: 0.125rem 0.4rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  white-space: nowrap;
  color: rgb(15 23 42);
  background: rgb(125 211 252);
}

:global(.dark) .boundary-line {
  background: rgba(100, 116, 139, 0.9);
}

:global(.dark) .boundary-handle:hover .boundary-line,
:global(.dark) .boundary-handle.is-active .boundary-line,
:global(.dark) .boundary-overlay-line {
  background: rgba(125, 211, 252, 0.95);
}

:global(.dark) .boundary-overlay-badge {
  color: rgb(226 232 240);
  background: rgb(14 116 144);
}
</style>
