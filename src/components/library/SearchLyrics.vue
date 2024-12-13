<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="modal-content w-full h-[80vh] max-w-screen-sm flex flex-col"
    overlay-transition="fade"
    content-transition="pop-fade"
    background="non-interactive"
    :lock-scroll="true"
  >
    <div class="modal-title-bar">
      <div class="modal-title">Search Lyrics</div>
      <button class="modal-button" @click="emit('close')"><Close /></button>
    </div>

    <div class="px-6 grow overflow-hidden flex flex-col gap-4 pb-6">
      <form @submit.prevent="doSearchLyrics" class="flex flex-col flex-none gap-4">
        <div class="grid grid-cols-2 gap-2">
          <div class="col-span-2">
            <label for="title" class="group-label mb-1">Title</label>
            <input
              type="text"
              id="title"
              v-model="title"
              class="input w-full py-1.5 px-2"
              placeholder="Title"
              :disabled="loading"
            >
          </div>

          <div>
            <label for="albumName" class="group-label mb-1">Album</label>
            <input
              type="text"
              id="artistName"
              v-model="albumName"
              class="input w-full py-1.5 px-2"
              placeholder="Album"
              :disabled="loading"
            >
          </div>

          <div>
            <label for="artistName" class="group-label mb-1">Artist</label>
            <input
              type="text"
              id="artistName"
              v-model="artistName"
              class="input w-full py-1.5 p-2"
              placeholder="Artist"
              :disabled="loading"
            >
          </div>
        </div>

        <div class="col-span-2 flex justify-center">
          <button class="button rounded-full text-xs px-6 py-2" :class="{ 'button-disabled': loading,  'button-primary': !loading }" :disabled="loading">Search</button>
        </div>
      </form>

      <div class="grow overflow-hidden">
        <div v-if="loading" class="flex justify-center items-center h-full">
          <Loading class="animate-spin" />
        </div>

        <div v-else class="flex flex-col h-full gap-2 overflow-auto">
          <div v-if="searchResult && searchResult.length" class="flex flex-col gap-1 overflow-auto">
            <div v-for="item in searchResult" :key="item.id" class="rounded bg-brave-98 hover:bg-brave-95 transition px-2 py-1 flex gap-2">
              <div class="h-full overflow-hidden grow">
                <div class="text-sm font-bold">
                  <span class="mr-2 text-brave-30">{{ item.name }}</span>
                  <span v-if="item.syncedLyrics" class="text-green-200 font-bold text-[0.65rem] bg-green-800 rounded px-1 py-0.5">Synced</span>
                  <span v-else-if="item.plainLyrics" class="text-gray-200 font-bold text-[0.65rem] bg-gray-800 rounded px-1 py-0.5">Plain</span>
                  <span v-else-if="item.instrumental" class="text-gray-200 font-bold text-[0.65rem] bg-gray-500 rounded px-1 py-0.5">Instrumental</span>
                  <span v-if="Math.round(item.duration) - Math.round(searchingTrack.duration) > 2" class="ml-1 text-blue-800 text-[0.75rem]">
                    +{{ humanDuration(Math.abs(item.duration - Math.round(searchingTrack.duration))) }}
                  </span>
                  <span v-else-if="Math.round(item.duration) - Math.round(searchingTrack.duration) < -2" class="ml-1 text-blue-800 text-[0.75rem]">
                    -{{ humanDuration(Math.abs(item.duration - Math.round(searchingTrack.duration))) }}
                  </span>
                </div>
                <div class="text-sm text-brave-35 truncate"><span>{{ item.albumName }}</span> | <span>{{ item.artistName }}</span></div>
              </div>

              <div class="flex gap-2 items-center">
                <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" title="Preview this lyrics" @click="preview(item)"><Eye /></button>
                <button class="text-brave-30 hover:bg-brave-30 hover:text-white rounded p-2 transition" title="Apply this lyrics" @click="apply(item)"><ContentSave /></button>
              </div>
            </div>
          </div>

          <div v-else class="flex justify-center items-center h-full text-sm text-gray-700">
            There is no lyrics record that matches your search
          </div>
        </div>
      </div>
    </div>
  </VueFinalModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted, watch } from 'vue'
import { Close, Loading, Eye, ContentSave } from 'mdue'
import { useToast } from 'vue-toastification'
import Preview from './search-lyrics/Preview.vue'
import { useModal } from 'vue-final-modal'

const toast = useToast()
const props = defineProps(['searchingTrack'])
const emit = defineEmits(['close'])

const loading = ref(true)
const searchResult = ref(null)
const previewingLyrics = ref(null)
const previewingTrack = ref(null)

const title = ref('')
const albumName = ref('')
const artistName = ref('')

const { open: openPreviewModal, close: closePreviewModal } = useModal({
  component: Preview,
  attrs: {
    track: previewingTrack,
    lyrics: previewingLyrics,
    onClose() {
      closePreviewModal()
    },
    onClosed() {
      previewingTrack.value = null
      previewingLyrics.value = null
    }
  },
})

const humanDuration = (seconds) => {
  return new Date(seconds * 1000).toISOString().slice(14, 19)
}

const doSearchLyrics = async () => {
  loading.value = true
  try {
    searchResult.value = await invoke('search_lyrics', { title: title.value, albumName: albumName.value, artistName: artistName.value, q: '' })
  } catch (error) {
    console.error(error)
    toast.error(error)
  } finally {
    loading.value = false
  }
}

const preview = (lyricsItem) => {
  previewingTrack.value = props.searchingTrack
  previewingLyrics.value = lyricsItem
  openPreviewModal()
}

const apply = async (lyricsItem) => {
  try {
    const result = await invoke('apply_lyrics', { trackId: props.searchingTrack.id, lrclibResponse: lyricsItem })
    toast.success(result)
  } catch (error) {
    console.error(error)
    toast.error(error)
  }
}

const initialize = async () => {
  if (!props.searchingTrack) {
    return
  }

  searchResult.value = null
  loading.value = true

  title.value = props.searchingTrack.title
  albumName.value = props.searchingTrack.album_name
  artistName.value = props.searchingTrack.artist_name
}

onMounted(async () => {
  await initialize()
  doSearchLyrics()
})
</script>
