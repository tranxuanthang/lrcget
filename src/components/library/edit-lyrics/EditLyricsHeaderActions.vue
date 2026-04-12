<template>
  <div class="flex-none flex gap-1 items-center">
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

    <VTooltip theme="lrcget-tooltip">
      <button
        class="button text-sm px-5 py-1.5 h-8 w-24 rounded-full"
        :class="{ 'button-primary': !isDirty, 'button-disabled': isDirty }"
        :disabled="isDirty"
        @click="emit('publish')"
      >
        Publish
      </button>

      <template #popper>
        <div class="text-xs font-bold">
          {{ publishButtonTooltip }}
        </div>
      </template>
    </VTooltip>

    <VTooltip theme="lrcget-tooltip">
      <Check v-if="publishStatus === 'clean'" class="text-lime-500 text-2xl block" />
      <AlertCircleOutline
        v-else-if="publishStatus === 'plain-text-only'"
        class="text-orange-500 text-2xl block"
      />
      <AlertCircle v-else class="text-red-500 text-2xl block" />

      <template #popper>
        <div class="text-xs font-bold">
          <div v-for="line in publishStatusLines" :key="line">
            {{ line }}
          </div>
        </div>
      </template>
    </VTooltip>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import Check from '~icons/mdi/check'
import AlertCircleOutline from '~icons/mdi/alert-circle-outline'
import AlertCircle from '~icons/mdi/alert-circle'
const emit = defineEmits(['save', 'publish'])

const props = defineProps({
  isDirty: {
    type: Boolean,
    required: true,
  },
  publishButtonTooltip: {
    type: String,
    required: true,
  },
  publishStatus: {
    type: String,
    required: true,
  },
  publishStatusTooltip: {
    type: String,
    required: true,
  },
})

const publishStatusLines = computed(() => props.publishStatusTooltip.split('\n'))
</script>
