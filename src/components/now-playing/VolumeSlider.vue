<template>
  <div class="flex items-center gap-1 w-40">
    <button
      v-if="volume > 0"
      class="flex-none button text-neutral-800 dark:text-neutral-500 p-1 m-1 rounded-full"
      @click="mute"
    >
      <VolumeMedium />
    </button>
    <button
      v-else
      class="flex-none button text-neutral-800 dark:text-neutral-500 p-1 m-1 rounded-full"
      @click="unmute"
    >
      <VolumeMute />
    </button>

    <VueSlider
      v-model="volume"
      class="grow"
      :min="0"
      :max="1"
      :interval="0.01"
      :rail-style="{ backgroundColor: 'var(--slider-rail)' }"
      tooltip="hover"
      @change="chooseVolume"
    >
      <template #dot="{ pos, index, value, focus, disabled }">
        <div class="w-full h-full rounded-full bg-hoa-1100" />
      </template>

      <template #process="{ start, end }">
        <div class="absolute h-full rounded-full bg-hoa-1100" :style="'width: ' + end + '%;'" />
      </template>

      <template #tooltip="{ pos, index, value, focus, disabled }">
        <div
          v-if="value !== null"
          class="text-neutral-800 text-[0.6rem] font-bold rounded-lg px-1 py-0.5 bg-neutral-100"
        >
          {{ Math.round(value * 100) }}%
        </div>
      </template>
    </VueSlider>
  </div>
</template>

<script setup>
import VolumeMedium from '~icons/mdi/volume-medium'
import VolumeMute from '~icons/mdi/volume-mute'
import VueSlider from 'vue-3-slider-component'
import { ref, watch } from 'vue'

const props = defineProps(['volume'])
const emit = defineEmits(['setVolume'])

const volume = ref(props.volume)

const chooseVolume = value => {
  emit('setVolume', value)
}

const mute = () => {
  volume.value = 0.0
  emit('setVolume', 0.0)
}

const unmute = () => {
  volume.value = 1.0
  emit('setVolume', 1.0)
}

watch(
  () => props.volume,
  newVolume => {
    if (!newVolume) return

    volume.value = newVolume
  }
)
</script>
