<template>
  <div class="grow overflow-hidden flex flex-col relative">
    <SyncedWordTimingLane
      class="relative z-20 shrink-0 mt-2"
      :selected-line="selectedLine"
      :has-selected-line="hasSelectedLine"
      :progress-ms="progressMs"
      :all-lines="modelValue"
      :selected-line-index="selectedLineIndex"
      :file-path="filePath"
      @update:words="handleWordsUpdate"
      @word-timing-edited="handleWordTimingEdited"
      @play-line="handlePlayLine"
      @play-line-at-offset="handlePlayLineAtOffset"
      @select-next-line="selectLine"
    />

    <div
      ref="linesListElement"
      class="flex-1 overflow-y-auto py-4 relative z-0 outline-none"
      tabindex="0"
      @mousemove="handleMouseMove"
      @mouseleave="handleLinesMouseLeave"
      @scroll="handleLinesScroll"
      @click="handleContainerClick"
      @keydown.esc="handleEscKey"
    >
      <div>
        <SyncedInsertButton
          title="Add line before first"
          :opacity="insertControlOpacity(0)"
          @click="handleInsertButtonClick(0, $event)"
        />

        <div v-for="(line, index) in modelValue" :key="line.id">
          <SyncedLyricsLineRow
            :ref="component => setLineRowRef(component, index)"
            :line="line"
            :index="index"
            :row-class="rowClass(index)"
            :is-line-controls-visible="isLineControlsVisible(index)"
            :end-timestamp-diff-direction="getEndTimestampDiffDirection(index)"
            :is-editing="editingLineIndex === index"
            :editing-text="editingText"
            :timestamp-text="formatTimestampMs(line.start_ms)"
            :end-timestamp-text="formatTimestampMs(line.end_ms)"
            :set-line-input-ref="setLineInputRef"
            :progress-ms="progressMs"
            :next-line-start-ms="nextLineStartMs(index)"
            @mouseenter="hoveredLineIndex = index"
            @mouseleave="hoveredLineIndex = null"
            @select="selectLine"
            @mousedown-line="startDragSelection"
            @play-line="handlePlayLine"
            @sync-line="handleSyncLine"
            @rewind-line="handleRewindLine"
            @forward-line="handleForwardLine"
            @sync-end="handleSyncEnd"
            @sync-end-to-next="handleSyncEndToNext"
            @rewind-end="handleRewindEnd"
            @forward-end="handleForwardEnd"
            @delete-line="handleDeleteLine"
            @start-edit="startEditingLine"
            @save-edit="saveEditingLine"
            @cancel-edit="cancelEditingLine"
            @update:editing-text="handleEditingTextUpdate"
          />

          <SyncedInsertButton
            v-if="index < modelValue.length - 1"
            title="Add line between"
            :opacity="insertControlOpacity(index + 1)"
            @click="handleInsertButtonClick(index + 1, $event)"
          />
        </div>

        <SyncedInsertButton
          title="Add line after last"
          :opacity="insertControlOpacity(modelValue.length)"
          @click="handleInsertButtonClick(modelValue.length, $event)"
        />
      </div>
    </div>

    <!-- Floating bulk actions toolbar -->
    <div
      v-if="hasMultiSelection"
      class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 shadow-lg"
    >
      <span class="text-xs text-neutral-500 dark:text-neutral-400 whitespace-nowrap">
        {{ selectedLineIndices.length }} lines selected
      </span>
      <div class="w-px h-4 bg-neutral-200 dark:bg-neutral-700" />
      <button
        class="button button-normal p-1 rounded-full text-xs h-6 w-6"
        title="Rewind selected lines by 100ms"
        @click="handleBulkRewind"
      >
        <Rewind />
      </button>
      <button
        class="button button-normal p-1 rounded-full text-xs h-6 w-6"
        title="Forward selected lines by 100ms"
        @click="handleBulkForward"
      >
        <Forward />
      </button>
      <div class="w-px h-4 bg-neutral-200 dark:bg-neutral-700" />
      <button
        class="button button-normal p-1 rounded-full text-xs h-6 w-6 text-red-500 dark:text-red-400"
        title="Delete selected lines"
        @click="handleBulkDelete"
      >
        <Trash />
      </button>
    </div>

    <SyncedLyricsEmptyState
      v-if="modelValue.length === 0"
      :can-import-from-plain="canImportFromPlain"
      @import-lines-from-plain="handleImportLinesFromPlain"
      @import-lrc-file="emit('import-lrc-file')"
      @paste-lrc="emit('paste-lrc')"
      @add-line-at="handleAddLineAt"
      @mark-as-instrumental="handleMarkAsInstrumental"
    />
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, toRef, watch } from 'vue'
import Rewind from '~icons/mdi/rewind'
import Forward from '~icons/mdi/fast-forward'
import Trash from '~icons/mdi/trash-can'
import SyncedInsertButton from '@/components/library/edit-lyrics-v2/SyncedInsertButton.vue'
import SyncedLyricsEmptyState from '@/components/library/edit-lyrics-v2/SyncedLyricsEmptyState.vue'
import SyncedLyricsLineRow from '@/components/library/edit-lyrics-v2/SyncedLyricsLineRow.vue'
import SyncedWordTimingLane from '@/components/library/edit-lyrics-v2/SyncedWordTimingLane.vue'
import { useEditLyricsV2SyncedInlineEditing } from '@/composables/edit-lyrics-v2/useEditLyricsV2SyncedInlineEditing.js'
import { useEditLyricsV2SyncedInsertHover } from '@/composables/edit-lyrics-v2/useEditLyricsV2SyncedInsertHover.js'
import { formatTimestampMs } from '@/utils/lyricsfile.js'

const props = defineProps({
  modelValue: {
    type: Array,
    required: true,
  },
  selectedLineIndex: {
    type: Number,
    default: -1,
  },
  selectedLineIndices: {
    type: Array,
    default: () => [],
  },
  canImportFromPlain: {
    type: Boolean,
    default: false,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
  filePath: {
    type: String,
    default: null,
  },
})

const emit = defineEmits([
  'update:modelValue',
  'update:selected-line-index',
  'update:selected-line-indices',
  'play-line',
  'play-line-at-offset',
  'sync-line',
  'rewind-line',
  'forward-line',
  'sync-end',
  'sync-end-to-next',
  'rewind-end',
  'forward-end',
  'delete-line',
  'bulk-rewind-lines',
  'bulk-forward-lines',
  'bulk-delete-lines',
  'add-line-at',
  'import-lines-from-plain',
  'import-lrc-file',
  'paste-lrc',
  'editing-state-change',
  'update:words',
  'word-timing-edited',
  'update-line-text',
  'mark-as-instrumental',
])

// Effective end of a line for overlap detection: prefer the line's own end_ms,
// otherwise fall back to the next line's start_ms. Returns null if neither
// is available — such a line can't overlap-detect cleanly so it's skipped.
const getLineEffectiveEndMs = (line, index) => {
  if (Number.isFinite(line?.end_ms)) return line.end_ms
  const nextLine = props.modelValue[index + 1]
  if (nextLine && Number.isFinite(nextLine.start_ms)) return nextLine.start_ms
  return null
}

// Indexes of every line that's part of an overlapping pair anywhere in the
// song. Relies on the invariant that lines are sorted by start_ms. 
// For each new line, if the maximum effective end seen so far reaches past 
// this line's start, both participate in an overlap. 
const overlappingLineIndexes = computed(() => {
  const overlaps = new Set()
  const lines = props.modelValue
  let maxEnd = -Infinity
  let maxEndIndex = -1

  for (let i = 0; i < lines.length; i++) {
    if (!Number.isFinite(lines[i]?.start_ms)) continue
    const endI = getLineEffectiveEndMs(lines[i], i)
    if (!Number.isFinite(endI)) continue

    if (maxEnd > lines[i].start_ms) {
      overlaps.add(maxEndIndex)
      overlaps.add(i)
    }

    if (endI > maxEnd) {
      maxEnd = endI
      maxEndIndex = i
    }
  }

  return overlaps
})

const nextLineStartMs = index => {
  const next = props.modelValue[index + 1]
  return next && Number.isFinite(next.start_ms) ? next.start_ms : null
}

const hoveredLineIndex = ref(null)
const linesListElement = ref(null)
const modelValue = toRef(props, 'modelValue')

// Multi-selection drag state
const isDragging = ref(false)
const dragAnchorIndex = ref(-1)
const didJustDrag = ref(false)
const dragStartPos = ref(null)

const selectedIndexSet = computed(() => new Set(props.selectedLineIndices))
const hasMultiSelection = computed(() => props.selectedLineIndices.length >= 2)

const isLineRowSelected = index => selectedIndexSet.value.has(index)

const startDragSelection = (index, event) => {
  if (event.ctrlKey || event.metaKey) {
    // Ctrl/Cmd+click toggles individual line
    emit('update:selected-line-indices', index)
    return
  }

  didJustDrag.value = false
  dragStartPos.value = { x: event.clientX, y: event.clientY }
  isDragging.value = true
  dragAnchorIndex.value = index
  emit('update:selected-line-indices', { start: index, end: index })
}

const updateDragSelection = index => {
  if (!isDragging.value) {
    return
  }
  if (index !== dragAnchorIndex.value) {
    didJustDrag.value = true
  }
  emit('update:selected-line-indices', { start: dragAnchorIndex.value, end: index })
}

const endDragSelection = () => {
  isDragging.value = false
  dragAnchorIndex.value = -1
  dragStartPos.value = null
}

const handleMouseMove = event => {
  if (isDragging.value && dragStartPos.value) {
    const dx = Math.abs(event.clientX - dragStartPos.value.x)
    const dy = Math.abs(event.clientY - dragStartPos.value.y)
    if (dx > 5 || dy > 5) {
      didJustDrag.value = true
    }
  }
  handleLinesMouseMove(event)
}

const handleContainerClick = event => {
  // If a drag just finished, ignore the click so it doesn't clear the selection
  if (didJustDrag.value) {
    didJustDrag.value = false
    return
  }
  // Clicking empty space in the container clears multi-selection,
  // but clicks on line rows or buttons are ignored here.
  if (!event.target.closest('.group, button, input')) {
    emit('update:selected-line-indices', { clear: true })
  }
}

const handleEscKey = () => {
  emit('update:selected-line-indices', { clear: true })
}

const handleDocumentMouseUp = () => {
  endDragSelection()
}

onMounted(() => {
  document.addEventListener('mouseup', handleDocumentMouseUp)
})

onUnmounted(() => {
  document.removeEventListener('mouseup', handleDocumentMouseUp)
})

const handleBulkRewind = () => {
  emit('bulk-rewind-lines', props.selectedLineIndices)
}

const handleBulkForward = () => {
  emit('bulk-forward-lines', props.selectedLineIndices)
}

const handleBulkDelete = () => {
  emit('bulk-delete-lines', props.selectedLineIndices)
}

const handleUpdateLineText = (lineIndex, newText) => {
  emit('update-line-text', lineIndex, newText)
}

const {
  editingLineIndex,
  editingText,
  setLineInputRef,
  startEditingLine,
  saveEditingLine,
  cancelEditingLine,
  handleLineCountChange: handleInlineEditingLineCountChange,
} = useEditLyricsV2SyncedInlineEditing({
  modelValue,
  emit,
  selectLine: index => emit('update:selected-line-index', index),
  updateLineText: handleUpdateLineText,
})

const {
  lineRowElements,
  setLineRowRef,
  insertControlOpacity,
  handleLinesMouseMove,
  handleLinesMouseLeave,
  handleLinesScroll,
  handleLineCountChange: handleInsertHoverLineCountChange,
} = useEditLyricsV2SyncedInsertHover({ modelValue })

const scrollLineIntoView = index => {
  if (!Number.isInteger(index) || index < 0) {
    return
  }

  if (!(linesListElement.value instanceof HTMLElement)) {
    return
  }

  const lineRowElement = lineRowElements.value[index]
  if (!(lineRowElement instanceof HTMLElement)) {
    return
  }

  lineRowElement.scrollIntoView({
    block: 'nearest',
    inline: 'nearest',
  })
}

const rowClass = index => {
  if (isLineRowSelected(index) || props.selectedLineIndex === index || editingLineIndex.value === index) {
    return 'bg-neutral-100 dark:bg-neutral-800'
  }

  if (hoveredLineIndex.value === index) {
    return 'bg-neutral-50 dark:bg-neutral-800/50'
  }

  // Dim warning color for every line that's part of an overlapping pair, so
  // overlap clusters in the song are visible at a glance.
  if (overlappingLineIndexes.value.has(index)) {
    return 'bg-amber-50 dark:bg-amber-900/20'
  }

  return 'bg-transparent'
}

const selectLine = index => {
  if (didJustDrag.value) {
    didJustDrag.value = false
    return
  }
  emit('update:selected-line-index', index)
}

const isLineControlsVisible = index =>
  hoveredLineIndex.value === index || props.selectedLineIndex === index || isLineRowSelected(index)

const getEndTimestampDiffDirection = index => {
  const lineEndMs = props.modelValue[index]?.end_ms
  const nextLineStartMs = props.modelValue[index + 1]?.start_ms

  if (!Number.isFinite(lineEndMs) || !Number.isFinite(nextLineStartMs) || lineEndMs === nextLineStartMs) {
    return null
  }

  return lineEndMs < nextLineStartMs ? 'before' : 'after'
}

const emitLineAction = (eventName, index, selectBefore = true) => {
  if (selectBefore) {
    selectLine(index)
  }

  emit(eventName, index)
}

const handlePlayLine = index => {
  emitLineAction('play-line', index, false)
}

const handlePlayLineAtOffset = payload => {
  emit('play-line-at-offset', payload)
}

const handleSyncLine = index => {
  emitLineAction('sync-line', index)
}

const handleRewindLine = index => {
  emitLineAction('rewind-line', index)
}

const handleForwardLine = index => {
  emitLineAction('forward-line', index)
}

const handleSyncEnd = index => {
  emitLineAction('sync-end', index)
}

const handleSyncEndToNext = index => {
  emitLineAction('sync-end-to-next', index)
}

const handleRewindEnd = index => {
  emitLineAction('rewind-end', index)
}

const handleForwardEnd = index => {
  emitLineAction('forward-end', index)
}

const handleDeleteLine = index => {
  emitLineAction('delete-line', index, false)
}

const handleAddLineAt = index => {
  emit('add-line-at', index)
}

const handleInsertButtonClick = (index, event) => {
  event.stopPropagation()
  handleAddLineAt(index)
}

const handleImportLinesFromPlain = () => {
  emit('import-lines-from-plain')
}

const handleMarkAsInstrumental = () => {
  emit('mark-as-instrumental')
}

const handleEditingTextUpdate = value => {
  editingText.value = value
}

const handleWordsUpdate = ({ lineIndex, words, lineStartMs }) => {
  emit('update:words', { lineIndex, words, lineStartMs })
}

const handleWordTimingEdited = ({ lineIndex, startMs }) => {
  emit('word-timing-edited', { lineIndex, startMs })
}

const hasSelectedLine = computed(
  () => props.selectedLineIndex >= 0 && props.selectedLineIndex < props.modelValue.length
)

const selectedLine = computed(() => {
  if (!hasSelectedLine.value) return null
  return props.modelValue[props.selectedLineIndex]
})

watch(
  () => props.modelValue.length,
  lineCount => {
    handleInsertHoverLineCountChange(lineCount)
    handleInlineEditingLineCountChange(lineCount)
  }
)

watch(
  () => props.selectedLineIndex,
  (selectedLineIndex, previousLineIndex) => {
    if (selectedLineIndex === previousLineIndex) {
      return
    }

    nextTick(() => {
      scrollLineIntoView(selectedLineIndex)
    })
  }
)

watch(hoveredLineIndex, newIndex => {
  if (isDragging.value && Number.isInteger(newIndex)) {
    updateDragSelection(newIndex)
  }
})
</script>
