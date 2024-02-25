<template>
  <div class="flex w-full group hover:bg-brave-98 hover:shadow hover:shadow-brave-95/50
    border border-transparent hover:border-brave-95 transition rounded cursor-default"
  >
    <div v-if="artist" class="p-1 flex flex-col grow" @click="$emit('openArtist', artist)">
      <div class="font-bold text-sm text-brave-20">{{ artist.name }}</div>

      <div class="flex items-center gap-2">
        <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ artist.tracks_count }} tracks</div>
      </div>
    </div>

    <div class="flex items-center gap-2 p-1">
      <div v-if="artist" class="transition gap-1">
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="downloadLyricsMultiple"><DownloadMultiple /></button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { DownloadMultiple } from 'mdue'
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps(['artistId'])
defineEmits(['openArtist'])

const { addToQueue } = useDownloader()

const artist = ref(null)

const downloadLyricsMultiple = async () => {
  const trackIds = await invoke('get_artist_track_ids', { artistId: artist.value.id })
  addToQueue(trackIds)
}

onMounted(async () => {
  artist.value = await invoke('get_artist', { artistId: props.artistId })
})
</script>
