<template>
  <div class="px-4 py-2 flex justify-between items-stretch gap-1">
    <div class="flex gap-1">
      <button
        class="button button-normal px-3 py-1 text-lg rounded-full"
        title="Sync line & move next (Alt+Enter)"
        @click="emit('sync-line')"
      >
        <EqualEnter /> <span class="text-xs">Sync Line & Move Next</span>
      </button>
      <button
        class="button button-normal px-3 py-1 text-lg rounded-full"
        title="Sync line (Alt+X)"
        @click="emit('sync-line-only')"
      >
        <Equal />
      </button>
      <button
        class="button button-normal px-3 py-1 text-lg rounded-full"
        title="Rewind line 100ms (Alt+K)"
        @click="emit('rewind')"
      >
        <Minus />
      </button>
      <button
        class="button button-normal px-3 py-1 text-lg rounded-full"
        title="Forward line 100ms (Alt+J)"
        @click="emit('forward')"
      >
        <Plus />
      </button>
      <button
        class="button button-normal px-3 py-1 text-lg rounded-full"
        title="Replay line (Alt+Z)"
        @click="emit('repeat')"
      >
        <MotionPlay />
      </button>
    </div>

    <div>
      <button
        class="button button-warning px-3 py-1 text-lg rounded-full"
        title="Mark track as instrumental"
        @click="emit('mark-instrumental')"
      >
        <Music /> <span class="text-xs">Mark Instrumental</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import Equal from '~icons/mdi/equal'
import MotionPlay from '~icons/mdi/motion-play'
import Minus from '~icons/mdi/minus'
import Plus from '~icons/mdi/plus'
import Music from '~icons/mdi/music'
import EqualEnter from '@/components/icons/EqualEnter.vue'

const emit = defineEmits([
  'sync-line',
  'sync-line-only',
  'rewind',
  'forward',
  'repeat',
  'mark-instrumental',
])
</script>
