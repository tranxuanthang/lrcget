<template>
  <div class="relative grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 h-full overflow-hidden">
    <button
      class="absolute bottom-2 right-2 flex items-center gap-1 px-3 py-1 rounded text-xs font-bold bg-brave-90 text-brave-20 dark:bg-brave-10 dark:text-brave-95 hover:bg-brave-80 dark:hover:bg-brave-20"
      @click.stop="onCopy"
      :aria-label="copied ? 'Copied' : 'Copy'"
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
import { ContentCopy } from 'mdue'

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
