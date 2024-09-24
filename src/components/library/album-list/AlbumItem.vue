<template>
  <div class="flex w-full group hover:bg-brave-98 hover:shadow hover:shadow-brave-95/50
    border border-transparent hover:border-brave-95 transition rounded cursor-default dark:hover:bg-brave-5 dark:hover:border-brave-30 dark:hover:shadow-brave-30/50"
  >
    <div v-if="album" class="p-1 flex flex-col grow" @click="$emit('openAlbum', album)">
      <div class="font-bold text-sm text-brave-20 dark:text-brave-95">{{ album.name }}</div>

      <div class="flex items-center gap-2">
        <div class="text-sm text-brave-30 group-hover:text-brave-20 transition dark:text-brave-90 dark:group-hover:text-brave-90">{{ album.tracks_count }} tracks</div>
        <div class="border-r border-brave-80 h-3 flex-none"></div>
        <div class="text-sm text-brave-30 group-hover:text-brave-20 transition dark:text-brave-90 dark:group-hover:text-brave-90">{{ album.artist_name }}</div>
      </div>
    </div>

    <div class="flex items-center gap-2 p-1">
      <div v-if="album" class="transition gap-1">
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition dark:text-white dark:hover:bg-brave-30 dark:hover:text-white" @click.prevent="downloadLyricsMultiple"><DownloadMultiple /></button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { DownloadMultiple } from 'mdue'
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps(['albumId'])
defineEmits(['openAlbum'])

const { addToQueue } = useDownloader()

const album = ref(null)

const downloadLyricsMultiple = async () => {
  const trackIds = await invoke('get_album_track_ids', { albumId: album.value.id })
  addToQueue(trackIds)
}

onMounted(async () => {
  album.value = await invoke('get_album', { albumId: props.albumId })
})
</script>
