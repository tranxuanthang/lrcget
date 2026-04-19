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
        class="button p-0.5 rounded-full text-xs h-5 w-5 bg-brave-90 dark:bg-brave-20 text-brave-25 dark:text-brave-99 absolute -left-1.5 top-1/2 -translate-y-1/2 z-10"
        title="Rewind line by 100ms"
        @click.stop="emit('rewind-line', index)"
      >
        <Minus />
      </button>
      <div
        class="px-3 py-0.5 text-xs font-mono rounded-full bg-brave-90 dark:bg-brave-20 text-brave-25 dark:text-brave-99 min-w-[5.75rem] text-center"
        :class="{ 'font-bold': isLinePlaying }"
      >
        {{ timestampText }}
      </div>
      <button
        v-show="isLineControlsVisible && line.start_ms"
        class="button p-0.5 rounded-full text-xs h-5 w-5 bg-brave-90 dark:bg-brave-20 text-brave-25 dark:text-brave-99 absolute -right-1.5 top-1/2 -translate-y-1/2 z-10"
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
      class="grow h-full px-2 rounded-md border border-brave-80 dark:border-brave-25 bg-brave-100 dark:bg-brave-10 outline-none"
      :class="{ 'font-bold': isLinePlaying }"
      @blur="emit('save-edit')"
      @keydown.enter.prevent="emit('save-edit')"
      @keydown.esc.prevent="emit('cancel-edit')"
    />

    <div
      v-else
      class="grow min-h-7 flex items-center px-2 rounded-md cursor-text"
      :class="{ 'font-bold': isLinePlaying, 'opacity-80': !isLinePlaying }"
      @click="emit('select', index)"
      @dblclick="emit('start-edit', index)"
    >
      <template v-if="hasWordSync">
        <span
          v-for="(word, wordIndex) in line.words"
          :key="wordIndex"
          class="whitespace-pre-wrap"
          :class="{
            'text-yellow-500 dark:text-yellow-400': wordIndex === currentWordIndex,
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
          class="button p-0.5 rounded-full text-xs h-5 w-5 bg-cyan-100 text-cyan-800  dark:bg-cyan-800 dark:text-white absolute -left-1.5 top-1/2 -translate-y-1/2 z-10"
          title="Rewind end timestamp by 100ms"
          @click.stop="emit('rewind-end', index)"
        >
          <Minus />
        </button>
        <div
          v-show="isLineControlsVisible"
          class="px-3 py-0.5 text-xs font-mono rounded-full bg-cyan-100 dark:bg-cyan-800 text-cyan-800 dark:text-white min-w-[5.75rem] text-center"
          :class="{ 'opacity-50': !line.end_ms }"
        >
          {{ endTimestampText }}
        </div>
        <button
          v-show="isLineControlsVisible && line.end_ms"
          class="button p-0.5 rounded-full text-xs h-5 w-5 bg-cyan-100 text-cyan-800  dark:bg-cyan-800 dark:text-white absolute -right-1.5 top-1/2 -translate-y-1/2 z-10"
          title="Forward end timestamp by 100ms"
          @click.stop="emit('forward-end', index)"
        >
          <Plus />
        </button>
      </div>

      <button
        v-show="isLineControlsVisible"
        class="button bg-cyan-100 text-cyan-800 hover:bg-cyan-200 dark:bg-cyan-800 dark:text-white hover:dark:bg-cyan-700 p-1 rounded-full text-sm h-6 w-6 mr-4"
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
  isLinePlaying: {
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
  if (!hasWordSync.value || !props.isLinePlaying || !props.line.words) {
    return -1
  }

  const words = props.line.words
  for (let i = 0; i < words.length; i++) {
    const word = words[i]
    const nextWord = words[i + 1]

    // Check if current time falls within this word's time window
    const wordStart = word.start_ms
    const wordEnd = nextWord ? nextWord.start_ms : Infinity

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
