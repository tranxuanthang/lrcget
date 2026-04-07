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
    />

    <div
      class="flex-1 overflow-y-auto py-2"
      @mousemove="handleLinesMouseMove"
      @mouseleave="handleLinesMouseLeave"
      @scroll="handleLinesScroll"
    >
      <SyncedLyricsEmptyState
        v-if="modelValue.length === 0"
        :can-import-from-plain="canImportFromPlain"
        @import-lines-from-plain="handleImportLinesFromPlain"
        @add-line-at="handleAddLineAt"
      />

      <template v-else>
        <SyncedInsertButton
          title="Add line before first"
          :opacity="insertControlOpacity(0)"
          @click="handleInsertButtonClick(0, $event)"
        />

        <template
          v-for="(line, index) in modelValue"
          :key="index"
        >
          <SyncedLyricsLineRow
            :ref="(component) => setLineRowRef(component, index)"
            :line="line"
            :index="index"
            :row-class="rowClass(index)"
            :is-line-controls-visible="isLineControlsVisible(index)"
            :is-line-playing="isLinePlaying(index)"
            :is-editing="editingLineIndex === index"
            :editing-text="editingText"
            :timestamp-text="formatTimestampMs(line.start_ms)"
            :set-line-input-ref="setLineInputRef"
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
        </template>

        <SyncedInsertButton
          title="Add line after last"
          :opacity="insertControlOpacity(modelValue.length)"
          @click="handleInsertButtonClick(modelValue.length, $event)"
        />
      </template>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, toRef, watch } from 'vue'
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
    required: true
  },
  selectedLineIndex: {
    type: Number,
    default: -1
  },
  playingLineIndex: {
    type: Number,
    default: -1
  },
  canImportFromPlain: {
    type: Boolean,
    default: false
  },
  progressMs: {
    type: Number,
    default: 0
  }
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
  'update-line-text'
])

const hoveredLineIndex = ref(null)
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
  handleLineCountChange: handleInlineEditingLineCountChange
} = useEditLyricsV2SyncedInlineEditing({
  modelValue,
  emit,
  selectLine: (index) => emit('update:selected-line-index', index),
  updateLineText: handleUpdateLineText
})

const {
  setLineRowRef,
  insertControlOpacity,
  handleLinesMouseMove,
  handleLinesMouseLeave,
  handleLinesScroll,
  handleLineCountChange: handleInsertHoverLineCountChange
} = useEditLyricsV2SyncedInsertHover({ modelValue })

const rowClass = (index) => {
  if (props.selectedLineIndex === index || editingLineIndex.value === index) {
    return 'bg-brave-95 dark:bg-brave-10/60'
  }

  if (hoveredLineIndex.value === index) {
    return 'bg-brave-98 dark:bg-brave-10/30'
  }

  return 'bg-transparent'
}

const selectLine = (index) => {
  emit('update:selected-line-index', index)
}

const isLineControlsVisible = (index) => (
  hoveredLineIndex.value === index || props.selectedLineIndex === index
)

const isLinePlaying = (index) => props.playingLineIndex === index

const emitLineAction = (eventName, index, selectBefore = true) => {
  if (selectBefore) {
    selectLine(index)
  }

  emit(eventName, index)
}

const handlePlayLine = (index) => {
  emitLineAction('play-line', index)
}

const handleSyncLine = (index) => {
  emitLineAction('sync-line', index)
}

const handleRewindLine = (index) => {
  emitLineAction('rewind-line', index)
}

const handleForwardLine = (index) => {
  emitLineAction('forward-line', index)
}

const handleDeleteLine = (index) => {
  emitLineAction('delete-line', index, false)
}

const handleAddLineAt = (index) => {
  emit('add-line-at', index)
}

const handleInsertButtonClick = (index, event) => {
  event.stopPropagation()
  handleAddLineAt(index)
}

const handleImportLinesFromPlain = () => {
  emit('import-lines-from-plain')
}

const handleEditingTextUpdate = (value) => {
  editingText.value = value
}

const handleWordsUpdate = ({ lineIndex, words }) => {
  emit('update:words', { lineIndex, words })
}

const handleWordTimingEdited = ({ lineIndex, startMs }) => {
  emit('word-timing-edited', { lineIndex, startMs })
}

const hasSelectedLine = computed(() => props.selectedLineIndex >= 0 && props.selectedLineIndex < props.modelValue.length)

const selectedLine = computed(() => {
  if (!hasSelectedLine.value) return null
  return props.modelValue[props.selectedLineIndex]
})

watch(() => props.modelValue.length, (lineCount) => {
  handleInsertHoverLineCountChange(lineCount)
  handleInlineEditingLineCountChange(lineCount)
})
</script>
