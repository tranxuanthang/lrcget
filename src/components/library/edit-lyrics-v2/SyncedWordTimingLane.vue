<template>
  <div
    class="relative z-20 flex flex-col px-2 py-2 rounded-lg overflow-visible h-[5rem] transition-[min-height] duration-200 ease-out"
    :class="hasSelectedLine ? 'bg-neutral-100 dark:bg-neutral-800' : 'bg-white dark:bg-neutral-950'"
  >
    <!-- Empty state - no line selected -->
    <div v-if="!hasSelectedLine" class="flex items-center justify-center h-full">
      <span class="text-sm text-neutral-700 dark:text-neutral-400 italic">
        Select a lyric line to edit word timings
      </span>
    </div>

    <!-- Feature not available states -->
    <div v-else-if="!hasLineContent" class="flex items-center justify-center h-full">
      <span class="text-sm text-neutral-700 dark:text-neutral-400 italic">
        Add lyrics content to enable word timing
      </span>
    </div>

    <div v-else-if="!hasLineStartTime" class="flex items-center justify-center h-full">
      <span class="text-sm text-neutral-700 dark:text-neutral-400 italic">
        Sync the line (set start time) to enable word timing
      </span>
    </div>

    <div v-else-if="!hasLineEndTime" class="flex items-center justify-center h-full">
      <span class="text-sm text-neutral-700 dark:text-neutral-400 italic">
        Set the line end timestamp to define the timing window
      </span>
    </div>

    <!-- Word timing timeline -->
    <template v-else-if="isWordSyncAvailable">
      <!-- Header with line info -->
      <div class="flex items-center justify-between mb-2 shrink-0">
        <div class="flex items-center gap-3 text-xs text-neutral-600 dark:text-neutral-400">
          <span class="font-mono bg-neutral-200 dark:bg-neutral-700 text-neutral-800 dark:text-neutral-200 px-2 py-0.5 rounded">
            {{ formatTimestampMs(selectedLine.start_ms) }} -
            {{ formatTimestampMs(actualLineEndMs) }}
          </span>
          <span class="truncate max-w-xs">{{ selectedLine.text || '(empty)' }}</span>
        </div>

        <div class="flex items-center gap-2">
          <button
            class="button button-normal text-xs px-2 py-1 rounded flex items-center gap-1"
            :title="playLineTitle"
            @click="handlePlayLine"
          >
            <Play class="w-3.5 h-3.5" />
            <span>Play</span>
          </button>
          <button
            class="button button-primary text-xs px-2 py-1 rounded flex items-center gap-1"
            :title="syncWordTitle"
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
          <VDropdown theme="lrcget-dropdown" placement="bottom-end">
            <button
              class="button button-normal text-xs px-1.5 py-1 rounded flex items-center"
              title="More actions"
              type="button"
            >
              <DotsVertical class="w-3.5 h-3.5" />
            </button>

            <template #popper>
              <div class="dropdown-container">
                <button v-close-popper class="dropdown-item" @click="handleImportLrcFile">
                  <FileMusicOutline class="text-neutral-800 dark:text-neutral-300" />
                  <span class="dropdown-label">Import LRC file</span>
                </button>
                <button v-close-popper class="dropdown-item" @click="handleImportLyricsfile">
                  <FileDocumentOutline class="text-neutral-800 dark:text-neutral-300" />
                  <span class="dropdown-label">Import Lyricsfile</span>
                </button>
                <div class="dropdown-divider" />
                <button v-close-popper class="dropdown-item" @click="handleClearAllTimings">
                  <DeleteSweepOutline class="text-neutral-800 dark:text-neutral-300" />
                  <span class="dropdown-label">Clear all timings</span>
                </button>
              </div>
            </template>
          </VDropdown>
        </div>
      </div>

      <!-- Timeline with word segments -->
      <div
        ref="timelineElement"
        class="relative flex-1 bg-white dark:bg-neutral-900 rounded border border-neutral-300 dark:border-neutral-600 transition-opacity duration-200"
        :class="{ 'opacity-50': !hasActualWords }"
        @click="handleTimelineClick"
      >
        <!-- Timeline grid lines (every 500ms) -->
        <!-- <div class="absolute inset-0 pointer-events-none">
          <template v-for="n in gridLinesCount" :key="n">
            <div
              class="absolute top-0 bottom-0 w-px bg-neutral-200 dark:bg-hoa-1100 opacity-50"
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
          :next-word-text="displayedWords[index]?.text || ''"
          :next-word-start-ms="displayedWords[index]?.start_ms ?? null"
          :next-word-end-ms="
            index + 1 < displayedWords.length ? getWordEndMs(index + 1) : null
          "
          :start-ms="word.start_ms"
          :end-ms="getWordEndMs(index)"
          :line-start-ms="laneStartMs"
          :line-end-ms="laneEndMs"
          :timeline-width="timelineWidth"
          :progress-ms="progressMs"
          :selected-boundary-index="selectedBoundaryIndex"
          :selected-boundary-indices="selectedBoundaryIndices"
          @split-at="handleSegmentSplitAt"
        />

        <button
          v-for="index in boundaryIndexes"
          :key="`boundary-${index}`"
          type="button"
          class="group absolute top-0 bottom-0 z-30 -ml-2 w-4 cursor-ew-resize bg-transparent"
          :style="{ left: `${timeToPercent(displayedWords[index].start_ms)}%` }"
          :title="`Adjust start of ${displayedWords[index].text}`"
          @pointerdown="handleBoundaryPointerDown(index, $event)"
          @click="selectBoundary(index, $event)"
        >
          <span
            class="absolute left-1/2 top-0 bottom-0 w-0.5 -translate-x-1/2 transition-all duration-150 ease-linear bg-neutral-300/70 dark:bg-hoa-1000/70 group-hover:bg-neutral-600 dark:group-hover:bg-neutral-300 group-hover:w-[3px] group-hover:ring-1 group-hover:ring-neutral-500/25"
            :class="getBoundaryLineClass(index)"
          />
        </button>

        <div
          v-if="dragState"
          class="absolute inset-y-0 z-20 pointer-events-none"
          :style="{ left: `${timeToPercent(dragState.currentStartMs)}%` }"
        >
          <div
            class="absolute top-0 bottom-0 w-[3px] -translate-x-1/2 bg-neutral-600 dark:bg-neutral-300 ring-1 ring-neutral-500/25"
          />
          <div
            class="absolute top-[-0.375rem] left-0 -translate-x-1/2 -translate-y-full px-[0.4rem] py-0.5 rounded-full text-xs leading-4 whitespace-nowrap text-neutral-800 bg-neutral-200 dark:text-white dark:bg-hoa-1100"
          >
            {{ formatTimestampMs(dragState.currentStartMs) }}
          </div>
        </div>

        <!-- Current playhead indicator -->
        <div
          v-if="progressMs >= lineStartMs && progressMs <= laneEndMs"
          class="absolute -top-1 bottom-0 w-px bg-neutral-400 dark:bg-neutral-400 z-20 pointer-events-none"
          :style="{ left: `${playheadPercent}%` }"
        >
          <div
            class="absolute -top-1 -left-[3px] w-0 h-0 border-l-[4px] border-r-[4px] border-t-[6px] border-l-transparent border-r-transparent border-t-neutral-400 dark:border-t-neutral-400"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Equal from '~icons/mdi/equal'
import Play from '~icons/mdi/play'
import Close from '~icons/mdi/close'
import DotsVertical from '~icons/mdi/dots-vertical'
import FileMusicOutline from '~icons/mdi/file-music-outline'
import FileDocumentOutline from '~icons/mdi/file-document-outline'
import DeleteSweepOutline from '~icons/mdi/delete-sweep-outline'
import SyncedWordTimingSegment from '@/components/library/edit-lyrics-v2/SyncedWordTimingSegment.vue'
import { useEditLyricsV2WordBoundaryDrag } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordBoundaryDrag.js'
import { useEditLyricsV2WordTimingHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordTimingHotkeys.js'
import {
  syncedEditorShortcutBindings,
  wordTimingShortcutBindings,
  withShortcutTitle,
} from '@/composables/edit-lyrics-v2/shortcutRegistry.js'
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

const emit = defineEmits([
  'update:words',
  'word-timing-edited',
  'play-line',
  'select-next-line',
  'import-lrc-file',
  'import-lyricsfile',
  'clear-all-timings',
])

const timelineElement = ref(null)
const timelineWidth = ref(0)
const laneStartMs = ref(0)
const laneEndMs = ref(0)
const segmentedTokenTexts = ref(null)
const segmentationRequestId = ref(0)

const playLineTitle = withShortcutTitle(
  'Play line from beginning',
  syncedEditorShortcutBindings,
  'replaySelectedLine'
)

const syncWordTitle = withShortcutTitle(
  'Sync word at current playback position',
  wordTimingShortcutBindings,
  'syncSelectedSeparatorAndAdvance'
)

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

  if (!hasActualWords.value) {
    const lineText = props.selectedLine?.text || ''
    const charabiaTokens = segmentedTokenTexts.value

    if (
      Array.isArray(charabiaTokens) &&
      charabiaTokens.length > 0 &&
      charabiaTokens.join('') === lineText
    ) {
      return distributeWordTimings(
        charabiaTokens.map(text => ({ text })),
        lineStartMs.value,
        actualLineEndMs.value
      )
    }
  }

  const lineWithWords = ensureLineWords(props.selectedLine, props.allLines, props.selectedLineIndex)

  return lineWithWords.words || []
})

const {
  dragState,
  displayedWords,
  boundaryIndexes,
  selectedBoundaryIndex,
  selectedBoundaryIndices,
  startBoundaryDrag,
  selectBoundary,
  isBoundarySelected,
  selectPreviousBoundary,
  selectNextBoundary,
  syncSelectedBoundary,
  deleteSelectedBoundaries,
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
  const isSelected = !dragState.value && isBoundarySelected(index)

  if (isSelected) {
    return 'bg-hoa-1000 dark:bg-neutral-300 w-[3px] ring-2 ring-neutral-300/35'
  }

  if (isActive) {
    return 'bg-neutral-600 dark:bg-neutral-300 w-[3px] ring-1 ring-neutral-500/25'
  }

  return ''
}

const handleSyncWord = () => {
  const selectedBoundaryBeforeSync = selectedBoundaryIndex.value
  const synced = syncSelectedBoundary(props.progressMs)

  if (
    synced &&
    selectedBoundaryBeforeSync === words.value.length - 1 &&
    props.selectedLineIndex < props.allLines.length - 1
  ) {
    emit('select-next-line')
  }
}

const handleSyncWordNoAdvance = () => {
  syncSelectedBoundary(props.progressMs, { advance: false })
}

const handlePlayLine = () => {
  if (!isWordSyncAvailable.value) return
  emit('play-line', props.selectedLineIndex)
}

const splitTextByGrapheme = text => {
  if (!text || typeof text !== 'string') {
    return []
  }

  if (typeof Intl !== 'undefined' && typeof Intl.Segmenter === 'function') {
    const segmenter = new Intl.Segmenter('und', { granularity: 'grapheme' })
    return Array.from(segmenter.segment(text), item => item.segment)
  }

  return Array.from(text)
}

const getWordEndMsFromList = (wordsList, index) => {
  if (index >= wordsList.length - 1) {
    return laneEndMs.value
  }

  const nextWordStart = wordsList[index + 1]?.start_ms
  return Number.isFinite(nextWordStart) ? nextWordStart : laneEndMs.value
}

const handleDeleteSelectedBoundaries = () => {
  if (!isWordSyncAvailable.value) return
  deleteSelectedBoundaries()
}

const handleSegmentSplitAt = ({ wordIndex, splitIndex, splitRatio }) => {
  if (!isWordSyncAvailable.value) {
    return
  }

  if (!Number.isInteger(wordIndex) || wordIndex < 0 || wordIndex >= displayedWords.value.length) {
    return
  }

  const currentWord = displayedWords.value[wordIndex]
  const graphemes = splitTextByGrapheme(currentWord?.text || '')
  if (graphemes.length <= 1) {
    return
  }

  const fallbackSplitIndex = Math.max(1, Math.min(graphemes.length - 1, Math.floor(graphemes.length / 2)))
  const normalizedSplitIndex = Number.isInteger(splitIndex)
    ? Math.max(1, Math.min(graphemes.length - 1, splitIndex))
    : fallbackSplitIndex

  const leftText = graphemes.slice(0, normalizedSplitIndex).join('')
  const rightText = graphemes.slice(normalizedSplitIndex).join('')

  const wordStartMs = Number.isFinite(currentWord?.start_ms) ? currentWord.start_ms : laneStartMs.value
  const wordEndMs = getWordEndMsFromList(displayedWords.value, wordIndex)
  const normalizedSplitRatio = Number.isFinite(splitRatio)
    ? Math.max(0, Math.min(1, splitRatio))
    : normalizedSplitIndex / graphemes.length
  const splitTimeMs = Math.max(
    wordStartMs + 1,
    Math.min(wordEndMs - 1, Math.round(wordStartMs + (wordEndMs - wordStartMs) * normalizedSplitRatio))
  )

  const updatedWords = [
    ...displayedWords.value.slice(0, wordIndex),
    { text: leftText, start_ms: wordStartMs },
    { text: rightText, start_ms: splitTimeMs },
    ...displayedWords.value.slice(wordIndex + 1),
  ]

  emit('update:words', {
    lineIndex: props.selectedLineIndex,
    words: updatedWords,
    lineStartMs: laneStartMs.value,
  })

  selectBoundary(Math.min(updatedWords.length - 1, wordIndex + 1))
}

const loadDefaultSegmentation = async ({ force = false } = {}) => {
  if (!isWordSyncAvailable.value) {
    segmentedTokenTexts.value = null
    return
  }

  if (!force && hasActualWords.value) {
    segmentedTokenTexts.value = null
    return
  }

  const lineText = props.selectedLine?.text || ''
  if (!lineText) {
    segmentedTokenTexts.value = []
    return
  }

  const requestId = ++segmentationRequestId.value

  try {
    const tokens = await invoke('segment_words', { text: lineText })

    if (requestId !== segmentationRequestId.value) {
      return
    }

    if (!Array.isArray(tokens)) {
      segmentedTokenTexts.value = null
      return
    }

    segmentedTokenTexts.value = tokens
      .filter(token => typeof token === 'string')
      .filter(token => token.length > 0)
  } catch (error) {
    if (requestId !== segmentationRequestId.value) {
      return
    }

    segmentedTokenTexts.value = null
    console.error(error)
  }
}

const { bindWordTimingHotkeys, unbindWordTimingHotkeys } = useEditLyricsV2WordTimingHotkeys({
  isWordSyncAvailable,
  selectedBoundaryIndex,
  words,
  syncSelectedBoundaryAtProgress: () => handleSyncWord(),
  syncSelectedBoundaryAtProgressNoAdvance: () => handleSyncWordNoAdvance(),
  selectPreviousBoundary: () => selectPreviousBoundary(),
  selectNextBoundary: () => selectNextBoundary(),
  resetBoundarySelection: () => resetBoundarySelection(),
  deleteSelectedBoundaries: () => handleDeleteSelectedBoundaries(),
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

    segmentedTokenTexts.value = null
    void loadDefaultSegmentation()

    nextTick(() => {
      updateTimelineWidth()
    })
  },
  { immediate: true }
)

const handleResetWords = async () => {
  if (!isWordSyncAvailable.value) return

  // Clear the words object entirely - this removes persisted word timings.
  emit('update:words', {
    lineIndex: props.selectedLineIndex,
    words: undefined,
  })

  // Reset selected boundary after clearing.
  resetBoundarySelection()

  // Wait for parent state to apply, then force segmentation so first reset is reliable.
  await nextTick()
  await loadDefaultSegmentation({ force: true })
}

const handleImportLrcFile = () => {
  emit('import-lrc-file')
}

const handleImportLyricsfile = () => {
  emit('import-lyricsfile')
}

const handleClearAllTimings = () => {
  emit('clear-all-timings')
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
    segmentedTokenTexts.value = null
    return
  }

  if (!wasAvailable) {
    syncLaneWindowToSelection()
    segmentedTokenTexts.value = null
    void loadDefaultSegmentation()
  }
})

watch(hasActualWords, (hasWords, hadWords) => {
  if (hadWords && !hasWords && isWordSyncAvailable.value) {
    segmentedTokenTexts.value = null
    void loadDefaultSegmentation()
  }
})

watch(
  () => props.selectedLine?.text,
  () => {
    segmentedTokenTexts.value = null
    if (isWordSyncAvailable.value) {
      void loadDefaultSegmentation()
    }
  }
)

watch(
  [lineStartMs, actualLineEndMs],
  () => {
    if (!props.hasSelectedLine || !props.selectedLine) {
      return
    }

    laneStartMs.value = lineStartMs.value
    laneEndMs.value = actualLineEndMs.value
  },
  { immediate: true }
)

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

<style scoped>
.dropdown-container {
  @apply p-1 min-w-[12rem];
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700 rounded cursor-pointer h-8 gap-2 w-full;
}

.dropdown-divider {
  @apply h-px bg-neutral-100 dark:bg-neutral-700 my-1;
}

.dropdown-label {
  @apply text-neutral-800 dark:text-neutral-300 text-sm font-bold;
}
</style>
