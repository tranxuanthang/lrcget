<template>
  <div class="grow overflow-hidden">
    <div
      class="h-full overflow-y-auto px-2 py-2"
      @mousemove="handleLinesMouseMove"
      @mouseleave="handleLinesMouseLeave"
      @scroll="handleLinesScroll"
    >
      <div
        v-if="modelValue.length === 0"
        class="h-full flex items-center justify-center px-3"
      >
        <div class="w-full max-w-lg rounded-lg border border-brave-90 dark:border-brave-20 bg-brave-98 dark:bg-brave-10 p-5">
          <h3 class="text-base font-semibold">No synced lyric lines yet</h3>
          <p class="text-sm text-brave-45 dark:text-brave-70 mt-1">
            Import from plain lyrics or add a new synced line manually.
          </p>

          <div class="mt-4 flex flex-wrap gap-2">
            <button
              class="button button-normal px-8 py-2 rounded-full"
              :class="canImportFromPlain ? '' : 'button-disabled'"
              :disabled="!canImportFromPlain"
              @click="handleImportLinesFromPlain"
            >
              Import from plain lyrics
            </button>
            <button
              class="button button-normal px-8 py-2 rounded-full"
              @click="handleAddLineAt(0)"
            >
              Add line manually
            </button>
          </div>
        </div>
      </div>

      <template v-else>
        <div class="relative h-0">
          <button
            class="button button-normal absolute left-1/2 -translate-x-1/2 -translate-y-1/2 p-0.5 rounded-full text-xs"
            title="Add line before first"
            :style="{ opacity: insertControlOpacity(0) }"
            :class="insertControlOpacity(0) > 0.05 ? '' : 'pointer-events-none'"
            @click.stop="handleAddLineAt(0)"
          >
            <Plus />
          </button>
        </div>

        <template
          v-for="(line, index) in modelValue"
          :key="index"
        >
          <div
            class="group flex items-center gap-2 px-2 py-1.5 rounded-md transition-colors h-9"
            :class="rowClass(index)"
            :ref="(element) => setLineRowRef(element, index)"
            @mouseenter="hoveredLineIndex = index"
            @mouseleave="hoveredLineIndex = null"
            @click="selectLine(index)"
          >
            <div class="flex items-center gap-1 w-[7.5rem]">
              <button
                v-show="isLineControlsVisible(index)"
                class="button button-normal p-1 rounded-full text-sm"
                title="Play line"
                @click.stop="handlePlayLine(index)"
              >
                <Play />
              </button>
              <button
                v-show="isLineControlsVisible(index)"
                class="button button-normal p-1 rounded-full text-sm"
                title="Sync line to current playback"
                @click.stop="handleSyncLine(index)"
              >
                <Equal />
              </button>
              <button
                v-show="isLineControlsVisible(index)"
                class="button button-normal p-1 rounded-full text-sm"
                title="Rewind line by 100ms"
                @click.stop="handleRewindLine(index)"
              >
                <Minus />
              </button>
              <button
                v-show="isLineControlsVisible(index)"
                class="button button-normal p-1 rounded-full text-sm"
                title="Forward line by 100ms"
                @click.stop="handleForwardLine(index)"
              >
                <Plus />
              </button>
            </div>

            <div
              class="flex-none px-2 py-0.5 text-xs font-mono rounded-full bg-brave-90 dark:bg-brave-20 text-brave-25 dark:text-brave-99 min-w-[5.75rem] text-center"
              :class="{ 'font-bold': isLinePlaying(index) }"
            >
              {{ formatTimestampMs(line.start_ms) }}
            </div>

            <input
              v-if="editingLineIndex === index"
              :ref="setLineInputRef"
              v-model="editingText"
              class="grow h-full px-2 rounded-md border border-brave-80 dark:border-brave-25 bg-brave-100 dark:bg-brave-10 outline-none"
              :class="{ 'font-bold': isLinePlaying(index) }"
              @blur="saveEditingLine"
              @keydown.enter.prevent="saveEditingLine"
              @keydown.esc.prevent="cancelEditingLine"
            >

            <div
              v-else
              class="grow min-h-7 flex items-center px-2 rounded-md cursor-text"
              :class="{ 'font-bold': isLinePlaying(index) }"
              @click="selectLine(index)"
              @dblclick="startEditingLine(index)"
            >
              {{ line.text || ' ' }}
            </div>

            <button
              v-show="isLineControlsVisible(index)"
              class="button button-normal p-1 rounded-full text-sm"
              title="Delete line"
              @click.stop="handleDeleteLine(index)"
            >
              <Close />
            </button>
          </div>

          <div
            v-if="index < modelValue.length - 1"
            class="relative h-0"
          >
            <button
              class="button button-normal absolute left-1/2 -translate-x-1/2 -translate-y-1/2 p-0.5 rounded-full text-xs"
              title="Add line between"
              :style="{ opacity: insertControlOpacity(index + 1) }"
              :class="insertControlOpacity(index + 1) > 0.05 ? '' : 'pointer-events-none'"
              @click.stop="handleAddLineAt(index + 1)"
            >
              <Plus />
            </button>
          </div>
        </template>

        <div class="relative h-0">
          <button
            class="button button-normal absolute left-1/2 -translate-x-1/2 -translate-y-1/2 p-0.5 rounded-full text-xs"
            title="Add line after last"
            :style="{ opacity: insertControlOpacity(modelValue.length) }"
            :class="insertControlOpacity(modelValue.length) > 0.05 ? '' : 'pointer-events-none'"
            @click.stop="handleAddLineAt(modelValue.length)"
          >
            <Plus />
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { nextTick, ref, watch } from 'vue'
import { Play, Equal, Minus, Plus, Close } from 'mdue'
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

const editingLineIndex = ref(null)
const editingText = ref('')
const lineInput = ref(null)
const lineRowElements = ref([])
const hoveredLineIndex = ref(null)
const hoveredInsertIndex = ref(-1)
const hoveredInsertOpacity = ref(0)
const lastMouseClientY = ref(null)

const setLineInputRef = (element) => {
  lineInput.value = element
}

const setLineRowRef = (element, index) => {
  if (!element) {
    return
  }

  lineRowElements.value[index] = element
}

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

const getInsertCenterY = (insertIndex) => {
  const lineCount = props.modelValue.length
  if (lineCount === 0) {
    return null
  }

  if (insertIndex === 0) {
    return lineRowElements.value[0]?.getBoundingClientRect().top ?? null
  }

  if (insertIndex === lineCount) {
    return lineRowElements.value[lineCount - 1]?.getBoundingClientRect().bottom ?? null
  }

  const previousRow = lineRowElements.value[insertIndex - 1]
  const nextRow = lineRowElements.value[insertIndex]
  if (!previousRow || !nextRow) {
    return null
  }

  const previousRect = previousRow.getBoundingClientRect()
  const nextRect = nextRow.getBoundingClientRect()
  return (previousRect.bottom + nextRect.top) / 2
}

const updateHoveredInsertByMouseY = (mouseClientY) => {
  if (!Number.isFinite(mouseClientY) || props.modelValue.length < 1) {
    hoveredInsertIndex.value = -1
    hoveredInsertOpacity.value = 0
    return
  }

  const maxDistancePx = 26
  let nearestInsertIndex = -1
  let nearestDistance = Number.POSITIVE_INFINITY

  for (let insertIndex = 0; insertIndex <= props.modelValue.length; insertIndex += 1) {
    const centerY = getInsertCenterY(insertIndex)
    if (!Number.isFinite(centerY)) {
      continue
    }

    const distance = Math.abs(mouseClientY - centerY)
    if (distance < nearestDistance) {
      nearestDistance = distance
      nearestInsertIndex = insertIndex
    }
  }

  if (nearestDistance > maxDistancePx || nearestInsertIndex === -1) {
    hoveredInsertIndex.value = -1
    hoveredInsertOpacity.value = 0
    return
  }

  hoveredInsertIndex.value = nearestInsertIndex
  hoveredInsertOpacity.value = Math.max(0, 1 - (nearestDistance / maxDistancePx))
}

const insertControlOpacity = (insertIndex) => {
  if (hoveredInsertIndex.value !== insertIndex) {
    return 0
  }

  return hoveredInsertOpacity.value
}

const handleLinesMouseMove = (event) => {
  lastMouseClientY.value = event.clientY
  updateHoveredInsertByMouseY(event.clientY)
}

const handleLinesMouseLeave = () => {
  lastMouseClientY.value = null
  hoveredInsertIndex.value = -1
  hoveredInsertOpacity.value = 0
}

const handleLinesScroll = () => {
  if (!Number.isFinite(lastMouseClientY.value)) {
    return
  }

  updateHoveredInsertByMouseY(lastMouseClientY.value)
}

const handlePlayLine = (index) => {
  selectLine(index)
  emit('play-line', index)
}

const handleSyncLine = (index) => {
  selectLine(index)
  emit('sync-line', index)
}

const handleRewindLine = (index) => {
  selectLine(index)
  emit('rewind-line', index)
}

const handleForwardLine = (index) => {
  selectLine(index)
  emit('forward-line', index)
}

const handleDeleteLine = (index) => {
  emit('delete-line', index)
}

const handleAddLineAt = (index) => {
  emit('add-line-at', index)
}

const handleImportLinesFromPlain = () => {
  emit('import-lines-from-plain')
}

const startEditingLine = (index) => {
  selectLine(index)
  editingLineIndex.value = index
  editingText.value = props.modelValue[index]?.text || ''
  emit('editing-state-change', true)

  nextTick(() => {
    lineInput.value?.focus()
    lineInput.value?.select()
  })
}

const saveEditingLine = () => {
  if (editingLineIndex.value === null) {
    return
  }

  const nextLines = props.modelValue.map((line, index) => {
    if (index !== editingLineIndex.value) {
      return line
    }

    return {
      ...line,
      text: editingText.value
    }
  })

  emit('update:modelValue', nextLines)
  editingLineIndex.value = null
  editingText.value = ''
  emit('editing-state-change', false)
}

const cancelEditingLine = () => {
  editingLineIndex.value = null
  editingText.value = ''
  emit('editing-state-change', false)
}

watch(() => props.modelValue.length, (lineCount) => {
  lineRowElements.value = lineRowElements.value.slice(0, lineCount)

  if (editingLineIndex.value === null) {
    if (lineCount < 1) {
      hoveredInsertIndex.value = -1
      hoveredInsertOpacity.value = 0
    }

    if (Number.isFinite(lastMouseClientY.value)) {
      nextTick(() => {
        updateHoveredInsertByMouseY(lastMouseClientY.value)
      })
    }

    return
  }

  if (editingLineIndex.value >= lineCount) {
    cancelEditingLine()
  }
})
</script>
