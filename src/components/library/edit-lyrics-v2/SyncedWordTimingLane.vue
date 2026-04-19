<template>
  <div
    class="relative flex flex-col px-2 py-2 rounded-lg overflow-hidden h-[5rem] transition-[min-height] duration-200 ease-out"
    :class="hasSelectedLine ? 'bg-brave-95 dark:bg-brave-10' : 'bg-brave-98 dark:bg-brave-5'"
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

    <div v-else-if="!hasLineEndTime" class="flex items-center justify-center h-full">
      <span class="text-sm text-brave-50 dark:text-brave-70 italic">
        Set the line end timestamp to define the timing window
      </span>
    </div>

    <!-- Word timing timeline -->
    <template v-else-if="isWordSyncAvailable">
      <!-- Header with line info -->
      <div class="flex items-center justify-between mb-2 shrink-0">
        <div class="flex items-center gap-3 text-xs text-brave-40 dark:text-brave-60">
          <span class="font-mono bg-brave-90 dark:bg-brave-20 px-2 py-0.5 rounded">
            {{ formatTimestampMs(selectedLine.start_ms) }} -
            {{ formatTimestampMs(actualLineEndMs) }}
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
            title="Reset word timings to default state"
            @click="handleResetWords"
          >
            <Close class="w-3.5 h-3.5" />
            <span>Reset</span>
          </button>
        </div>
      </div>

      <!-- Timeline with word segments -->
      <div
        ref="timelineElement"
        class="relative flex-1 bg-brave-98 dark:bg-brave-5 rounded border border-brave-80 dark:border-brave-25 transition-opacity duration-200"
        :class="{ 'opacity-50': !hasActualWords }"
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
          :line-start-ms="laneStartMs"
          :line-end-ms="laneEndMs"
          :timeline-width="timelineWidth"
          :progress-ms="progressMs"
        />

        <button
          v-for="index in boundaryIndexes"
          :key="`boundary-${index}`"
          type="button"
          class="group absolute top-0 bottom-0 z-30 -ml-2 w-4 cursor-ew-resize bg-transparent"
          :style="{ left: `${timeToPercent(displayedWords[index].start_ms)}%` }"
          :title="`Adjust start of ${displayedWords[index].text}`"
          @pointerdown="handleBoundaryPointerDown(index, $event)"
          @click="selectBoundary(index)"
        >
          <span
            class="absolute left-1/2 top-0 bottom-0 w-0.5 -translate-x-1/2 transition-all duration-150 ease-linear bg-brave-70/70 dark:bg-brave-60/70 group-hover:bg-brave-50 dark:group-hover:bg-brave-70 group-hover:w-[3px] group-hover:ring-1 group-hover:ring-brave-50/25"
            :class="getBoundaryLineClass(index)"
          />
        </button>

        <div
          v-if="dragState"
          class="absolute inset-y-0 z-20 pointer-events-none"
          :style="{ left: `${timeToPercent(dragState.currentStartMs)}%` }"
        >
          <div
            class="absolute top-0 bottom-0 w-[3px] -translate-x-1/2 bg-brave-50 dark:bg-brave-70 ring-1 ring-brave-50/25"
          />
          <div
            class="absolute top-[-0.375rem] left-0 -translate-x-1/2 -translate-y-full px-[0.4rem] py-0.5 rounded-full text-xs leading-4 whitespace-nowrap text-brave-20 bg-brave-80 dark:text-brave-95 dark:bg-brave-30"
          >
            {{ formatTimestampMs(dragState.currentStartMs) }}
          </div>
        </div>

        <!-- Current playhead indicator -->
        <div
          v-if="progressMs >= lineStartMs && progressMs <= laneEndMs"
          class="absolute -top-1 bottom-0 w-px bg-brave-40/80 dark:bg-brave-50 z-20 pointer-events-none"
          :style="{ left: `${playheadPercent}%` }"
        >
          <div
            class="absolute -top-1 -left-[3px] w-0 h-0 border-l-[4px] border-r-[4px] border-t-[6px] border-l-transparent border-r-transparent border-t-brave-40/80 dark:border-t-brave-50"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import Equal from '~icons/mdi/equal'
import Play from '~icons/mdi/play'
import Close from '~icons/mdi/close'
import SyncedWordTimingSegment from '@/components/library/edit-lyrics-v2/SyncedWordTimingSegment.vue'
import { useEditLyricsV2WordBoundaryDrag } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordBoundaryDrag.js'
import { useEditLyricsV2WordTimingHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordTimingHotkeys.js'
import { formatTimestampMs } from '@/utils/lyricsfile.js'
import { ensureLineWords, distributeWordTimings, hasValidWords } from '@/utils/word-tokenizer.js'

const props = defineProps({
  selectedLine: {
    type: Object,
    default: null,
  },
  hasSelectedLine: {
    type: Boolean,
    default: false,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
  allLines: {
    type: Array,
    default: () => [],
  },
  selectedLineIndex: {
    type: Number,
    default: -1,
  },
})

const emit = defineEmits(['update:words', 'word-timing-edited', 'play-line', 'select-next-line'])

const timelineElement = ref(null)
const timelineWidth = ref(0)
const laneStartMs = ref(0)
const laneEndMs = ref(0)

// Availability checks for word sync feature
const hasLineContent = computed(() => {
  return props.selectedLine && props.selectedLine.text && props.selectedLine.text.trim().length > 0
})

const hasLineStartTime = computed(() => {
  return props.selectedLine && Number.isFinite(props.selectedLine.start_ms)
})

const hasLineEndTime = computed(() => {
  return props.selectedLine && Number.isFinite(props.selectedLine.end_ms)
})

const isWordSyncAvailable = computed(() => {
  return hasLineContent.value && hasLineStartTime.value && hasLineEndTime.value
})

// Check if the line has actual saved words (not auto-generated)
const hasActualWords = computed(() => {
  return hasValidWords(props.selectedLine)
})

const actualLineEndMs = computed(() => {
  if (!props.selectedLine) return 0

  // Use the line's own end_ms if available
  if (Number.isFinite(props.selectedLine.end_ms)) {
    return props.selectedLine.end_ms
  }

  // Fallback: start_ms + 2000ms
  if (Number.isFinite(props.selectedLine.start_ms)) {
    return props.selectedLine.start_ms + 2000
  }

  return 2000
})

const lineStartMs = computed(() => {
  return Number.isFinite(props.selectedLine?.start_ms) ? props.selectedLine.start_ms : 0
})

const syncLaneWindowToSelection = () => {
  if (!props.hasSelectedLine || !props.selectedLine) {
    laneStartMs.value = 0
    laneEndMs.value = 0
    return
  }

  laneStartMs.value = lineStartMs.value
  laneEndMs.value = actualLineEndMs.value
}

const words = computed(() => {
  if (!isWordSyncAvailable.value) return []

  const lineWithWords = ensureLineWords(props.selectedLine, props.allLines, props.selectedLineIndex)

  return lineWithWords.words || []
})

const {
  dragState,
  displayedWords,
  boundaryIndexes,
  selectedBoundaryIndex,
  startBoundaryDrag,
  selectBoundary,
  syncSelectedBoundary,
  resetBoundarySelection,
  cancelBoundaryInteraction,
} = useEditLyricsV2WordBoundaryDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  timelineStartMs: laneStartMs,
  timelineEndMs: laneEndMs,
  selectedLineIndex: computed(() => props.selectedLineIndex),
  onUpdateWords: payload => emit('update:words', payload),
  onWordTimingEdited: payload => emit('word-timing-edited', payload),
})

const playheadPercent = computed(() => {
  if (!isWordSyncAvailable.value) return 0

  const duration = laneEndMs.value - laneStartMs.value
  if (duration <= 0) return 0

  const elapsed = props.progressMs - laneStartMs.value
  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
})

const updateTimelineWidth = () => {
  if (timelineElement.value) {
    timelineWidth.value = timelineElement.value.clientWidth
  }
}

const timeToPercent = timeMs => {
  if (!isWordSyncAvailable.value) return 0

  const duration = laneEndMs.value - laneStartMs.value
  if (duration <= 0) return 0

  const elapsed = timeMs - laneStartMs.value
  return Math.max(0, Math.min(100, (elapsed / duration) * 100))
}

const clientXToTime = clientX => {
  if (!timelineElement.value || !isWordSyncAvailable.value) {
    return laneStartMs.value
  }

  if (laneEndMs.value <= laneStartMs.value) {
    return laneStartMs.value
  }

  const rect = timelineElement.value.getBoundingClientRect()
  const width = rect.width
  if (width <= 0) {
    return laneStartMs.value
  }

  const clampedX = Math.max(0, Math.min(width, clientX - rect.left))
  const duration = laneEndMs.value - laneStartMs.value
  return Math.round(laneStartMs.value + (clampedX / width) * duration)
}

const getWordEndMs = index => {
  if (index >= displayedWords.value.length - 1) {
    return laneEndMs.value
  }

  const nextWordStart = displayedWords.value[index + 1]?.start_ms
  return Number.isFinite(nextWordStart) ? nextWordStart : laneEndMs.value
}

const handleBoundaryPointerDown = (rightWordIndex, event) => {
  startBoundaryDrag(rightWordIndex, event, clientXToTime)
}

const getBoundaryLineClass = index => {
  const isActive =
    dragState.value?.rightWordIndex === index ||
    (!dragState.value && selectedBoundaryIndex.value === index)
  const isSelected = !dragState.value && selectedBoundaryIndex.value === index

  if (isSelected) {
    return 'bg-brave-60 dark:bg-brave-70 w-[3px] ring-2 ring-brave-70/35'
  }

  if (isActive) {
    return 'bg-brave-50 dark:bg-brave-70 w-[3px] ring-1 ring-brave-50/25'
  }

  return ''
}

const handleSyncWord = () => {
  syncSelectedBoundary(props.progressMs)
}

const handlePlayLine = () => {
  if (!isWordSyncAvailable.value) return
  emit('play-line', props.selectedLineIndex)
}

const { bindWordTimingHotkeys, unbindWordTimingHotkeys } = useEditLyricsV2WordTimingHotkeys({
  isWordSyncAvailable,
  selectedBoundaryIndex,
  words,
  selectedLineIndex: computed(() => props.selectedLineIndex),
  allLines: computed(() => props.allLines),
  syncSelectedBoundaryAtProgress: () => handleSyncWord(),
  onSelectNextLine: nextLineIndex => emit('select-next-line', nextLineIndex),
})

watch(
  () => props.selectedLineIndex,
  (newIndex, oldIndex) => {
    cancelBoundaryInteraction()
    // Only reset boundary index when actually changing to a different line
    if (newIndex !== oldIndex) {
      resetBoundarySelection()
      syncLaneWindowToSelection()
    } else if (!props.hasSelectedLine) {
      syncLaneWindowToSelection()
    }
    nextTick(() => {
      updateTimelineWidth()
    })
  },
  { immediate: true }
)

const handleResetWords = () => {
  if (!isWordSyncAvailable.value) return

  // Clear the words object entirely - this will make the timeline
  // appear with balanced words but reduced opacity (auto-generated state)
  emit('update:words', {
    lineIndex: props.selectedLineIndex,
    words: undefined,
  })

  // Reset selected boundary to first after reset
  resetBoundarySelection()
}

const handleTimelineClick = event => {
  event.stopPropagation()
}

watch(
  () => props.hasSelectedLine,
  hasLine => {
    if (hasLine) {
      nextTick(() => {
        updateTimelineWidth()
      })
      return
    }

    syncLaneWindowToSelection()
  },
  { immediate: true }
)

watch(isWordSyncAvailable, (available, wasAvailable) => {
  if (!available) {
    cancelBoundaryInteraction()
    return
  }

  if (!wasAvailable) {
    syncLaneWindowToSelection()
  }
})

watch(
  () => props.allLines,
  () => {
    if (dragState.value) {
      cancelBoundaryInteraction()
    }
  },
  { deep: true }
)

onMounted(() => {
  window.addEventListener('resize', updateTimelineWidth)
  bindWordTimingHotkeys()
})

onUnmounted(() => {
  cancelBoundaryInteraction()
  window.removeEventListener('resize', updateTimelineWidth)
  unbindWordTimingHotkeys()
})
</script>
