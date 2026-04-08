<template>
  <BaseModal
    title="YAML Debug"
    :esc-to-close="true"
    :click-to-close="true"
    content-class="w-full h-[90vh] max-w-screen-lg"
    body-class="p-0"
    @close="emit('close')"
  >
    <div class="h-full flex flex-col relative">
      <div class="grow min-h-0 overflow-auto bg-black text-green-400 text-sm p-4 select-text">
        <pre class="font-mono whitespace-pre-wrap break-words overflow-x-auto select-text">{{ content }}</pre>
      </div>
      <button
        class="absolute bottom-4 right-4 button text-sm px-3 py-1.5 h-8 rounded-full button-normal"
        @click="copyToClipboard"
      >
        <span class="flex items-center gap-1">
          <ContentCopy class="text-base" />
          {{ copied ? 'Copied!' : 'Copy' }}
        </span>
      </button>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import { ContentCopy } from 'mdue'
import BaseModal from '@/components/common/BaseModal.vue'

defineProps({
  content: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

const copied = ref(false)

async function copyToClipboard() {
  const content = document.querySelector('.bg-black pre')?.innerText
  if (!content) return

  try {
    await navigator.clipboard.writeText(content)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>
