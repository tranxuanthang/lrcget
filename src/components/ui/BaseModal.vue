<template>
  <div>
    <Transition name="pop-fade">
      <div v-if="model" class="fixed top-0 left-0 w-full h-full flex items-center justify-center z-30" @click="handleBackgroundClick">
        <slot v-if="model" class="pointer-events-auto" v-bind="$attrs" />
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="model" class="fixed top-0 left-0 h-full w-full z-20 bg-black/30 cursor-default"></div>
    </Transition>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const emit = defineEmits(['close'])

const model = defineModel()

const props = defineProps({
  clickOutsideToClose: {
    type: Boolean,
    default: true,
  },
})

const handleBackgroundClick = (event) => {
  if (event.currentTarget === event.target && props.clickOutsideToClose) {
    model.value = false
  }
}

const closeModal = () => {
  model.value = false
}

defineExpose({
  closeModal,
})
</script>
