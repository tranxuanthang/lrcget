<template>
  <BaseModal
    content-class="w-full h-[80vh] max-w-screen-sm"
    @close="emit('close')"
    :title="`${props.track.name} - ${props.track.artistName}`"
    body-class="flex flex-col h-full min-h-0"
  >
    <template v-if="props.track.syncedLyrics && props.track.plainLyrics">
      <div class="flex justify-center mb-2">
        <div class="rounded-full p-1 bg-brave-95 text-brave-30 dark:bg-brave-10 dark:text-brave-95 flex justify-center gap-1">
          <button class="button text-xs w-36 px-3 py-1.5 rounded-full" :class="{ 'bg-brave-30 text-white': lyricsType === 'synced', 'hover:bg-brave-90 dark:hover:bg-brave-10': lyricsType !== 'synced' }" @click="lyricsType = 'synced'">Synced Lyrics</button>
          <button class="button text-xs w-36 px-3 py-1.5 rounded-full" :class="{ 'bg-brave-30 text-white': lyricsType !== 'synced', 'hover:bg-brave-90 dark:hover:bg-brave-10': lyricsType === 'synced' }" @click="lyricsType = 'plain'">Plain Lyrics</button>
        </div>
      </div>

      <div v-if="lyricsType === 'synced'" class="grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 whitespace-pre-line p-4 overflow-scroll select-text">
        {{ props.track.syncedLyrics }}
      </div>

      <div v-else-if="lyricsType === 'plain'" class="grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 whitespace-pre-line p-4 overflow-scroll select-text">
        {{ props.track.plainLyrics }}
      </div>
    </template>

    <div v-else-if="props.track.plainLyrics" class="grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 whitespace-pre-line p-4 overflow-scroll">
      {{ props.track.plainLyrics }}
    </div>

    <div v-else-if="props.track.instrumental" class="grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center">
      This track is instrumental
    </div>

    <div v-else class="grow rounded bg-brave-99 text-brave-30 dark:bg-brave-1 dark:text-brave-95 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center">
      There is currently no lyrics submitted for this track
    </div>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps(['track'])
const emit = defineEmits(['close'])

const lyricsType = ref('synced')
</script>
