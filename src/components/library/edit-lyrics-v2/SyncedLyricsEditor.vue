<template>
  <div class="grow overflow-hidden">
    <div
      class="h-full overflow-y-auto px-2 py-2"
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
          @click.stop="handleAddLineAt(0)"
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
            @click.stop="handleAddLineAt(index + 1)"
          />
        </template>

        <SyncedInsertButton
          title="Add line after last"
          :opacity="insertControlOpacity(modelValue.length)"
          @click.stop="handleAddLineAt(modelValue.length)"
        />
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, toRef, watch } from 'vue'
import SyncedInsertButton from '@/components/library/edit-lyrics-v2/SyncedInsertButton.vue'
import SyncedLyricsEmptyState from '@/components/library/edit-lyrics-v2/SyncedLyricsEmptyState.vue'
import SyncedLyricsLineRow from '@/components/library/edit-lyrics-v2/SyncedLyricsLineRow.vue'
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
  'editing-state-change'
])

const hoveredLineIndex = ref(null)
const modelValue = toRef(props, 'modelValue')

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
  selectLine: (index) => emit('update:selected-line-index', index)
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

const handleImportLinesFromPlain = () => {
  emit('import-lines-from-plain')
}

const handleEditingTextUpdate = (value) => {
  editingText.value = value
}

watch(() => props.modelValue.length, (lineCount) => {
  handleInsertHoverLineCountChange(lineCount)
  handleInlineEditingLineCountChange(lineCount)
})
</script>
