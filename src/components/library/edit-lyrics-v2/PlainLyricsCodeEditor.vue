<template>
  <div class="grow overflow-hidden flex flex-col gap-2 h-full">
    <div ref="cmContainer" class="relative h-full w-full">
      <div
        class="overflow-hidden absolute w-full"
        :style="{ height: `${cmHeight}px`, fontSize: `${fontSize}em` }"
        @wheel="handleWheel"
      >
        <AsyncCodemirror
          v-if="shouldLoadCodeMirror"
          v-model="lyricsProxy"
          placeholder="Plain lyrics is currently empty"
          class="codemirror-custom h-full outline-none"
          :autofocus="true"
          :indent-with-tab="true"
          :tab-size="2"
          :config="{ height: 'auto' }"
        />

        <div v-else class="flex flex-col h-full items-center justify-center text-sm text-neutral-500">
          <div class="animate-spin text-xl text-neutral-800">
            <Loading />
          </div>
          <div>Loading editor...</div>
        </div>
      </div>
    </div>

    <div class="flex flex-col w-fit self-end bg-neutral-50 dark:bg-neutral-900 rounded-lg">
      <div class="toolbar px-2 py-1 flex items-stretch gap-1">
        <button
          class="button button-normal px-1.5 py-0.5 text-sm rounded-full"
          title="Zoom out"
          @click="emit('change-font-size', -1)"
        >
          <MagnifyMinus />
        </button>
        <button
          class="button button-normal px-1.5 py-0.5 text-sm rounded-full w-[4.5em]"
          title="Reset zoom level"
          @click="emit('reset-font-size')"
        >
          {{ (fontSize * 100).toFixed(0) }}%
        </button>
        <button
          class="button button-normal px-1.5 py-0.5 text-sm rounded-full"
          title="Zoom in"
          @click="emit('change-font-size', 1)"
        >
          <MagnifyPlus />
        </button>
      </div>
    </div>

    <PlainLyricsEmptyState
      v-if="shouldShowEmptyState"
      :can-import-from-synced="canImportFromSynced"
      @import-lines-from-synced="handleImportFromSynced"
      @add-line-manually="handleAddLineManually"
      @mark-as-instrumental="handleMarkAsInstrumental"
    />
  </div>
</template>

<script setup>
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref, watch } from 'vue'
import PlainLyricsEmptyState from '@/components/library/edit-lyrics-v2/PlainLyricsEmptyState.vue'
import Loading from '~icons/mdi/loading'
import MagnifyPlus from '~icons/mdi/magnify-plus'
import MagnifyMinus from '~icons/mdi/magnify-minus'
const AsyncCodemirror = defineAsyncComponent(async () => {
  const { Codemirror } = await import('vue-codemirror')
  return Codemirror
})

const props = defineProps({
  modelValue: {
    type: String,
    required: true,
  },
  fontSize: {
    type: Number,
    default: 1,
  },
  syncedLines: {
    type: Array,
    default: () => [],
  },
})

const emit = defineEmits([
  'update:modelValue',
  'change-font-size',
  'reset-font-size',
  'import-lines-from-synced',
  'mark-as-instrumental',
])

const cmContainer = ref(null)
const cmHeight = ref(0)
const shouldLoadCodeMirror = ref(false)
const manuallyDismissed = ref(false)

const lyricsProxy = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

const handleResize = () => {
  if (!cmContainer.value) {
    return
  }

  cmHeight.value = cmContainer.value.offsetHeight
}

const handleWheel = event => {
  if (!event.ctrlKey) {
    return
  }

  emit('change-font-size', event.deltaY > 0 ? -1 : 1)
}

const shouldShowEmptyState = computed(() => {
  return props.modelValue.trim().length === 0 && !manuallyDismissed.value
})

const canImportFromSynced = computed(() => {
  return props.syncedLines.length > 0
})

const handleImportFromSynced = () => {
  if (!canImportFromSynced.value) {
    return
  }

  const linesText = props.syncedLines
    .map(line => line.text || '')
    .filter(text => text.length > 0)
    .join('\n')

  emit('update:modelValue', linesText)
}

const handleAddLineManually = () => {
  // Simply hide the empty state
  manuallyDismissed.value = true
}

const handleMarkAsInstrumental = () => {
  emit('mark-as-instrumental')
}

// Reset manuallyDismissed when content is added so that if user clears it again,
// the empty state will show
watch(
  () => props.modelValue,
  newValue => {
    if (newValue.trim().length > 0) {
      manuallyDismissed.value = false
    }
  }
)

onMounted(() => {
  window.addEventListener('resize', handleResize)
  handleResize()
  setTimeout(() => {
    shouldLoadCodeMirror.value = true
  }, 100)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
</script>
