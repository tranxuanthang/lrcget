<template>
  <div
    class="flex w-full group hover:bg-neutral-50 hover:shadow hover:shadow-neutral-100/50 border border-transparent hover:border-neutral-100 transition rounded cursor-default dark:hover:bg-neutral-900 dark:hover:border-neutral-700 dark:hover:shadow-black/50"
  >
    <div v-if="album" class="p-1 flex flex-col grow" @click="$emit('openAlbum', album)">
        <div class="font-bold text-sm text-neutral-800 dark:text-neutral-200">
          {{ album.name }}
        </div>

        <div class="flex items-center gap-2">
          <div
            class="text-sm text-neutral-500 group-hover:text-neutral-500 transition dark:text-neutral-400 dark:group-hover:text-neutral-400"
          >
            {{ album.tracks_count }} tracks
          </div>
          <div class="border-r border-neutral-300 h-3 flex-none dark:border-neutral-600" />
          <div
            class="text-sm text-neutral-500 group-hover:text-neutral-500 transition dark:text-neutral-400 dark:group-hover:text-neutral-400"
          >
            {{ album.artist_name }}
          </div>
        </div>
    </div>

    <div class="flex items-center gap-2 p-1">
      <div v-if="album" class="transition gap-1">
        <button
          class="text-neutral-800 hover:bg-hoa-1100 hover:text-white rounded p-2 transition dark:text-white dark:hover:bg-hoa-1100 dark:hover:text-white"
          @click.prevent="downloadLyricsMultiple"
        >
          <DownloadMultiple />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import DownloadMultiple from '~icons/mdi/download-multiple'
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps(['albumId'])
defineEmits(['openAlbum'])

const { addToQueue } = useDownloader()

const album = ref(null)

const downloadLyricsMultiple = async () => {
  const config = await invoke('get_config')
  const trackIds = await invoke('get_album_track_ids', {
    albumId: album.value.id,
    withoutPlainLyrics: config.skip_tracks_with_plain_lyrics,
    withoutSyncedLyrics: config.skip_tracks_with_synced_lyrics,
  })
  addToQueue(trackIds)
}

onMounted(async () => {
  album.value = await invoke('get_album', { albumId: props.albumId })
})
</script>
