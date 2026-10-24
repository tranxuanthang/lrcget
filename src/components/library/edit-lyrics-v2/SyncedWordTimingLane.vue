<template>
  <div
    class="relative z-20 flex flex-col px-2 py-2 rounded-lg overflow-visible transition-[height] duration-200 ease-out"
    :class="[
      hasSelectedLine ? 'bg-neutral-100 dark:bg-neutral-800' : 'bg-white dark:bg-neutral-950',
      laneHeightClass,
    ]"
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
          <div v-if="filePath" class="flex items-center">
            <button
              class="button button-normal h-6 w-6 rounded-l-full rounded-r-none border-r border-neutral-300 dark:border-neutral-700 flex items-center justify-center"
              :title="spectrogramVisible ? 'Hide spectrogram' : 'Show spectrogram'"
              @click="toggleSpectrogramVisible"
            >
              <Waveform v-if="spectrogramVisible" class="w-3.5 h-3.5" />
              <EyeOff v-else class="w-3.5 h-3.5" />
            </button>

            <VDropdown theme="lrcget-dropdown" placement="bottom-end">
              <button
                type="button"
                class="button button-normal h-6 w-5 rounded-r-full rounded-l-none flex items-center justify-center"
                title="Spectrogram colour theme"
                aria-label="Spectrogram colour theme"
                aria-haspopup="menu"
              >
                <ChevronDown class="w-3 h-3" />
              </button>

              <template #popper>
                <div class="theme-menu" role="menu" aria-label="Spectrogram colour theme">
                  <button
                    v-for="theme in SPECTROGRAM_THEMES"
                    :key="theme.id"
                    v-close-popper
                    type="button"
                    role="menuitemradio"
                    :aria-checked="isSpectrogramThemeSelected(theme.id)"
                    :aria-label="theme.label"
                    :title="theme.label"
                    class="theme-option"
                    :class="{ 'theme-option-selected': isSpectrogramThemeSelected(theme.id) }"
                    @click="handleSelectSpectrogramTheme(theme.id)"
                  >
                    <span
                      class="h-4 w-4 shrink-0 rounded-full ring-1 ring-black/10 dark:ring-white/15"
                      :style="{ backgroundColor: theme.swatchColor }"
                      aria-hidden="true"
                    />
                  </button>
                </div>
              </template>
            </VDropdown>
          </div>
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
        </div>
      </div>

      <div class="relative flex flex-col flex-1 min-h-0">
      <SpectrogramPanel
        v-if="filePath && spectrogramVisible"
        :file-path="filePath"
        :start-ms="laneStartMs"
        :end-ms="laneEndMs"
        :theme="spectrogramTheme"
        class="mb-2 shrink-0"
        @seek="$emit('seek', $event)"
      />

      <!-- Timeline with word segments -->
      <div
        ref="timelineElement"
        class="relative flex-1 bg-white dark:bg-neutral-900 rounded border border-neutral-300 dark:border-neutral-600 transition-opacity duration-200 cursor-pointer"
        :class="{ 'opacity-50': !hasActualWords }"
        @pointerdown="handleTimelinePointerDown"
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
          :drag-active="isBoundaryDragActive"
          @split-at="handleSegmentSplitAt"
        />

        <button
          v-for="index in interiorBoundaryIndexes"
          :key="`boundary-${index}`"
          type="button"
          class="group absolute top-0 bottom-0 z-30 -ml-2 w-4 cursor-ew-resize bg-transparent"
          :style="{ left: `${timeToPercent(displayedWords[index].start_ms)}%` }"
          :title="`Adjust start of ${displayedWords[index].text}`"
          @pointerdown="handleBoundaryPointerDown(index, $event)"
          @click.stop="selectBoundary(index, $event)"
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

        <div
          v-if="endDragState"
          class="absolute inset-y-0 z-20 pointer-events-none"
          :style="{ left: `${timeToPercent(endDragState.currentEndMs)}%` }"
        >
          <div
            class="absolute top-0 bottom-0 w-[3px] -translate-x-1/2 bg-neutral-600 dark:bg-neutral-300 ring-1 ring-neutral-500/25"
          />
          <div
            class="absolute top-[-0.375rem] left-0 -translate-x-1/2 -translate-y-full px-[0.4rem] py-0.5 rounded-full text-xs leading-4 whitespace-nowrap text-neutral-800 bg-neutral-200 dark:text-white dark:bg-hoa-1100"
          >
            {{ formatTimestampMs(endDragState.currentEndMs) }}
          </div>
        </div>

        <button
          v-if="displayedWords.length > 0"
          type="button"
          class="button button-primary absolute left-0 top-0 bottom-0 z-40 w-3.5 p-0 rounded-l cursor-ew-resize"
          title="Nudge line start earlier (drag to trim)"
          @pointerdown="handleBoundaryPointerDown(0, $event)"
          @click.stop="handleExpandStartClick"
        >
          <ChevronLeft class="w-3 h-3" />
        </button>

        <button
          v-if="displayedWords.length > 0"
          type="button"
          class="button button-primary absolute right-0 top-0 bottom-0 z-40 w-3.5 p-0 rounded-r cursor-ew-resize"
          title="Nudge line end later (drag to trim)"
          @pointerdown="handleEndPointerDown"
          @click.stop="handleExpandEndClick"
        >
          <ChevronRight class="w-3 h-3" />
        </button>

      </div>

      <!-- Playhead is a sibling of the spectrogram + timeline so its top
           sits just above the spectrogram (when visible) instead of just
           above the timeline. -->
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
import Waveform from '~icons/mdi/waveform'
import EyeOff from '~icons/mdi/eye-off'
import ChevronDown from '~icons/mdi/chevron-down'
import ChevronLeft from '~icons/mdi/chevron-left'
import ChevronRight from '~icons/mdi/chevron-right'
import { useGlobalState } from '@/composables/global-state.js'
import SpectrogramPanel from '@/components/library/edit-lyrics-v2/SpectrogramPanel.vue'
import SyncedWordTimingSegment from '@/components/library/edit-lyrics-v2/SyncedWordTimingSegment.vue'
import { useEditLyricsV2WordBoundaryDrag } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordBoundaryDrag.js'
import { useEditLyricsV2LineEndDrag } from '@/composables/edit-lyrics-v2/useEditLyricsV2LineEndDrag.js'
import { useEditLyricsV2WordTimingHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2WordTimingHotkeys.js'
import { resolveWordSplitTimeMs } from '@/utils/word-split.js'
import {
  syncedEditorShortcutBindings,
  wordTimingShortcutBindings,
  withShortcutTitle,
} from '@/composables/edit-lyrics-v2/shortcutRegistry.js'
import { formatTimestampMs } from '@/utils/lyricsfile.js'
import { SPECTROGRAM_THEMES } from '@/utils/spectrogram-themes.js'
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
  filePath: {
    type: String,
    default: null,
  },
})

const emit = defineEmits([
  'update:words',
  'word-timing-edited',
  'play-line',
  'select-next-line',
  'seek',
  'rewind-line',
  'forward-end',
  'set-line-end',
])

const {
  spectrogramVisible,
  toggleSpectrogramVisible,
  showSpectrogram,
  spectrogramTheme,
  selectSpectrogramTheme,
} = useGlobalState()

const isSpectrogramThemeSelected = themeId => spectrogramTheme.value === themeId

const handleSelectSpectrogramTheme = async themeId => {
  await selectSpectrogramTheme(themeId)
  await showSpectrogram()
}

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

const hasSpectrogramSlot = computed(
  () => !!props.filePath && isWordSyncAvailable.value && spectrogramVisible.value
)

// Three-way lane height so collapsing the spectrogram doesn't squash the
// word-timing timeline: tall with spectrogram, mid-tall in word-sync without
// spectrogram (preserves timeline size), compact otherwise.
const laneHeightClass = computed(() => {
  if (hasSpectrogramSlot.value) return 'h-[13rem]'
  if (isWordSyncAvailable.value) return 'h-[7rem]'
  return 'h-[5rem]'
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
  isDraggingBoundary,
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

const { endDragState, isDraggingEnd, startEndDrag, cancelEndDrag } = useEditLyricsV2LineEndDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  lineEndMs: laneEndMs,
  selectedLineIndex: computed(() => props.selectedLineIndex),
  onCommitLineEnd: payload => emit('set-line-end', payload),
})

const interiorBoundaryIndexes = computed(() =>
  boundaryIndexes.value.filter(index => index > 0)
)

const isBoundaryDragActive = computed(() => Boolean(dragState.value) || Boolean(endDragState.value))

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
    // While trimming the right edge, the last word follows the drag preview so
    // its segment shrinks live before the commit lands.
    if (endDragState.value) {
      return endDragState.value.currentEndMs
    }
    return laneEndMs.value
  }

  const nextWordStart = displayedWords.value[index + 1]?.start_ms
  return Number.isFinite(nextWordStart) ? nextWordStart : laneEndMs.value
}

// Click-to-seek is decided by where the press started, not by drag state: a
// commit's own reactive update fires the selectedLine watcher, which cancels
// the drag and clears the isDragging flags in a microtask, before the click is
// dispatched. Boundary/arrow pointerdowns stopPropagation, so only a press that
// reaches the timeline arms it.
const isTimelineClickArmed = ref(false)

const handleTimelinePointerDown = () => {
  isTimelineClickArmed.value = true
}

const handleBoundaryPointerDown = (rightWordIndex, event) => {
  isTimelineClickArmed.value = false
  startBoundaryDrag(rightWordIndex, event, clientXToTime)
}

const handleEndPointerDown = event => {
  isTimelineClickArmed.value = false
  startEndDrag(event, clientXToTime)
}

const handleExpandStartClick = () => {
  if (isDraggingBoundary.value || !isWordSyncAvailable.value) {
    return
  }
  emit('rewind-line', props.selectedLineIndex)
}

const handleExpandEndClick = () => {
  if (isDraggingEnd.value || !isWordSyncAvailable.value) {
    return
  }
  emit('forward-end', props.selectedLineIndex)
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

const handleSegmentSplitAt = ({ wordIndex, splitIndex, splitRatio, splitClientX }) => {
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
  // Prefer the highlighted divider's actual position on the lane: map its
  // client-X through the timeline's time scale so the split lands under the
  // highlight the user clicked. This avoids the trailing-space failure where a
  // zero-width trailing space drives the width ratio to ~1.0 and pins the new
  // segment onto the word's end bound (a zero-length segment). Fall back to the
  // width ratio when no client-X is available (non-text-node measurement path).
  const ratioTimeMs = Math.round(wordStartMs + (wordEndMs - wordStartMs) * normalizedSplitRatio)
  const highlightTimeMs = Number.isFinite(splitClientX) ? clientXToTime(splitClientX) : null
  const splitTimeMs = resolveWordSplitTimeMs({
    wordStartMs,
    wordEndMs,
    highlightTimeMs,
    ratioTimeMs,
  })

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

// Watch the selected line's identity AND its timestamps so that the lane window refreshes on deletions and changes.
watch(
  [
    () => props.selectedLineIndex,
    () => props.selectedLine?.start_ms,
    () => props.selectedLine?.end_ms,
  ],
  ([newIndex], [oldIndex] = []) => {
    cancelBoundaryInteraction()
    cancelEndDrag()
    // Only reset boundary index when actually moving to a different line.
    if (newIndex !== oldIndex) {
      resetBoundarySelection()
    }
    syncLaneWindowToSelection()
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

const handleTimelineClick = event => {
  event.stopPropagation()
  // Releasing a boundary/arrow drag dispatches its click on the timeline (the
  // common ancestor of the button and the release target), so the button's own
  // @click.stop never sees it. Only seek when the press started on the lane.
  if (!isTimelineClickArmed.value) return
  isTimelineClickArmed.value = false
  if (laneEndMs.value <= laneStartMs.value) return
  const rect = timelineElement.value?.getBoundingClientRect()
  if (!rect || rect.width <= 0) return
  const fraction = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width))
  const timeMs = laneStartMs.value + fraction * (laneEndMs.value - laneStartMs.value)
  emit('seek', timeMs / 1000)
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
    cancelEndDrag()
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
    if (endDragState.value) {
      cancelEndDrag()
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
  cancelEndDrag()
  window.removeEventListener('resize', updateTimelineWidth)
  unbindWordTimingHotkeys()
})
</script>

<style scoped>
.theme-menu {
  @apply w-11 p-1;
}

.theme-option {
  @apply flex h-8 w-full items-center justify-center rounded hover:bg-neutral-100 dark:hover:bg-neutral-700;
}

.theme-option-selected {
  @apply bg-hoa-100/60 hover:bg-hoa-100 dark:bg-hoa-1100/15 dark:hover:bg-hoa-1100/25;
}

.theme-option:focus-visible {
  @apply outline-none ring-2 ring-inset ring-hoa-1100;
}
</style>
