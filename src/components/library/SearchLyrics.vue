<template>
  <div>
    <div class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-30" :class="{ 'hidden': !props.isShow }">
      <div class="w-full h-[95vh] max-w-screen-sm rounded-lg m-4 bg-white flex flex-col gap-2">
        <div class="flex-none flex justify-between items-center px-6 py-2">
          <div class="text-thin text-xl text-brave-15">Search Lyrics</div>
          <button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="close"><Close /></button>
        </div>

        <div class="px-6 grow overflow-hidden flex flex-col gap-4">
          <form @submit.prevent="doSearchLyrics" class="flex flex-col gap-4">
            <div class="grid grid-col-2 gap-2">
              <div class="col-span-2">
                <label for="title" class="group-label mb-1">Title</label>
                <input type="text" id="title" v-model="title" class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block w-full p-1.5" placeholder="Title">
              </div>

              <div>
                <label for="albumName" class="group-label mb-1">Album</label>
                <input type="text" id="artistName" v-model="albumName" class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block w-full p-1.5" placeholder="Album">
              </div>

              <div>
                <label for="artistName" class="group-label mb-1">Artist</label>
                <input type="text" id="artistName" v-model="artistName" class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block w-full p-1.5" placeholder="Artist">
              </div>
            </div>

            <div class="col-span-2 flex justify-center">
              <button class="button button-primary rounded-full text-xs px-6 py-2">Search</button>
            </div>
          </form>

          <div class="grow">
            <div v-if="loading" class="flex justify-center items-center h-full">
              <Loading class="animate-spin" />
            </div>

            <div v-else class="flex flex-col h-full gap-2">
              <div v-if="searchResult && searchResult.length" class="flex flex-col gap-1">
                <div v-for="item in searchResult" :key="item.id" class="rounded bg-brave-98 hover:bg-brave-95 transition px-2 py-1 flex gap-2">
                  <div class="grow">
                    <div class="text-sm font-bold">
                      <span class="mr-2 text-brave-30">{{ item.name }}</span>
                      <span v-if="item.syncedLyrics" class="text-green-200 font-bold text-[0.65rem] bg-green-800 rounded px-1 py-0.5">Synced</span>
                      <span v-else-if="item.plainLyrics" class="text-gray-200 font-bold text-[0.65rem] bg-gray-800 rounded px-1 py-0.5">Plain</span>
                      <span v-else-if="item.instrumental" class="text-gray-200 font-bold text-[0.65rem] bg-gray-500 rounded px-1 py-0.5">Instrumental</span>
                      <span v-if="item.duration - Math.round(searchingTrack.duration) > 0" class="ml-1 text-yellow-800 text-[0.75rem]">
                        +{{ humanDuration(Math.abs(item.duration - Math.round(searchingTrack.duration))) }}
                      </span>
                      <span v-else-if="item.duration - Math.round(searchingTrack.duration) < 0" class="ml-1 text-yellow-800 text-[0.75rem]">
                        -{{ humanDuration(Math.abs(item.duration - Math.round(searchingTrack.duration))) }}
                      </span>
                    </div>
                    <div class="text-sm text-brave-35 truncate"><span>{{ item.albumName }}</span> | <span>{{ item.artistName }}</span></div>
                  </div>

                  <div class="flex gap-2 items-center">
                    <button class="px-2 py-1 bg-brave-90 hover:bg-brave-80 active:bg-brave-70 text-brave-30 text-sm font-bold rounded transition" @click="preview(item)">Preview</button>
                    <button class="px-2 py-1 bg-brave-40 hover:bg-brave-35 active:bg-brave-30 text-brave-95 text-sm font-bold rounded transition" @click="apply(item)">Apply</button>
                  </div>
                </div>
              </div>

              <div v-else class="flex justify-center items-center h-full text-sm text-gray-700">
                There is no lyrics record that matches your search
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="fixed top-0 left-0 h-full w-full z-20 bg-black/30" :class="{ 'hidden': !props.isShow }" @click="close">
    </div>
  </div>

  <Teleport to="body">
    <Preview :is-show="!!previewingTrack" :track="previewingTrack" :lyrics="previewingLyrics" @close="closePreview" />
  </Teleport>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted } from 'vue'
import { Close, Loading } from 'mdue'
import { useToast } from 'vue-toastification'
import { useSearchLyrics } from '@/composables/search-lyrics.js'
import Preview from './search-lyrics/Preview.vue'

const toast = useToast()
const props = defineProps(['isShow'])
const { searchingTrack } = useSearchLyrics()
const loading = ref(true)
const exactMatch = ref(null)
const searchResult = ref(null)
const previewingLyrics = ref(null)
const previewingTrack = ref(null)

const close = () => {
  searchingTrack.value = null
}

const title = ref('')
const albumName = ref('')
const artistName = ref('')

const humanDuration = (seconds) => {
  return new Date(seconds * 1000).toISOString().slice(14, 19)
}

const doSearchLyrics = async () => {
  try {
    searchResult.value = await invoke('search_lyrics', { title: title.value, albumName: albumName.value, artistName: artistName.value })
  } catch (error) {
    console.error(error)
    toast.error(error)
  } finally {
    loading.value = false
  }
}

const preview = (lyricsItem) => {
  previewingTrack.value = searchingTrack.value
  previewingLyrics.value = lyricsItem
}

const closePreview = () => {
  previewingTrack.value = null
  previewingLyrics.value = null
}

const apply = async (lyricsItem) => {
  try {
    const result = await invoke('apply_lyrics', { trackId: searchingTrack.value.id, lrclibResponse: lyricsItem })
    toast.success(result)
  } catch (error) {
    console.error(error)
    toast.error(error)
  } finally {
    // close()
  }
}

onMounted(async () => {
  title.value = searchingTrack.value.title
  albumName.value = searchingTrack.value.album_name
  artistName.value = searchingTrack.value.artist_name

  try {
    exactMatch.value = await invoke('retrieve_lyrics', { title: title.value, albumName: albumName.value, artistName: artistName.value, duration: searchingTrack.value.duration })
  } catch {
    exactMatch.value = null
  }

  doSearchLyrics()
})
</script>
