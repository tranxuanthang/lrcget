<template>
  <div class="flex w-full group hover:bg-brave-98 hover:dark:bg-brave-5 hover:shadow hover:shadow-brave-95/50
    border border-transparent hover:border-brave-95 transition rounded cursor-default"
    :class="{
      'border-brave-95 bg-brave-99': isPlaying
      }"
  >
    <!-- Track title, album, and artist -->
    <div class="flex-none w-[65%] flex p-1" @click="playTrack(track)">
      <div v-if="track">
        <div class="font-bold text-sm text-brave-20 flex items-center">
          <Equalizer v-if="isPlaying && status === 'playing'" class="mr-1 text-brave-20" />
          <span>{{ track.title }}</span>
        </div>

        <div class="flex flex-wrap items-center gap-2 line-clamp-1">
          <span class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ track.album_name }}</span>
          <span class="text-brave-80 h-2 mx-1 flex-none">|</span>
          <span class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ track.artist_name }}</span>
        </div>
      </div>
    </div>

    <!-- Duration -->
    <div class="flex-none w-[10%] flex items-center justify-end p-1" @click="playTrack(track)">
      <div v-if="track" class="text-brave-30 font-bold text-xs text-right">{{ humanDuration(track.duration) }}</div>
    </div>

    <!-- Lyrics indication -->
    <div class="flex-none w-[10%] flex items-center justify-center p-1" @click="playTrack(track)">
      <div v-if="track">
        <span v-if="track.instrumental" class="text-gray-200 font-bold text-[0.67rem] bg-gray-500 rounded px-1 py-0.5">Instrumental</span>
        <span v-else-if="track.lrc_lyrics" class="text-green-200 font-bold text-[0.67rem] bg-green-800 rounded px-1 py-0.5">Synced</span>
        <span v-else-if="track.txt_lyrics" class="text-gray-200 font-bold text-[0.67rem] bg-gray-800 rounded px-1 py-0.5">Plain</span>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex-none w-[15%] h-full flex justify-end items-center p-1">
      <div v-if="track" class="flex justify-end items-center gap-1">
        <button v-if="isPlaying && status ==='playing'" @click.prevent="pause" class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition"><Pause /></button>
        <button v-else-if="isPlaying && status === 'stopped'" @click.prevent="playTrack(track)" class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition"><Replay /></button>
        <button v-else v-on="isPlaying ? {click: resume} : {click: () => playTrack(track)}" class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition"><Play /></button>
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="searchLyrics(track)"><TextSearch /></button>
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="editLyrics(track)"><PlaylistEdit /></button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Play, Pause, TextSearch, PlaylistEdit, Replay } from 'mdue'
import { humanDuration } from '../../../utils/human-duration.js'
import { useSearchLyrics } from '../../../composables/search-lyrics.js'
import { useEditLyrics } from '../../../composables/edit-lyrics.js'
import Equalizer from '@/components/icons/Equalizer.vue'
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { usePlayer } from '@/composables/player.js'

const { playTrack, playingTrack, status, pause, resume } = usePlayer()

const { searchLyrics } = useSearchLyrics()
const { editLyrics } = useEditLyrics()
const props = defineProps(['trackId'])
const track = ref(null)

// const downloadLyrics = () => {
//   addToQueue([track.value.id])
// }

const isPlaying = computed(() => {
  return playingTrack.value && track.value && playingTrack.value.id === track.value.id
})

onMounted(async () => {
  track.value = await invoke('get_track', { trackId: props.trackId })

  listen('reload-track-id', async (event) => {
    const payload = event.payload
    if (track.value.id === payload) {
      track.value = await invoke('get_track', { trackId: props.trackId })
    }
  })
})
</script>
