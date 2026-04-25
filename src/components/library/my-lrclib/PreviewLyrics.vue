<template>
  <BaseModal
    content-class="w-full h-[80vh] max-w-screen-sm"
    :title="`${props.track.name} - ${props.track.artistName}`"
    body-class="flex flex-col h-full min-h-0"
    @close="emit('close')"
  >
    <template v-if="resolvedLyrics.syncedLyrics && resolvedLyrics.plainLyrics">
      <div class="flex justify-center mb-2">
        <div
          class="rounded-full p-1 bg-neutral-50 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-300 flex justify-center gap-1"
        >
          <button
            class="button text-xs w-36 px-3 py-1.5 rounded-full"
            :class="{
              'bg-hoa-1100 text-white': lyricsType === 'synced',
              'hover:bg-neutral-100 dark:hover:bg-neutral-900': lyricsType !== 'synced',
            }"
            @click="lyricsType = 'synced'"
          >
            Synced Lyrics
          </button>
          <button
            class="button text-xs w-36 px-3 py-1.5 rounded-full"
            :class="{
              'bg-hoa-1100 text-white': lyricsType !== 'synced',
              'hover:bg-neutral-100 dark:hover:bg-neutral-900': lyricsType === 'synced',
            }"
            @click="lyricsType = 'plain'"
          >
            Plain Lyrics
          </button>
        </div>
      </div>

      <LyricsPanel
        :text="lyricsType === 'synced' ? resolvedLyrics.syncedLyrics : resolvedLyrics.plainLyrics"
      />
    </template>

    <LyricsPanel v-else-if="resolvedLyrics.plainLyrics" :text="resolvedLyrics.plainLyrics" />

    <div
      v-else-if="resolvedLyrics.instrumental"
      class="grow rounded bg-white text-neutral-800 dark:bg-neutral-950 dark:text-neutral-200 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center"
    >
      This track is instrumental
    </div>

    <div
      v-else
      class="grow rounded bg-white text-neutral-800 dark:bg-neutral-950 dark:text-neutral-200 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center"
    >
      There is currently no lyrics submitted for this track
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, ref } from 'vue'
import LyricsPanel from './LyricsPanel.vue'
import { normalizeLrclibLyrics } from '@/utils/lyricsfile.js'

const props = defineProps(['track'])
const emit = defineEmits(['close'])

const lyricsType = ref('synced')
const resolvedLyrics = computed(() => normalizeLrclibLyrics(props.track))
</script>
