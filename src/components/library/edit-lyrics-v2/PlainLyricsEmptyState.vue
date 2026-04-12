<template>
  <div class="absolute bottom-16 left-1/2 -translate-x-1/2 px-3 z-10">
    <div
      class="w-full max-w-lg rounded-lg border border-brave-90 dark:border-brave-20 bg-brave-98 dark:bg-brave-10 p-5 shadow-lg"
    >
      <h3 class="text-base font-semibold">No plain lyrics yet</h3>
      <p class="text-sm text-brave-45 dark:text-brave-70 mt-1">
        Import from synced lyrics or add plain lyrics manually.
      </p>

      <div class="mt-4 flex flex-wrap gap-2">
        <button
          class="button px-2 py-1 text-xs rounded-full"
          :class="canImportFromSynced ? 'button-normal' : 'button-disabled'"
          :disabled="!canImportFromSynced"
          @click="emit('import-lines-from-synced')"
        >
          Import from synced lyrics
        </button>
        <button
          class="button button-normal px-2 py-1 text-xs rounded-full"
          @click="emit('add-line-manually')"
        >
          Add line manually
        </button>
        <button
          class="button button-normal px-2 py-1 text-xs rounded-full"
          @click="emit('mark-as-instrumental')"
        >
          Mark as instrumental
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  canImportFromSynced: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['import-lines-from-synced', 'add-line-manually', 'mark-as-instrumental'])
</script>
