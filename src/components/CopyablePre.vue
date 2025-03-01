<template>
  <div class="overflow-auto max-w-full pr-12 relative bg-indigo-900 text-gray-100">
    <pre ref="copyablePre" class="whitespace-pre overflow-x-auto cursor-pointer select-all p-4"><slot /></pre>
    <div class="absolute top-0 right-0 h-full flex items-center px-1">
      <button class="p-2 rounded bg-white dark:bg-black text-indigo-800 dark:text-indigo-400" @click="copyToClipboard">
        <ContentCopy />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { ContentCopy } from 'mdue'

const copyablePre = ref(null)

async function copyToClipboard() {
  if (copyablePre.value && navigator.clipboard && window.isSecureContext) {
    // Use the Clipboard API when available
    try {
      await navigator.clipboard.writeText(copyablePre.value.innerText)
      console.log('Content copied to clipboard')
      // Optionally, show user feedback here (e.g., a tooltip or toast message)
    } catch (err) {
      console.error('Could not copy text: ', err)
    }
  } else {
    // Fallback method: Create a temporary textarea element to copy the text
    const textArea = document.createElement("textarea")
    textArea.value = copyablePre.value ? copyablePre.value.innerText : ''
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()
    try {
      const successful = document.execCommand('copy')
      console.log(successful ? 'Content copied to clipboard' : 'Failed to copy')
    } catch (err) {
      console.error('Failed to copy content: ', err)
    }
    document.body.removeChild(textArea)
  }
}
</script>
