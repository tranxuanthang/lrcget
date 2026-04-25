<template>
  <div
    class="word-segment absolute flex items-center justify-center px-1 py-1 text-sm select-none h-full"
    :class="segmentClass"
    :style="segmentStyle"
    :title="`${word.text} (${formatTimestampMs(startMs)} - ${formatTimestampMs(endMs)})`"
  >
    <span class="truncate">{{ word.text }}</span>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { formatTimestampMs } from '@/utils/lyricsfile.js'

const props = defineProps({
  word: {
    type: Object,
    required: true,
  },
  wordIndex: {
    type: Number,
    required: true,
  },
  startMs: {
    type: Number,
    default: 0,
  },
  endMs: {
    type: Number,
    default: 0,
  },
  lineStartMs: {
    type: Number,
    default: 0,
  },
  lineEndMs: {
    type: Number,
    default: 0,
  },
  timelineWidth: {
    type: Number,
    default: 0,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
})

const isPlaying = computed(() => {
  // The word is playing when: startMs <= currentTime < endMs
  return props.progressMs >= props.startMs && props.progressMs < props.endMs
})

const segmentClass = computed(() => {
  const baseClasses = [
    'bg-neutral-200 dark:bg-neutral-700',
    'text-neutral-800 dark:text-neutral-300',
    'border-r',
    'border-neutral-300',
    'dark:border-neutral-600',
  ]

  if (isPlaying.value) {
    baseClasses.push(
      'bg-hoa-1100',
      'dark:bg-hoa-1100',
      'text-white',
      'dark:text-white',
      'font-bold',
      'border-r',
      'border-hoa-1100',
      'dark:border-hoa-1100'
    )
  }

  return baseClasses
})

const segmentStyle = computed(() => {
  if (!props.timelineWidth || props.lineEndMs <= props.lineStartMs) {
    return {}
  }

  const duration = props.lineEndMs - props.lineStartMs
  const leftPercent = ((props.startMs - props.lineStartMs) / duration) * 100
  const wordDuration = Math.max(0, props.endMs - props.startMs)
  const widthPercent = (wordDuration / duration) * 100
  const minWidth = Math.max(24, (props.word.text?.length || 1) * 6)

  return {
    left: `${leftPercent}%`,
    width: `${widthPercent}%`,
    minWidth: `${minWidth}px`,
    transition: 'none',
  }
})
</script>

<style scoped>
.word-segment {
  user-select: none;
  touch-action: none;
}
</style>
