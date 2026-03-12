<template>
  <div class="grow overflow-hidden flex flex-col gap-2 h-full">
    <div class="relative h-full w-full" ref="cmContainer">
      <div class="overflow-hidden absolute w-full" :style="{ height: `${cmHeight}px`, fontSize: `${fontSize}em` }" @wheel="handleWheel">
        <AsyncCodemirror
          v-if="shouldLoadCodeMirror"
          v-model="lyricsProxy"
          placeholder="Lyrics is currently empty"
          class="codemirror-custom h-full outline-none"
          :autofocus="true"
          :indent-with-tab="true"
          :tab-size="2"
          :extensions="extensions"
          :config="{ height: 'auto' }"
          @ready="emit('ready', $event)"
        />

        <div v-else class="flex flex-col h-full items-center justify-center text-sm text-brave-40">
          <div class="animate-spin text-xl text-brave-30"><Loading /></div>
          <div>Loading editor...</div>
        </div>
      </div>
    </div>

    <div class="flex flex-col w-fit self-end bg-brave-95 dark:bg-brave-10 rounded-lg">
      <div class="toolbar px-2 py-1 flex items-stretch gap-1">
        <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full" title="Zoom out" @click="emit('change-font-size', -1)"><MagnifyMinus /></button>
        <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full w-[4.5em]" title="Reset zoom level" @click="emit('reset-font-size')">{{ (fontSize * 100).toFixed(0) }}%</button>
        <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full" title="Zoom in" @click="emit('change-font-size', 1)"><MagnifyPlus /></button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue'
import { Loading, MagnifyPlus, MagnifyMinus } from 'mdue'

const AsyncCodemirror = defineAsyncComponent(async () => {
  const { Codemirror } = await import('vue-codemirror')
  return Codemirror
})

const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },
  extensions: {
    type: Array,
    default: () => []
  },
  fontSize: {
    type: Number,
    default: 1
  }
})

const emit = defineEmits(['update:modelValue', 'ready', 'change-font-size', 'reset-font-size'])

const cmContainer = ref(null)
const cmHeight = ref(0)
const shouldLoadCodeMirror = ref(false)

const lyricsProxy = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const handleResize = () => {
  if (!cmContainer.value) {
    return
  }

  cmHeight.value = cmContainer.value.offsetHeight
}

const handleWheel = (event) => {
  if (!event.ctrlKey) {
    return
  }

  emit('change-font-size', event.deltaY > 0 ? -1 : 1)
}

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
