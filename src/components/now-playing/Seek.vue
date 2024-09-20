<template>
  <VueSlider
    v-model="progressPercent"
    :min="0"
    :max="1"
    :interval="0.001"
    :duration="0"
    :rail-style="{ backgroundColor: '#ffd9e2' }"
    :dot-style="{ transition: 'initial' }"
    :tooltip-style="{ zIndex: 200 }"
    tooltip="hover"
    @change="chooseProgress"
  >
    <template #dot="{pos, index, value, focus, disabled}">
      <div
        class="w-full h-full rounded-full bg-brave-30"
      />
    </template>

    <template #process="{ start, end }">
      <div
        class="absolute h-full rounded-full bg-brave-30"
        :style="'width: ' + end + '%;'"
      />
    </template>

    <template #tooltip="{pos, index, value, focus, disabled}">
      <div v-if="value" class="text-brave-30 text-[0.6rem] font-bold rounded-lg px-1 py-0.5 bg-brave-90">{{ humanDuration(value * props.duration) }}</div>
    </template>
  </VueSlider>
</template>

<script setup>
import VueSlider from "vue-3-slider-component";
import { ref, onMounted, watch } from 'vue'
import { humanDuration } from '@/utils/human-duration'
import _throttle from 'lodash/throttle'

const props = defineProps(['duration', 'progress'])
const emit = defineEmits(['seek'])
const isGracePeriod = ref(false)
const gracePeriodTimeout = ref(null)

const progressPercent = ref(0)

// There is a slight delay after seeking before the player can actually start playing from the new position due to kira's StreamingSoundHandle.
// So this is a hack to prevent the seek bar from jumping back to the old position after user seeks.
// Also throttle the seek event to prevent it from being called too frequently.
const chooseProgress = _throttle((value) => {
  emit('seek', value * props.duration)

  isGracePeriod.value = true

  clearTimeout(gracePeriodTimeout.value)
  gracePeriodTimeout.value = setTimeout(() => {
    isGracePeriod.value = false
  }, 500)
}, 200)

onMounted(() => {
  progressPercent.value = props.progress / props.duration
})

watch(() => props.progress, (newProgress) => {
  if (isGracePeriod.value) return
  progressPercent.value = newProgress / props.duration
})

watch(() => props.duration, (newDuration) => {
  progressPercent.value = props.progress / newDuration
})
</script>
