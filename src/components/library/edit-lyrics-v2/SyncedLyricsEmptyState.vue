<template>
  <div class="h-full flex items-center justify-center px-3">
    <div class="w-full max-w-lg rounded-lg border border-brave-90 dark:border-brave-20 bg-brave-98 dark:bg-brave-10 p-5">
      <h3 class="text-base font-semibold">No synced lyric lines yet</h3>
      <p class="text-sm text-brave-45 dark:text-brave-70 mt-1">
        Import from plain lyrics or add a new synced line manually.
      </p>

      <div class="mt-4 flex flex-wrap gap-2">
        <button
          class="button button-normal px-8 py-2 rounded-full"
          :class="canImportFromPlain ? '' : 'button-disabled'"
          :disabled="!canImportFromPlain"
          @click="emit('import-lines-from-plain')"
        >
          Import from plain lyrics
        </button>
        <button
          class="button button-normal px-8 py-2 rounded-full"
          @click="emit('add-line-at', 0)"
        >
          Add line manually
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  canImportFromPlain: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['import-lines-from-plain', 'add-line-at'])
</script>
