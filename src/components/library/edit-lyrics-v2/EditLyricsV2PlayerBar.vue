<template>
  <div class="flex gap-4 items-center px-4 py-2">
    <button
      v-if="status !== 'playing'"
      class="button button-normal p-2 rounded-full text-xl"
      @click.prevent="emit('play-toggle')"
    >
      <Play />
    </button>
    <button
      v-else
      class="button button-normal p-2 rounded-full text-xl"
      @click.prevent="emit('pause')"
    >
      <Pause />
    </button>
    <div class="flex-none w-12 text-xs text-neutral-600 dark:text-neutral-400">
      {{ humanDuration(progress) }}
    </div>
    <Seek class="grow" :duration="duration" :progress="progress" @seek="emit('seek', $event)" />
    <div class="flex-none w-12 text-xs text-neutral-600 dark:text-neutral-400">
      {{ humanDuration(duration) }}
    </div>

    <label class="flex-none inline-flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-400">
      <span>Speed</span>
      <select
        class="px-2 py-1 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-200"
        :value="String(playbackSpeed)"
        @change="handleSpeedChange"
      >
        <option v-for="option in speedOptions" :key="option" :value="String(option)">
          {{ option.toFixed(2).replace(/\.00$/, '') }}x
        </option>
      </select>
    </label>
  </div>
</template>

<script setup>
import Play from '~icons/mdi/play'
import Pause from '~icons/mdi/pause'
import Seek from '@/components/now-playing/Seek.vue'

defineProps({
  status: {
    type: String,
    required: true,
  },
  duration: {
    type: Number,
    default: 0,
  },
  progress: {
    type: Number,
    default: 0,
  },
  playbackSpeed: {
    type: Number,
    default: 1.0,
  },
})

const emit = defineEmits(['play-toggle', 'pause', 'seek', 'set-playback-speed'])

const speedOptions = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]

const handleSpeedChange = event => {
  const value = Number(event.target.value)
  if (Number.isFinite(value)) {
    emit('set-playback-speed', value)
  }
}

const humanDuration = seconds => {
  const boundedSeconds = seconds || 0
  return new Date(boundedSeconds * 1000).toISOString().slice(11, 19)
}
</script>
