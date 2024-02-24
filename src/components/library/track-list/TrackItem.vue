<template>
  <td @click="$emit('playTrack')">
    <div class="flex flex-col">
      <div class="font-bold text-sm text-brave-20 group-hover:text-brave-15 transition">{{ props.title }}</div>

      <div class="flex items-center gap-2">
        <div class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ props.albumName }}</div>
        <div class="border-r border-brave-80 h-3 flex-none"></div>
        <div class="text-sm text-brave-20 group-hover:text-brave-15 transition">{{ props.artistName }}</div>
      </div>
    </div>
  </td>

  <td class="text-brave-30 font-bold text-xs text-right" @click="$emit('playTrack')">
    {{ humanDuration(props.duration) }}
  </td>

  <td @click="$emit('playTrack')" class="text-center">
    <span v-if="props.lrcLyrics" class="text-green-200 font-bold text-[0.67rem] bg-green-800 rounded px-1 py-0.5">Synced</span>
    <span v-else-if="props.txtLyrics" class="text-gray-200 font-bold text-[0.67rem] bg-gray-800 rounded px-1 py-0.5">Plain</span>
  </td>

  <td class="text-right">
    <div class="flex justify-end gap-1">
      <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="$emit('playTrack')"><Play /></button>
      <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="searchLyrics(props.track)"><TextSearch /></button>
      <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="editLyrics(props.track)"><PlaylistEdit /></button>
    </div>
  </td>
</template>

<script setup>
import { Play, TextSearch, PlaylistEdit } from 'mdue'
import { humanDuration } from '../../../utils/human-duration.js'
import { useSearchLyrics } from '../../../composables/search-lyrics.js'
import { useEditLyrics } from '../../../composables/edit-lyrics.js'

const { searchLyrics } = useSearchLyrics()
const { editLyrics } = useEditLyrics()
const props = defineProps(['track', 'title', 'albumName', 'artistName', 'txtLyrics', 'lrcLyrics', 'duration'])
defineEmits(['playTrack', 'downloadLyrics'])
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
