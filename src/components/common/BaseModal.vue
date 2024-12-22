<template>
  <VueFinalModal
    class="flex justify-center items-center"
    :content-class="['modal-content p-6 gap-6 flex flex-col', contentClass]"
    overlay-class="bg-brave-1/70 dark:bg-brave-1/70"
    overlay-transition="fade"
    content-transition="pop-fade"
    v-bind="$attrs"
  >
    <div v-if="title || $slots.title" class="flex-none flex justify-between items-center relative">
      <div class="flex-none" v-if="$slots.titleLeft">
        <slot name="titleLeft" />
      </div>

      <div
       class="text-lg grow line-clamp-1 overflow-hidden text-brave-30 dark:text-brave-90"
       :class="{ 'text-center': $slots.titleLeft }"
      >
        {{ title }}
      </div>

      <button
        v-if="closeButton"
        class="absolute top-1/2 right-0 -translate-y-1/2 flex-none button text-brave-30 dark:text-brave-90 rounded-full h-12 w-12"
        @click="emit('close')"
      >
        <Close />
      </button>
    </div>

    <div :class="bodyClass">
      <slot />
    </div>

    <div v-if="$slots.footer" class="flex-none flex justify-center">
      <slot name="footer" />
    </div>
  </VueFinalModal>
</template>

<script setup>
import { Close } from 'mdue'

const props = defineProps({
  title: {
    type: String,
    required: false
  },
  contentClass: {
    type: String,
    default: 'w-full h-[80vh] max-w-screen-sm'
  },
  bodyClass: {
    type: String,
    default: 'grow overflow-auto'
  },
  closeButton: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['close'])
</script>
