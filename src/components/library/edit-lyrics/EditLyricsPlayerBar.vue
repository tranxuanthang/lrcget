<template>
  <div class="flex gap-4 items-center px-4 py-2">
    <button v-if="status !== 'playing'" class="button button-normal p-2 rounded-full text-xl" @click.prevent="emit('play-toggle')"><Play /></button>
    <button v-else class="button button-normal p-2 rounded-full text-xl" @click.prevent="emit('pause')"><Pause /></button>
    <div class="flex-none w-12 text-xs">{{ humanDuration(progress) }}</div>
    <Seek class="grow" :duration="duration" :progress="progress" @seek="emit('seek', $event)" />
    <div class="flex-none w-12 text-xs">{{ humanDuration(duration) }}</div>
  </div>
</template>

<script setup>
import Play from '~icons/mdi/play'
import Pause from '~icons/mdi/pause'
import Seek from '@/components/now-playing/Seek.vue'

defineProps({
  status: {
    type: String,
    required: true
  },
  duration: {
    type: Number,
    default: 0
  },
  progress: {
    type: Number,
    default: 0
  }
})

const emit = defineEmits(['play-toggle', 'pause', 'seek'])

const humanDuration = (seconds) => {
  const boundedSeconds = seconds || 0
  return new Date(boundedSeconds * 1000).toISOString().slice(11, 19)
}
</script>
