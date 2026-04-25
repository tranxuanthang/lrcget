<template>
  <div
    class="flex w-full group hover:bg-neutral-50 hover:shadow hover:shadow-neutral-100/50 border border-transparent hover:border-neutral-100 transition rounded cursor-default dark:hover:bg-neutral-900 dark:hover:border-neutral-700 dark:hover:shadow-black/50"
  >
    <div v-if="artist" class="p-1 flex flex-col grow" @click="$emit('openArtist', artist)">
        <div class="font-bold text-sm text-neutral-800 dark:text-neutral-200">
          {{ artist.name }}
        </div>

        <div class="flex items-center gap-2">
          <div
            class="text-sm text-neutral-500 group-hover:text-neutral-500 transition dark:text-neutral-400 dark:group-hover:text-neutral-400"
          >
            {{ artist.tracks_count }} tracks
          </div>
        </div>
    </div>

    <div class="flex items-center gap-2 p-1">
      <div v-if="artist" class="transition gap-1">
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

const props = defineProps(['artistId'])
defineEmits(['openArtist'])

const { addToQueue } = useDownloader()

const artist = ref(null)

const downloadLyricsMultiple = async () => {
  const config = await invoke('get_config')
  const trackIds = await invoke('get_artist_track_ids', {
    artistId: artist.value.id,
    withoutPlainLyrics: config.skip_tracks_with_plain_lyrics,
    withoutSyncedLyrics: config.skip_tracks_with_synced_lyrics,
  })
  addToQueue(trackIds)
}

onMounted(async () => {
  artist.value = await invoke('get_artist', { artistId: props.artistId })
})
</script>
