<template>
  <div class="relative h-0">
    <button
      class="button button-normal absolute left-1/2 -translate-x-1/2 -translate-y-1/2 p-0.5 rounded-full text-xs"
      :title="title"
      :style="{ opacity }"
      :class="opacity > 0.05 ? '' : 'pointer-events-none'"
      @click="(e) => emit('click', e)"
    >
      <Plus />
    </button>
  </div>
</template>

<script setup>
import { Plus } from 'mdue'

defineProps({
  title: {
    type: String,
    required: true
  },
  opacity: {
    type: Number,
    default: 0
  }
})

const emit = defineEmits(['click'])
</script>
