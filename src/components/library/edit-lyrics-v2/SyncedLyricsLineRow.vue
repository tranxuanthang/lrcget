<template>
  <div
    ref="rowElement"
    class="group flex items-center gap-2 px-4 py-1.5 rounded-md transition-colors h-9"
    :class="rowClass"
    @mouseenter="emit('mouseenter', index)"
    @mouseleave="emit('mouseleave')"
    @click="emit('select', index)"
  >
    <div class="flex items-center gap-1 w-[3.5rem]">
      <button
        v-show="isLineControlsVisible"
        class="button p-1 rounded-full text-sm h-6 w-6"
        :class="line.start_ms ? 'button-normal' : 'button-disabled'"
        :disabled="!line.start_ms"
        title="Play line"
        @click.stop="emit('play-line', index)"
      >
        <Play />
      </button>
      <button
        v-show="isLineControlsVisible"
        class="button button-primary p-1 rounded-full text-sm h-6 w-6"
        title="Sync line to current playback"
        @click.stop="emit('sync-line', index)"
      >
        <Equal />
      </button>
    </div>

    <div class="relative flex-none">
      <button
        v-show="isLineControlsVisible && line.start_ms"
        class="button p-0.5 rounded-full text-xs h-5 w-5 bg-hoa-100 dark:bg-hoa-1500 text-hoa-800 dark:text-hoa-200 absolute -left-1.5 top-1/2 -translate-y-1/2 z-10"
        title="Rewind line by 100ms"
        @click.stop="emit('rewind-line', index)"
      >
        <Minus />
      </button>
      <div
        class="px-3 py-0.5 text-xs font-mono rounded-full bg-hoa-100 dark:bg-hoa-1500 text-hoa-1300 dark:text-hoa-200 min-w-[5.75rem] text-center"
        :class="{ 'font-bold': isLinePlaying }"
      >
        {{ timestampText }}
      </div>
      <button
        v-show="isLineControlsVisible && line.start_ms"
        class="button p-0.5 rounded-full text-xs h-5 w-5 bg-hoa-100 dark:bg-hoa-1500 text-hoa-800 dark:text-hoa-200 absolute -right-1.5 top-1/2 -translate-y-1/2 z-10"
        title="Forward line by 100ms"
        @click.stop="emit('forward-line', index)"
      >
        <Plus />
      </button>
    </div>

    <input
      v-if="isEditing"
      :ref="setLineInputRef"
      v-model="editingTextProxy"
      class="grow h-full px-2 rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 outline-none"
      :class="{ 'font-bold': isLinePlaying }"
      @blur="emit('save-edit')"
      @keydown.enter.prevent="emit('save-edit')"
      @keydown.esc.prevent="emit('cancel-edit')"
    />

    <div
      v-else
      class="grow min-h-7 flex items-center px-2 rounded-md cursor-text"
      :class="isLinePlaying ? 'font-bold text-neutral-900 dark:text-white' : 'text-neutral-600 dark:text-neutral-400'"
      @click="emit('select', index)"
      @dblclick="emit('start-edit', index)"
    >
      <template v-if="hasWordSync">
        <span
          v-for="(word, wordIndex) in line.words"
          :key="wordIndex"
          class="whitespace-pre-wrap"
          :class="{
            'text-yellow-600 dark:text-yellow-300 font-bold': wordIndex === currentWordIndex,
          }"
        >
          {{ word.text }}
        </span>
      </template>
      <template v-else>
        {{ line.text || ' ' }}
      </template>
    </div>

    <div class="flex-none flex gap-3">
      <div class="relative flex items-center">
        <button
          v-show="isLineControlsVisible && line.end_ms"
          class="button p-0.5 rounded-full text-xs h-5 w-5 bg-neutral-200 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-300 absolute -left-1.5 top-1/2 -translate-y-1/2 z-10"
          title="Rewind end timestamp by 100ms"
          @click.stop="emit('rewind-end', index)"
        >
          <Minus />
        </button>
        <div
          v-show="isLineControlsVisible"
          class="px-3 py-0.5 text-xs font-mono rounded-full bg-neutral-200 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-300 min-w-[5.75rem] text-center"
          :class="{ 'opacity-50': !line.end_ms }"
        >
          {{ endTimestampText }}
        </div>
        <button
          v-show="isLineControlsVisible && line.end_ms"
          class="button p-0.5 rounded-full text-xs h-5 w-5 bg-neutral-200 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-300 absolute -right-1.5 top-1/2 -translate-y-1/2 z-10"
          title="Forward end timestamp by 100ms"
          @click.stop="emit('forward-end', index)"
        >
          <Plus />
        </button>
      </div>

      <button
        v-show="isLineControlsVisible"
        class="button bg-neutral-200 text-neutral-600 hover:bg-neutral-300 dark:bg-neutral-700 dark:text-neutral-300 hover:dark:bg-neutral-600 p-1 rounded-full text-sm h-6 w-6 mr-4"
        title="Sync end timestamp to current playback"
        @click.stop="emit('sync-end', index)"
      >
        <Equal />
      </button>
    </div>

    <div class="flex items-center gap-1 w-[3.5rem] justify-end">
      <button
        v-show="isLineControlsVisible"
        class="button button-normal p-1 rounded-full text-sm h-6 w-6"
        title="Delete line"
        @click.stop="emit('delete-line', index)"
      >
        <Close />
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import Play from '~icons/mdi/play'
import Equal from '~icons/mdi/equal'
import Minus from '~icons/mdi/minus'
import Plus from '~icons/mdi/plus'
import Close from '~icons/mdi/close'
const props = defineProps({
  index: {
    type: Number,
    required: true,
  },
  line: {
    type: Object,
    required: true,
  },
  rowClass: {
    type: String,
    default: 'bg-transparent',
  },
  isLineControlsVisible: {
    type: Boolean,
    default: false,
  },
  isEditing: {
    type: Boolean,
    default: false,
  },
  editingText: {
    type: String,
    default: '',
  },
  timestampText: {
    type: String,
    default: '',
  },
  endTimestampText: {
    type: String,
    default: '',
  },
  setLineInputRef: {
    type: Function,
    required: true,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
})

const emit = defineEmits([
  'mouseenter',
  'mouseleave',
  'select',
  'play-line',
  'sync-line',
  'rewind-line',
  'forward-line',
  'delete-line',
  'start-edit',
  'save-edit',
  'cancel-edit',
  'update:editing-text',
  'sync-end',
  'rewind-end',
  'forward-end',
])

const rowElement = ref(null)

// Determine if this line is currently playing based on its own time range
const isLinePlaying = computed(() => {
  if (!Number.isFinite(props.line?.start_ms)) {
    return false
  }

  const startMs = props.line.start_ms
  const endMs = props.line?.end_ms

  // If end_ms is set, check if progress is within [start_ms, end_ms)
  if (Number.isFinite(endMs)) {
    return props.progressMs >= startMs && props.progressMs < endMs
  }

  // If no end_ms, use the old behavior: playing if progress >= start_ms
  // (This handles the case where end_ms hasn't been set yet)
  return props.progressMs >= startMs
})

const editingTextProxy = computed({
  get: () => props.editingText,
  set: value => emit('update:editing-text', value),
})

// Check if line has word-by-word synced data
const hasWordSync = computed(() => {
  return props.line?.words && Array.isArray(props.line.words) && props.line.words.length > 0
})

// Determine the currently playing word index based on progressMs
const currentWordIndex = computed(() => {
  if (!hasWordSync.value || !props.line.words) {
    return -1
  }

  // Check if we're within the line's time range
  if (!isLinePlaying.value) {
    return -1
  }

  const words = props.line.words
  const lineEndMs = props.line?.end_ms

  for (let i = 0; i < words.length; i++) {
    const word = words[i]
    const nextWord = words[i + 1]

    // Check if current time falls within this word's time window
    const wordStart = word.start_ms
    // Use next word's start_ms, or line's end_ms, or Infinity as fallback
    const wordEnd = nextWord ? nextWord.start_ms : Number.isFinite(lineEndMs) ? lineEndMs : Infinity

    if (props.progressMs >= wordStart && props.progressMs < wordEnd) {
      return i
    }
  }

  return -1
})

defineExpose({
  rowElement,
})
</script>
