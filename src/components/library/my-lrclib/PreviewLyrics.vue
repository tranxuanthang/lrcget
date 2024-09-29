<template>
  <BaseModal>
    <div class="w-[80vw] h-[80vh] max-w-screen-lg rounded-lg m-4 bg-white flex flex-col gap-2 select-none">
      <div class="flex flex-col h-full">
        <div class="modal-title-bar">
          <div class="modal-title">{{ props.track.name }} - {{ props.track.artistName }}</div>
          <button class="modal-button" @click="$emit('close')"><Close /></button>
        </div>

        <div class="px-6 pb-6 grow flex flex-col justify-between gap-4 overflow-hidden h-full">
          <template v-if="props.track.syncedLyrics && props.track.plainLyrics">
            <div class="flex justify-center">
              <div class="rounded-full p-1 bg-brave-95 text-brave-30 flex justify-center gap-1">
                <button class="button text-xs w-36 px-3 py-1.5 rounded-full" :class="{ 'bg-brave-30 text-white': lyricsType === 'synced', 'hover:bg-brave-90': lyricsType !== 'synced' }" @click="lyricsType = 'synced'">Synced Lyrics</button>
                <button class="button text-xs w-36 px-3 py-1.5 rounded-full" :class="{ 'bg-brave-30 text-white': lyricsType !== 'synced', 'hover:bg-brave-90': lyricsType === 'synced' }" @click="lyricsType = 'plain'">Plain Lyrics</button>
              </div>
            </div>

            <div v-if="lyricsType === 'synced'" class="grow rounded bg-brave-99 text-brave-30 whitespace-pre-line p-4 overflow-scroll select-text">
              {{ props.track.syncedLyrics }}
            </div>

            <div v-else-if="lyricsType === 'plain'" class="grow rounded bg-brave-99 text-brave-30 whitespace-pre-line p-4 overflow-scroll select-text">
              {{ props.track.plainLyrics }}
            </div>
          </template>

          <div v-else-if="props.track.plainLyrics" class="grow rounded bg-brave-99 text-brave-30 whitespace-pre-line p-4 overflow-scroll">
            {{ props.track.plainLyrics }}
          </div>

          <div v-else-if="props.track.instrumental" class="grow rounded bg-brave-99 text-brave-30 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center">
            This track is instrumental
          </div>

          <div v-else class="grow rounded bg-indigo-50 text-indigo-900 whitespace-pre-line p-4 overflow-scroll italic flex items-center justify-center">
            There is currently no lyrics submitted for this track
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, watch } from 'vue'
import BaseModal from '@/components/ui/BaseModal.vue'
import { Close } from 'mdue'

const props = defineProps(['track'])

const lyricsType = ref('synced')

watch(() => props.track, () => {
  if (props.track) {
    lyricsType.value = 'synced'
  }
})
</script>
