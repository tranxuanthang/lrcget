<template>
  <div class="flex items-center gap-1">
    <button v-if="volume > 0" @click="mute" class="button text-brave-30 p-1 m-1 rounded-full"><VolumeMedium /></button>
    <button v-else @click="unmute" class="button text-brave-30 p-1 m-1 rounded-full"><VolumeMute /></button>

    <div class="bg-brave-90 w-32 h-2 relative origin-center hover:scale-y-150 transition" @mouseover="onMouseOver" @mousemove="onMouseOver" @mouseleave="onMouseLeave" @click="chooseVolume">
      <div class="bg-brave-30 h-full" :style="{ width: volumePercent }"></div>
      <div class="bg-brave-60 h-full absolute top-0 left-0 opacity-40" :style="{ width: choosingVolumePercent }"></div>
    </div>
  </div>
</template>

<script setup>
import { VolumeMedium, VolumeMute } from 'mdue';
import { ref, computed } from 'vue'

const props = defineProps(['volume'])
const emit = defineEmits(['setVolume'])

const volumePercent = computed(() => `${props.volume * 100}%`)

const choosingVolume = ref(0.0)

const choosingVolumePercent = computed(() => `${choosingVolume.value * 100}%`)

const onMouseOver = (event) => {
  const totalWidth = event.currentTarget.clientWidth
  const seekWidth = event.offsetX
  choosingVolume.value = seekWidth / totalWidth
}

const onMouseLeave = () => {
  choosingVolume.value = 0.0
}

const chooseVolume = () => {
  emit('setVolume', choosingVolume.value)
}

const mute = () => {
  emit('setVolume', 0.0)
}

const unmute = () => {
  emit('setVolume', 1.0)
}
</script>