<template>
  <div>
    <LyricsViewer v-if="lyrics" :lyrics="lyrics" :duration="duration" :progress="progress" />
    <PlainLyricsViewer v-else-if="plainLyrics" :lyrics="plainLyrics" />
    <div class="bg-brave-95 backdrop-blur px-4 py-3 flex-none flex flex-col justify-center items-center gap-3">
      <div class="w-full flex gap-1 justify-center items-center">
        <div class="flex-none w-12 text-xs text-brave-30">{{ humanDuration(progress) }}</div>
        <Seek class="grow" :duration="duration" :progress="progress" @seek="seek" />
        <div class="flex-none w-12 text-xs text-brave-30">{{ humanDuration(duration) }}</div>
      </div>
      <div class="flex justify-between w-full">
        <div class="basis-1/3 flex-1 grow-0 flex flex-col justify-center items-start gap-0.5">
          <div v-if="playingTrack">
            <div class="text-xs font-bold text-brave-30 line-clamp-1">{{ playingTrack.title }}</div>
            <div class="text-xs text-brave-40 line-clamp-1">{{ playingTrack.album_name }} - {{ playingTrack.artist_name }}</div>
          </div>
        </div>
        <div class="basis-1/3 flex-1 flex justify-center items-center">
          <button v-if="status !== 'playing'" @click.prevent="resume" class="button button-primary text-white p-2 rounded-full text-xl"><Play /></button>
          <button v-else @click.prevent="pause" class="button button-primary text-white p-2 rounded-full text-xl"><Pause /></button>
        </div>
        <div class="basis-1/3 flex-1">
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from '@vue/reactivity'
import Seek from './now-playing/Seek.vue'
import LyricsViewer from './now-playing/LyricsViewer.vue'
import PlainLyricsViewer from './now-playing/PlainLyricsViewer.vue'
import { Play, Pause } from 'mdue'
import { usePlayer } from '@/composables/player.js'

const { playingTrack, status, duration, progress, pause, resume, seek } = usePlayer()

const lyrics = computed(() => {
  if (!playingTrack.value) {
    return null
  }

  return playingTrack.value.lrc_lyrics
})

const plainLyrics = computed(() => {
  if (!playingTrack.value) {
    return null
  }

  return playingTrack.value.txt_lyrics
})

const humanDuration = (seconds) => {
  return new Date(seconds * 1000).toISOString().slice(11, 19)
}
</script>
