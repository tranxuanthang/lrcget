<template>
  <div
    class="word-timing-lane relative flex flex-col px-2 py-2 rounded-lg overflow-hidden h-[5rem]"
    :class="hasSelectedLine ? 'bg-brave-95 dark:bg-brave-10' : 'bg-brave-98 dark:bg-brave-5'
"
  >
    <!-- Empty state - no line selected -->
    <div v-if="!hasSelectedLine" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Select a lyric line to edit word timings
      </span>
    </div>

    <!-- Feature not available states -->
    <div v-else-if="!hasLineContent" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Add lyrics content to enable word timing
      </span>
    </div>

    <div v-else-if="!hasLineStartTime" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Sync the line (set start time) to enable word timing
      </span>
    </div>

    <div v-else-if="!hasNextLineStartTime" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Sync the next line to define the timing window
      </span>
    </div>

    <!-- Word timing timeline -->
    <template v-else-if="isWordSyncAvailable">
      <!-- Header with line info -->
      <div class="flex items-center justify-between mb-2 shrink-0">
        <div class="flex items-center gap-3 text-xs text-brave-40 dark:text-brave-60">
          <span class="font-mono bg-brave-90 dark:bg-brave-20 px-2 py-0.5 rounded">
            {{ formatTimestampMs(selectedLine.start_ms) }} - {{ formatTimestampMs(lineEndMs) }}
          </span>
          <span class="truncate max-w-xs">{{ selectedLine.text || '(empty)' }}</span>
        </div>

        <div class="flex items-center gap-2">
          <button
            class="button button-normal text-xs px-2 py-1 rounded flex items-center gap-1"
            title="Play line from beginning"
            @click="handlePlayLine"
          >
            <Play class="w-3.5 h-3.5" />
            <span>Play</span>
          </button>
          <button
            class="button button-primary text-xs px-2 py-1 rounded flex items-center gap-1"
            title="Sync word at current playback position (z)"
            @click="handleSyncWord"
          >
            <Equal class="w-3.5 h-3.5" />
            <span>Sync word</span>
          </button>
          <button
            class="button button-normal text-xs px-2 py-1 rounded flex items-center gap-1"
            title="Distribute word timings evenly"
            @click="handleDistributeEvenly"
          >
            <Undo class="w-3.5 h-3.5" />
            <span>Redistribute</span>
          </button>
        </div>
      </div>

      <!-- Timeline with word segments -->
      <div
        ref="timelineElement"
        class="timeline-container relative flex-1 bg-brave-98 dark:bg-brave-5 rounded border border-brave-80 dark:border-brave-25 overflow-hidden"
        @click="handleTimelineClick"
      >
        <!-- Timeline grid lines (every 500ms) -->
        <!-- <div class="absolute inset-0 pointer-events-none">
          <template v-for="n in gridLinesCount" :key="n">
            <div
              class="absolute top-0 bottom-0 w-px bg-brave-80 dark:bg-brave-30 opacity-50"
              :style="{ left: `${(n / gridLinesCount) * 100}%` }"
            />
          </template>
        </div> -->

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
          :class="{
            'is-active': dragState?.rightWordIndex === index || (!dragState && selectedBoundaryIndex === index),
            'is-selected': !dragState && selectedBoundaryIndex === index
          }"
          :style="{ left: `${timeToPercent(displayedWords[index].start_ms)}%` }"
          :title="`Adjust start of ${displayedWords[index].text}`"
          @pointerdown="startBoundaryDrag(index, $event)"
          @click="selectBoundary(index)"
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
    </template>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import { Equal, Play, Undo } from 'mdue'
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

const emit = defineEmits(['update:words', 'word-timing-edited', 'play-line', 'select-next-line'])

const timelineElement = ref(null)
const timelineWidth = ref(0)
const dragState = ref(null)
const selectedBoundaryIndex = ref(1) // First drag handle (boundary between word 0 and 1) is selected by default
const isDraggingBoundary = ref(false)

// Availability checks for word sync feature
const hasLineContent = computed(() => {
  return props.selectedLine && props.selectedLine.text && props.selectedLine.text.trim().length > 0
})

const hasLineStartTime = computed(() => {
  return props.selectedLine && Number.isFinite(props.selectedLine.start_ms)
})

const hasNextLineStartTime = computed(() => {
  if (!props.selectedLine || props.selectedLineIndex < 0) return false

  if (props.selectedLineIndex + 1 >= props.allLines.length) {
    return false
  }

  const nextLine = props.allLines[props.selectedLineIndex + 1]
  return Number.isFinite(nextLine?.start_ms)
})

const isWordSyncAvailable = computed(() => {
  return hasLineContent.value && hasLineStartTime.value && hasNextLineStartTime.value
})

const lineEndMs = computed(() => {
  if (!props.selectedLine) return 0

  if (props.selectedLineIndex >= 0 && props.selectedLineIndex + 1 < props.allLines.length) {
    const nextLine = props.allLines[props.selectedLineIndex + 1]
    if (Number.isFinite(nextLine?.start_ms)) {
      return nextLine.start_ms
    }
  }

  // Fallback for last line
  if (Number.isFinite(props.selectedLine.start_ms)) {
    return props.selectedLine.start_ms + 2000
  }

  return 2000
})

const words = computed(() => {
  if (!isWordSyncAvailable.value) return []

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
  if (!isWordSyncAvailable.value) return 0

  const duration = lineEndMs.value - props.selectedLine.start_ms
  if (duration <= 0) return 0

  const elapsed = props.progressMs - props.selectedLine.start_ms
  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
})

const gridLinesCount = computed(() => {
  if (!isWordSyncAvailable.value) return 0

  const duration = lineEndMs.value - props.selectedLine.start_ms
  if (!Number.isFinite(duration) || duration <= 0) return 0

  return Math.max(0, Math.floor(duration / 500) - 1)
})

const updateTimelineWidth = () => {
  if (timelineElement.value) {
    timelineWidth.value = timelineElement.value.clientWidth
  }
}

const timeToPercent = (timeMs) => {
  if (!isWordSyncAvailable.value) return 0

  const duration = lineEndMs.value - props.selectedLine.start_ms
  if (duration <= 0) return 0

  const elapsed = timeMs - props.selectedLine.start_ms
  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
}

const clientXToTime = (clientX) => {
  if (!timelineElement.value || !isWordSyncAvailable.value) {
    return props.selectedLine?.start_ms || 0
  }

  if (lineEndMs.value <= props.selectedLine.start_ms) {
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

  const nextWordStart = displayedWords.value[index + 1]?.start_ms
  return Number.isFinite(nextWordStart) ? nextWordStart : lineEndMs.value
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
  isDraggingBoundary.value = true
  updateDragPosition(event.clientX)
}

const handlePointerUp = () => {
  commitDraggedBoundary()
  // Small delay before clearing drag state so click handler knows we were dragging
  setTimeout(() => {
    isDraggingBoundary.value = false
  }, 0)
  dragState.value = null
  stopBoundaryDrag()
}

const startBoundaryDrag = (rightWordIndex, event) => {
  event.preventDefault()
  event.stopPropagation()

  if (!isWordSyncAvailable.value || rightWordIndex <= 0 || rightWordIndex >= words.value.length) {
    return
  }

  isDraggingBoundary.value = false
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

const selectBoundary = (index) => {
  if (!isWordSyncAvailable.value || index <= 0 || index >= words.value.length) {
    return
  }
  // Don't select if we just finished dragging
  if (isDraggingBoundary.value) {
    return
  }
  selectedBoundaryIndex.value = index
}

const handleSyncWord = () => {
  if (!isWordSyncAvailable.value) return

  const rightWordIndex = selectedBoundaryIndex.value
  if (rightWordIndex <= 0 || rightWordIndex >= words.value.length) return

  const currentProgress = props.progressMs
  const { minStartMs, maxStartMs } = getBoundaryConstraint(rightWordIndex)

  // Clamp the sync time to valid boundaries
  const newStartMs = Math.max(minStartMs, Math.min(maxStartMs, currentProgress))

  const oldStartMs = words.value[rightWordIndex].start_ms
  const hasChanged = newStartMs !== oldStartMs

  if (hasChanged) {
    emit('update:words', {
      lineIndex: props.selectedLineIndex,
      words: words.value.map((word, index) => {
        if (index !== rightWordIndex) {
          return word
        }
        return {
          ...word,
          start_ms: newStartMs
        }
      })
    })
    // Note: Does NOT emit 'word-timing-edited' - line is not replayed
  }

  // Move to next boundary (if not at the last word)
  const nextIndex = rightWordIndex + 1
  if (nextIndex < words.value.length) {
    selectedBoundaryIndex.value = nextIndex
  }
}

const handlePlayLine = () => {
  if (!isWordSyncAvailable.value) return
  emit('play-line', props.selectedLineIndex)
}

// Keyboard shortcuts
const handleKeyDown = (event) => {
  // Only handle shortcuts when word sync is available and we're not editing
  if (!isWordSyncAvailable.value) return

  // Don't trigger if user is typing in an input
  if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') return

  switch (event.key.toLowerCase()) {
    case 'z':
      event.preventDefault()
      handleSyncWord()
      break
    case 'x':
      event.preventDefault()
      // Check if we're at the last word handle BEFORE syncing
      const isAtLastBoundary = selectedBoundaryIndex.value >= words.value.length - 1
      // Sync current word
      handleSyncWord()
      // If we were at the last word handle, move to next line
      if (isAtLastBoundary) {
        // Check if there's a next line to move to
        const nextLineIndex = props.selectedLineIndex + 1
        if (nextLineIndex < props.allLines.length) {
          // Emit event to select next line - parent will handle this
          emit('select-next-line', nextLineIndex)
        }
      }
      break
  }
}

watch(() => props.selectedLineIndex, (newIndex, oldIndex) => {
  stopBoundaryDrag()
  dragState.value = null
  // Only reset boundary index when actually changing to a different line
  if (newIndex !== oldIndex) {
    selectedBoundaryIndex.value = 1 // Reset to first boundary when line changes
  }
  nextTick(() => {
    updateTimelineWidth()
  })
}, { immediate: true })

const handleDistributeEvenly = () => {
  if (!isWordSyncAvailable.value) return

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

  // Reset selected boundary to first after distribute evenly
  selectedBoundaryIndex.value = 1

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

watch(isWordSyncAvailable, (available) => {
  if (!available && dragState.value) {
    dragState.value = null
    stopBoundaryDrag()
  }
})

watch(() => props.allLines, () => {
  if (dragState.value) {
    dragState.value = null
    stopBoundaryDrag()
  }
}, { deep: true })

onMounted(() => {
  window.addEventListener('resize', updateTimelineWidth)
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  stopBoundaryDrag()
  window.removeEventListener('resize', updateTimelineWidth)
  window.removeEventListener('keydown', handleKeyDown)
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
.boundary-handle.is-active .boundary-line,
.boundary-handle.is-selected .boundary-line {
  background: rgba(56, 189, 248, 0.95);
  width: 3px;
  box-shadow: 0 0 0 1px rgba(56, 189, 248, 0.18);
}

/* Lightened up color for selected state (when not dragging) */
.boundary-handle.is-selected .boundary-line {
  background: rgba(125, 211, 252, 0.95);
  box-shadow: 0 0 0 2px rgba(125, 211, 252, 0.3);
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
:global(.dark) .boundary-handle.is-selected .boundary-line,
:global(.dark) .boundary-overlay-line {
  background: rgba(125, 211, 252, 0.95);
}

/* Lightened up color for selected state in dark mode (when not dragging) */
:global(.dark) .boundary-handle.is-selected .boundary-line {
  background: rgba(125, 211, 252, 0.95);
  box-shadow: 0 0 0 2px rgba(125, 211, 252, 0.4);
}

:global(.dark) .boundary-overlay-badge {
  color: rgb(226 232 240);
  background: rgb(14 116 144);
}
</style>
