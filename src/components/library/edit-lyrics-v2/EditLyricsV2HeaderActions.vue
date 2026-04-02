<template>
  <div class="flex-none flex gap-2 items-center">
    <VTooltip theme="lrcget-tooltip">
      <button
        class="button text-sm px-5 py-1.5 h-8 w-24 rounded-full"
        :class="{ 'button-primary': isDirty, 'button-disabled': !isDirty }"
        :disabled="!isDirty"
        @click="emit('save')"
      >
        Save
      </button>

      <template #popper>
        <div class="text-xs font-bold">
          Save lyrics
          <span class="text-[0.65rem] text-brave-30 bg-brave-95 px-1 rounded-full">Ctrl+S</span>
        </div>
      </template>
    </VTooltip>
  </div>
</template>

<script setup>
const emit = defineEmits(['save'])

defineProps({
  isDirty: {
    type: Boolean,
    required: true
  }
})
</script>
