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
        <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition dark:text-white dark:hover:bg-brave-30 dark:hover:text-white" @click.prevent="downloadLyricsMultiple">
          <DownloadMultiple />
        </button>
      </div>
    </div>
  </div>

  <div
    class="album-item cursor-pointer flex items-center p-2 hover:bg-brave-95 dark:hover:bg-brave-10 rounded"
    @click="handleClick"
  >
    <div class="album-art flex-none mr-3">
      <div
        v-if="album"
        class="w-10 h-10 rounded bg-brave-90 dark:bg-brave-20 flex items-center justify-center overflow-hidden"
      >
        <img
          v-if="album.artwork_url"
          :src="album.artwork_url"
          :alt="album.title"
          class="w-full h-full object-cover"
        />
        <MusicNote v-else class="w-6 h-6 text-brave-40" />
      </div>
      <div v-else class="w-10 h-10 rounded bg-brave-95 dark:bg-brave-15 animate-pulse"></div>
    </div>

    <div class="album-info flex-1 min-w-0">
      <div v-if="album" class="album-title font-medium text-sm truncate">
        {{ album.title }}
      </div>
      <div v-else class="h-4 bg-brave-95 dark:bg-brave-15 w-32 rounded animate-pulse"></div>

      <div v-if="album" class="album-artist text-xs text-brave-40 dark:text-brave-60 truncate">
        {{ album.artist_name }}
      </div>
      <div v-else class="h-3 mt-1 bg-brave-95 dark:bg-brave-15 w-24 rounded animate-pulse"></div>
    </div>

    <div class="flex-none ml-2">
      <div v-if="album" class="text-xs text-brave-50">
        {{ album.track_count }} tracks
      </div>
      <div v-else class="h-3 bg-brave-95 dark:bg-brave-15 w-12 rounded animate-pulse"></div>
    </div>
  </div>
</template>

<script setup>
import { DownloadMultiple, MusicNote } from 'mdue'
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps({
  albumId: {
    type: Number,
    required: true
  }
})

const emit = defineEmits(['openAlbum', 'open-album'])

const { addToQueue } = useDownloader()

const album = ref(null)
const loading = ref(false)

const downloadLyricsMultiple = async () => {
  const config = await invoke('get_config')
  const trackIds = await invoke('get_album_track_ids', {
    albumId: album.value.id,
    withoutPlainLyrics: config.skip_tracks_with_plain_lyrics,
    withoutSyncedLyrics: config.skip_tracks_with_synced_lyrics
  })
  addToQueue(trackIds)
}

const loadAlbum = async () => {
  if (loading.value) return

  try {
    loading.value = true
    album.value = await invoke('get_album_details', { albumId: props.albumId })
  } catch (error) {
    console.error('Failed to load album:', error)
  } finally {
    loading.value = false
  }
}

const handleClick = () => {
  emit('open-album', album.value)
}

onMounted(async () => {
  album.value = await invoke('get_album', { albumId: props.albumId })
  loadAlbum()
})
</script>
