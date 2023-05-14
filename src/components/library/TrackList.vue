<template>
  <table>
    <thead class="">
      <tr class="text-xs text-brave-30/70 font-bold">
        <th class="text-left">Track</th>
        <th class="text-right">Duration</th>
        <th class="text-left">Lyrics</th>
        <th class="text-right"></th>
      </tr>
    </thead>
    <tbody>
      <TrackItem
        v-for="track in tracks"
        :key="track.file_path"
        :title="track.title"
        :album-name="track.album_name"
        :artist-name="track.artist_name"
        :lrc-lyrics="track.lrc_lyrics"
        :duration="track.duration"
        @play-track="$emit('playTrack', track)"
        @download-lyrics="$emit('downloadLyrics', track)"
      />
    </tbody>
  </table>
</template>

<script setup>
import TrackItem from './track-list/TrackItem.vue'

const props = defineProps(['tracks'])
defineEmits(['playTrack', 'downloadLyrics'])
</script>

<style scoped>
th {
  @apply px-4 pb-4;
}
</style>
