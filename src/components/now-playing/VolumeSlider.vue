<template>
  <div class="flex items-center gap-1 w-40">
    <button v-if="volume > 0" @click="mute"
      class="flex-none button text-brave-30 dark:text-brave-80 p-1 m-1 rounded-full">
      <VolumeMedium />
    </button>
    <button v-else @click="unmute" class="flex-none button text-brave-30 dark:text-brave-80 p-1 m-1 rounded-full">
      <VolumeMute />
    </button>

    <VueSlider class="grow" v-model="volume" :min="0" :max="1" :interval="0.01"
      :rail-style="{ backgroundColor: '#ffd9e2' }" tooltip="hover" @change="chooseVolume">
      <template #dot="{ pos, index, value, focus, disabled }">
        <div class="w-full h-full rounded-full bg-brave-30" />
      </template>

      <template #process="{ start, end }">
        <div class="absolute h-full rounded-full bg-brave-30" :style="'width: ' + end + '%;'" />
      </template>

      <template #tooltip="{ pos, index, value, focus, disabled }">
        <div v-if="value !== null" class="text-brave-30 text-[0.6rem] font-bold rounded-lg px-1 py-0.5 bg-brave-90">{{
          Math.round(value * 100) }}%</div>
      </template>
    </VueSlider>
  </div>
</template>

<script setup>
import { VolumeMedium, VolumeMute } from 'mdue';
import VueSlider from "vue-3-slider-component";
import { ref, watch } from 'vue'

const props = defineProps(['volume'])
const emit = defineEmits(['setVolume'])

const volume = ref(props.volume)

const chooseVolume = (value) => {
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

watch(() => props.volume, (newVolume) => {
  if (!newVolume) return

  volume.value = newVolume
})
</script>
