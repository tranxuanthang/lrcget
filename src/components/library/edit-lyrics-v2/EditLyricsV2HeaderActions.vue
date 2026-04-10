<template>
  <div class="flex-none flex gap-2 items-center">
    <div class="flex-none flex">
      <VTooltip theme="lrcget-tooltip">
        <button
          class="button text-sm px-5 py-1.5 h-8 w-24 rounded-l-full rounded-r-none border-r border-brave-95 dark:border-brave-5"
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

      <VDropdown theme="lrcget-dropdown" placement="bottom-start">
        <button
          class="button text-sm px-2 py-1.5 h-8 rounded-r-full rounded-l-none button-normal"
          type="button"
        >
          <ChevronDown class="text-base" />
        </button>

        <template #popper>
          <div class="dropdown-container">
            <button class="dropdown-item" @click="emit('save-and-publish')" v-close-popper>
              <span class="dropdown-label">Save and Publish</span>
            </button>

            <div class="dropdown-divider" />
            <div class="dropdown-section-label">Export to directory:</div>

            <label class="dropdown-item">
              <CheckboxButton
                v-model="exportPlainText"
                name="export-plain-text"
                id="export-plain-text"
              >
                <span class="dropdown-label">Plain lyrics (.txt)</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="exportSyncedLrc"
                name="export-synced-lrc"
                id="export-synced-lrc"
              >
                <span class="dropdown-label">Synced lyrics (.lrc)</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="exportEnhancedLrc"
                name="export-enhanced-lrc"
                id="export-enhanced-lrc"
              >
                <span class="dropdown-label">Enhanced LRC lyrics (.elrc)</span>
              </CheckboxButton>
            </label>
          </div>
        </template>
      </VDropdown>
    </div>

    <VTooltip theme="lrcget-tooltip">
      <button
        class="button text-sm px-3 py-1.5 h-8 rounded-full button-normal"
        @click="emit('debug')"
      >
        Debug
      </button>

      <template #popper>
        <div class="text-xs font-bold">View YAML debug</div>
      </template>
    </VTooltip>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import ChevronDown from '~icons/mdi/chevron-down'
import CheckboxButton from '@/components/common/CheckboxButton.vue'

const emit = defineEmits(['save', 'save-and-publish', 'debug'])

const exportPlainText = ref(false)
const exportSyncedLrc = ref(false)
const exportEnhancedLrc = ref(false)

defineProps({
  isDirty: {
    type: Boolean,
    required: true
  }
})
</script>

<style scoped>
.dropdown-container {
  @apply p-1 min-w-[17rem];
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-brave-90 dark:hover:bg-brave-15 rounded cursor-pointer h-8 gap-2 w-full;
}

.dropdown-divider {
  @apply h-px bg-brave-90 dark:bg-brave-15 my-1;
}

.dropdown-label {
  @apply text-brave-20 dark:text-brave-90 text-sm font-bold;
}

.dropdown-section-label {
  @apply text-xs uppercase font-bold text-brave-35 dark:text-brave-70 px-2 py-1;
}

</style>
