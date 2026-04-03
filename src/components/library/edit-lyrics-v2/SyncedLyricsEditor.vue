<template>
  <div class="grow overflow-hidden">
    <div class="h-full overflow-y-auto px-2 py-2">
      <div
        v-for="(line, index) in modelValue"
        :key="index"
        class="group flex items-center gap-2 px-2 py-1.5 rounded-md transition-colors h-9"
        :class="rowClass(index)"
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
      </div>
    </div>
  </div>
</template>

<script setup>
import { nextTick, ref } from 'vue'
import { Play, Equal, Minus, Plus } from 'mdue'
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
  }
})

const emit = defineEmits([
  'update:modelValue',
  'update:selected-line-index',
  'play-line',
  'sync-line',
  'rewind-line',
  'forward-line',
  'editing-state-change'
])

const editingLineIndex = ref(null)
const editingText = ref('')
const lineInput = ref(null)
const hoveredLineIndex = ref(null)

const setLineInputRef = (element) => {
  lineInput.value = element
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
</script>
