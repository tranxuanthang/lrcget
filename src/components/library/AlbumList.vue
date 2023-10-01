<template>
  <div v-if="!currentAlbum">
    <div
      v-for="album in albums"
      :key="album.name + album.artist_name"
      class="rounded-lg group hover:bg-brave-95 transition px-4 py-1 flex justify-between cursor-pointer"
    >
      <div class="flex flex-col grow" @click="openAlbum(album)">
        <div class="font-bold text-sm text-brave-20 group-hover:text-brave-15 transition">{{ album.name }}</div>

        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ album.tracks_count }} tracks</div>
          <div class="border-r border-brave-80 h-3 flex-none"></div>
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ album.artist_name }}</div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div class="transition gap-1">
          <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="downloadLyricsMultiple(album)"><DownloadMultiple /></button>
        </div>
      </div>
    </div>
  </div>

  <div class="w-full h-full" v-else>
    <div v-if="!loading">
      <div class="mb-4 mx-4"><button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="back"><ArrowLeft /></button></div>

      <div class="flex flex-col mb-8 mx-4">
        <div class="text-thin text-xl text-brave-15">
          {{ currentAlbum.name }}
        </div>
        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ currentAlbum.tracks_count }} tracks</div>
          <div class="border-r border-brave-80 h-3 flex-none"></div>
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ currentAlbum.artist_name }}</div>
        </div>
      </div>

      <div>
        <table class="w-full table-auto">
          <thead class="">
            <tr class="text-xs text-brave-30/70 font-bold">
              <th class="text-left w-full">Track</th>
              <th class="text-right">Duration</th>
              <th class="text-left">Lyrics</th>
              <th class="text-right"></th>
            </tr>
          </thead>
          <tbody>
            <TrackItem
              v-for="track in currentAlbumTracks"
              :key="track.file_path"
              :track="track"
              :title="track.title"
              :album-name="track.album_name"
              :artist-name="track.artist_name"
              :txt-lyrics="track.txt_lyrics"
              :lrc-lyrics="track.lrc_lyrics"
              :duration="track.duration"
              @play-track="$emit('playTrack', track)"
              @download-lyrics="$emit('downloadLyrics', track)"
            />
          </tbody>
        </table>
      </div>
    </div>

    <div v-else class="flex justify-center items-center w-full h-full">
      <div class="animate-spin text-xl"><Loading /></div>
    </div>
  </div>
</template>

<script setup>
import { Loading, ArrowLeft, DownloadMultiple } from 'mdue'
import { ref, onMounted } from 'vue'
import TrackItem from './track-list/TrackItem.vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { useToast } from 'vue-toastification'

const toast = useToast()
const props = defineProps(['albums'])
const emit = defineEmits(['playTrack', 'downloadLyrics', 'downloadLyricsMultiple'])

const currentAlbum = ref(null)
const currentAlbumTracks = ref([])
const loading = ref(false)

const openAlbum = async (album) => {
  loading.value = true
  currentAlbum.value = album
  try {
    currentAlbumTracks.value = await invoke('get_album_tracks', { albumId: currentAlbum.value.id })
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when opening an album. Error: ${error}`)
  } finally {
    loading.value = false
  }
}

const back = () => {
  currentAlbum.value = null
  currentAlbumTracks.value = []
}

const downloadLyricsMultiple = async (album) => {
  const tracks = await invoke('get_album_tracks', { albumId: album.id })
  emit('downloadLyricsMultiple', tracks)
}

onMounted(() => {
  listen('reload-database', async (event) => {
    if (currentAlbum.value) {
      openAlbum(currentAlbum.value)
    }
  })
})
</script>

<style scoped>
th {
  @apply px-4 pb-4;
}
</style>
