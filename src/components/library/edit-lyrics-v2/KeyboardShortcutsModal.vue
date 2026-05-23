<template>
  <BaseModal
    title="Keyboard Shortcuts"
    :esc-to-close="true"
    :click-to-close="true"
    content-class="w-full max-w-md max-h-[80vh]"
    body-class="overflow-auto"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-5">
      <div
        v-for="category in shortcutCategories"
        :key="category.id"
        class="flex flex-col gap-2"
      >
        <h3
          class="text-xs font-bold uppercase tracking-wider"
          :class="
            isCategoryActive(category.id)
              ? 'text-hoa-800 dark:text-hoa-200'
              : 'text-neutral-500 dark:text-neutral-400'
          "
        >
          {{ category.label }}
          <span
            v-if="isCategoryActive(category.id)"
            class="ml-1.5 inline-block w-1.5 h-1.5 rounded-full bg-hoa-500"
          />
        </h3>

        <div class="flex flex-col gap-1.5">
          <div
            v-for="(shortcut, index) in category.shortcuts"
            :key="index"
            class="flex items-center justify-between gap-4"
          >
            <span class="text-sm text-neutral-700 dark:text-neutral-300">{{ shortcut.description }}</span>
            <div class="flex items-center gap-1 shrink-0">
              <span
                v-for="(key, keyIndex) in shortcut.keys"
                :key="keyIndex"
                class="inline-flex items-center justify-center px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-700 border border-neutral-200 dark:border-neutral-600 text-neutral-700 dark:text-neutral-300 font-mono text-xs leading-none min-w-[1.5rem]"
              >
                {{ key }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import BaseModal from '@/components/common/BaseModal.vue'
import { shortcutCategories } from '@/composables/edit-lyrics-v2/keyboardShortcuts.js'

const props = defineProps({
  activeTab: {
    type: String,
    required: true,
  },
})

const emit = defineEmits(['close'])

const isCategoryActive = categoryId => {
  if (categoryId === 'global') return true
  if (props.activeTab === 'plain') return categoryId === 'plain'
  if (props.activeTab === 'synced') return categoryId === 'synced' || categoryId === 'wordTiming'
  return false
}
</script>
