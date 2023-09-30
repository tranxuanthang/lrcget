<template>
  <div v-if="!currentArtist">
    <div
      v-for="artist in artists"
      :key="artist.name"
      class="rounded-lg group hover:bg-brave-95 transition px-4 py-1 flex justify-between cursor-pointer"
    >
      <div class="flex flex-col grow" @click="openArtist(artist)">
        <div class="font-bold text-sm text-brave-20 group-hover:text-brave-15 transition">{{ artist.name }}</div>

        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ artist.tracks_count }} tracks</div>
          <!-- <div class="border-r border-brave-80 h-3 flex-none"></div>
                <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ artist.albums_count }} albums</div> -->
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div class="transition gap-1">
          <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" @click.prevent="downloadLyricsMultiple(artist)"><DownloadMultiple /></button>
        </div>
      </div>
    </div>
  </div>

  <div class="w-full h-full" v-else>
    <div v-if="!loading">
      <div class="mb-4 mx-4"><button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="back"><ArrowLeft /></button></div>

      <div class="flex flex-col mb-8 mx-4">
        <div class="text-thin text-xl text-brave-15">
          {{ currentArtist.name }}
        </div>
        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ currentArtist.tracks_count }} tracks</div>
        </div>
      </div>

      <div>
        <table class="w-full">
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
              v-for="track in currentArtistTracks"
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
import { ref } from 'vue'
import TrackItem from './track-list/TrackItem.vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useToast } from 'vue-toastification'

const toast = useToast()
const props = defineProps(['artists'])
const emit = defineEmits(['playTrack', 'downloadLyrics', 'downloadLyricsMultiple'])

const currentArtist = ref(null)
const currentArtistTracks = ref([])
const loading = ref(false)

const openArtist = async (artist) => {
  loading.value = true
  currentArtist.value = artist
  try {
    currentArtistTracks.value = await invoke('get_artist_tracks', { artistId: currentArtist.value.id })
  } catch (error) {
    console.error(error)
    toast.error('Unknown error happened when opening an artist. Please check the console for detail.')
  } finally {
    loading.value = false
  }
}

const back = () => {
  currentArtist.value = null
  currentArtistTracks.value = []
}

const downloadLyricsMultiple = async (artist) => {
  const tracks = await invoke('get_artist_tracks', { artistId: artist.id })
  emit('downloadLyricsMultiple', tracks)
}
</script>

<style scoped>
th {
  @apply px-4 pb-4;
}
</style>
