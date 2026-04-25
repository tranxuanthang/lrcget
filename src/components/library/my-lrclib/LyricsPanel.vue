<template>
  <div
    class="relative grow rounded bg-white text-neutral-800 dark:bg-neutral-950 dark:text-neutral-200 h-full overflow-hidden"
  >
    <button
      class="absolute bottom-2 right-2 flex items-center gap-1 px-3 py-1 rounded text-xs font-bold bg-neutral-100 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-800"
      :aria-label="copied ? 'Copied' : 'Copy'"
      @click.stop="onCopy"
    >
      <ContentCopy class="w-4 h-4" />
      <span>{{ copied ? 'Copied' : 'Copy' }}</span>
    </button>

    <div class="h-full overflow-auto whitespace-pre-wrap select-text p-4">
      {{ text }}
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import ContentCopy from '~icons/mdi/content-copy'
const props = defineProps({
  text: { type: String, required: true },
})

const copied = ref(false)

const onCopy = async () => {
  if (!props.text) return
  await navigator.clipboard.writeText(props.text)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}
</script>
