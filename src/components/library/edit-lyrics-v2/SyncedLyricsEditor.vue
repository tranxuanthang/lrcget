<template>
  <div class="grow overflow-hidden flex flex-col">
    <SyncedWordTimingLane
      class="shrink-0 mt-2"
      :selected-line="selectedLine"
      :has-selected-line="hasSelectedLine"
      :progress-ms="progressMs"
      :all-lines="modelValue"
      :selected-line-index="selectedLineIndex"
      @update:words="handleWordsUpdate"
      @word-timing-edited="handleWordTimingEdited"
      @play-line="handlePlayLine"
      @select-next-line="selectLine"
    />

    <div
      ref="linesListElement"
      class="flex-1 overflow-y-auto py-4 relative"
      @mousemove="handleLinesMouseMove"
      @mouseleave="handleLinesMouseLeave"
      @scroll="handleLinesScroll"
    >
      <div>
        <SyncedInsertButton
          title="Add line before first"
          :opacity="insertControlOpacity(0)"
          @click="handleInsertButtonClick(0, $event)"
        />

        <div v-for="(line, index) in modelValue" :key="index">
          <SyncedLyricsLineRow
            :ref="component => setLineRowRef(component, index)"
            :line="line"
            :index="index"
            :row-class="rowClass(index)"
            :is-line-controls-visible="isLineControlsVisible(index)"
            :is-line-playing="isLinePlaying(index)"
            :is-editing="editingLineIndex === index"
            :editing-text="editingText"
            :timestamp-text="formatTimestampMs(line.start_ms)"
            :set-line-input-ref="setLineInputRef"
            :progress-ms="progressMs"
            @mouseenter="hoveredLineIndex = index"
            @mouseleave="hoveredLineIndex = null"
            @select="selectLine"
            @play-line="handlePlayLine"
            @sync-line="handleSyncLine"
            @rewind-line="handleRewindLine"
            @forward-line="handleForwardLine"
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

    <SyncedLyricsEmptyState
      v-if="modelValue.length === 0"
      :can-import-from-plain="canImportFromPlain"
      @import-lines-from-plain="handleImportLinesFromPlain"
      @add-line-at="handleAddLineAt"
      @mark-as-instrumental="handleMarkAsInstrumental"
    />
  </div>
</template>

<script setup>
import { computed, nextTick, ref, toRef, watch } from 'vue'
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
  playingLineIndex: {
    type: Number,
    default: -1,
  },
  canImportFromPlain: {
    type: Boolean,
    default: false,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
})

const emit = defineEmits([
  'update:modelValue',
  'update:selected-line-index',
  'play-line',
  'sync-line',
  'rewind-line',
  'forward-line',
  'delete-line',
  'add-line-at',
  'import-lines-from-plain',
  'editing-state-change',
  'update:words',
  'word-timing-edited',
  'update-line-text',
  'mark-as-instrumental',
])

const hoveredLineIndex = ref(null)
const linesListElement = ref(null)
const modelValue = toRef(props, 'modelValue')

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
  if (props.selectedLineIndex === index || editingLineIndex.value === index) {
    return 'bg-brave-95 dark:bg-brave-10/60'
  }

  if (hoveredLineIndex.value === index) {
    return 'bg-brave-98 dark:bg-brave-10/30'
  }

  return 'bg-transparent'
}

const selectLine = index => {
  emit('update:selected-line-index', index)
}

const isLineControlsVisible = index =>
  hoveredLineIndex.value === index || props.selectedLineIndex === index

const isLinePlaying = index => props.playingLineIndex === index

const emitLineAction = (eventName, index, selectBefore = true) => {
  if (selectBefore) {
    selectLine(index)
  }

  emit(eventName, index)
}

const handlePlayLine = index => {
  emitLineAction('play-line', index)
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
</script>
