<template>
  <div class="flex w-full group hover:bg-brave-95 transition rounded cursor-default">
    <!-- Track title, album, and artist -->
    <div class="flex-none w-[65%] flex p-1" @click="$emit('playTrack', track)">
      <div v-if="track">
        <div class="font-bold text-sm text-brave-20 group-hover:text-brave-15 transition">{{ track.title }}</div>

        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ track.album_name }}</div>
          <div class="border-r border-brave-80 h-3 flex-none"></div>
          <div class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ track.artist_name }}</div>
        </div>
      </div>
    </div>

    <!-- Duration -->
    <div class="flex-none w-[10%] flex items-center justify-end p-1" @click="$emit('playTrack', track)">
      <div v-if="track" class="text-brave-30 font-bold text-xs text-right">{{ humanDuration(track.duration) }}</div>
    </div>

    <!-- Lyrics indication -->
    <div class="flex-none w-[10%] flex items-center justify-center p-1" @click="$emit('playTrack', track)">
      <div v-if="track">
        <span v-if="track.lrc_lyrics" class="text-green-200 font-bold text-[0.67rem] bg-green-800 rounded px-1 py-0.5">Synced</span>
        <span v-else-if="track.txt_lyrics" class="text-gray-200 font-bold text-[0.67rem] bg-gray-800 rounded px-1 py-0.5">Plain</span>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex-none w-[15%] h-full flex justify-end items-center p-1">
      <div v-if="track" class="flex justify-end items-center gap-1">
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="playTrack(track)"><Play /></button>
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="searchLyrics(track)"><TextSearch /></button>
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="editLyrics(track)"><PlaylistEdit /></button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Play, TextSearch, PlaylistEdit } from 'mdue'
import { humanDuration } from '../../../utils/human-duration.js'
import { useSearchLyrics } from '../../../composables/search-lyrics.js'
import { useEditLyrics } from '../../../composables/edit-lyrics.js'
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { usePlayer } from '@/composables/player.js'

const { playTrack } = usePlayer()

const { searchLyrics } = useSearchLyrics()
const { editLyrics } = useEditLyrics()
const props = defineProps(['trackId'])
defineEmits(['downloadLyrics'])
const track = ref(null)

onMounted(async () => {
  track.value = await invoke('get_track', { trackId: props.trackId })
})
</script>

<style scoped>
td {
  @apply px-4 py-1 group-hover:bg-brave-95 cursor-pointer transition;
}

td:first-child {
  @apply rounded-l-lg;
}

td:last-child {
  @apply rounded-r-lg;
}
</style>
