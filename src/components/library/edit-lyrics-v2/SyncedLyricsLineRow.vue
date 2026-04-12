<template>
  <div
    ref="rowElement"
    class="group flex items-center gap-2 px-4 py-1.5 rounded-md transition-colors h-9"
    :class="rowClass"
    @mouseenter="emit('mouseenter', index)"
    @mouseleave="emit('mouseleave')"
    @click="emit('select', index)"
  >
    <div class="flex items-center gap-1 w-[7.5rem]">
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
      <button
        v-show="isLineControlsVisible"
        class="button p-1 rounded-full text-sm h-6 w-6"
        :class="line.start_ms ? 'button-normal' : 'button-disabled'"
        :disabled="!line.start_ms"
        title="Rewind line by 100ms"
        @click.stop="emit('rewind-line', index)"
      >
        <Minus />
      </button>
      <button
        v-show="isLineControlsVisible"
        class="button p-1 rounded-full text-sm h-6 w-6"
        :class="line.start_ms ? 'button-normal' : 'button-disabled'"
        :disabled="!line.start_ms"
        title="Forward line by 100ms"
        @click.stop="emit('forward-line', index)"
      >
        <Plus />
      </button>
    </div>

    <div
      class="flex-none px-2 py-0.5 text-xs font-mono rounded-full bg-brave-90 dark:bg-brave-20 text-brave-25 dark:text-brave-99 min-w-[5.75rem] text-center"
      :class="{ 'font-bold': isLinePlaying }"
    >
      {{ timestampText }}
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

    <button
      v-show="isLineControlsVisible"
      class="button button-normal p-1 rounded-full text-sm"
      title="Delete line"
      @click.stop="emit('delete-line', index)"
    >
      <Close />
    </button>
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
