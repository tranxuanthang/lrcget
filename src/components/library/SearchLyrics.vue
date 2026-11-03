<template>
  <BaseModal
    title="Search Lyrics"
    content-class="w-full h-[80vh] max-w-screen-sm"
    background="non-interactive"
    :lock-scroll="true"
    @close="emit('close')"
  >
    <div class="grow overflow-hidden flex flex-col gap-6">
      <form class="flex flex-col flex-none gap-4" @submit.prevent="doSearchLyrics">
        <div class="grid grid-cols-2 gap-2">
          <div class="col-span-2">
            <label for="title" class="group-label mb-1">Title</label>
            <input
              id="title"
              v-model="title"
              type="text"
              class="input w-full py-1.5 px-2"
              placeholder="Title"
              :disabled="loading"
            />
          </div>

          <div>
            <label for="albumName" class="group-label mb-1">Album</label>
            <input
              id="artistName"
              v-model="albumName"
              type="text"
              class="input w-full py-1.5 px-2"
              placeholder="Album"
              :disabled="loading"
            />
          </div>

          <div>
            <label for="artistName" class="group-label mb-1">Artist</label>
            <input
              id="artistName"
              v-model="artistName"
              type="text"
              class="input w-full py-1.5 p-2"
              placeholder="Artist"
              :disabled="loading"
            />
          </div>
        </div>

        <div class="col-span-2 flex justify-center">
          <button
            class="button rounded-full text-xs px-6 py-2"
            :class="{ 'button-disabled': loading, 'button-primary': !loading }"
            :disabled="loading"
          >
            Search
          </button>
        </div>
      </form>

      <div class="grow overflow-hidden">
        <div v-if="loading" class="flex justify-center items-center h-full">
          <Loading class="animate-spin text-xl text-neutral-800 dark:text-neutral-400" />
        </div>

        <div v-else class="flex flex-col h-full gap-2 overflow-auto">
          <div v-if="results.length" class="flex flex-col gap-1 overflow-auto">
            <div
              v-for="result in results"
              :key="result.id"
              class="rounded bg-white dark:bg-neutral-900 hover:bg-neutral-50 dark:hover:bg-neutral-800 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700 transition px-2 py-1 flex gap-2"
            >
              <div class="h-full overflow-hidden grow">
                <div class="font-bold flex gap-1">
                  <span class="mr-1 text-sm text-neutral-800 dark:text-neutral-200">{{
                    result.trackName
                  }}</span>
                  <span
                    v-if="result.hasWordSync"
                    class="text-indigo-950 font-bold text-[0.65rem] px-1 py-0.5 rounded bg-gradient-to-r from-amber-300 via-pink-400 to-violet-400 ring-1 ring-inset ring-white/40"
                    title="Word-synced lyrics"
                    >Word Synced</span
                  >
                  <span
                    v-else-if="result.hasSyncedLyrics"
                    class="text-white font-bold text-[0.65rem] bg-hoa-1100 rounded px-1 py-0.5"
                    >Synced</span
                  >
                  <span
                    v-else-if="result.hasPlainLyrics"
                    class="text-neutral-800 dark:text-neutral-300 font-bold text-[0.65rem] bg-neutral-200 dark:bg-neutral-700 rounded px-1 py-0.5"
                    >Plain</span
                  >
                  <span
                    v-else-if="result.instrumental"
                    class="text-neutral-900 dark:text-neutral-200 font-bold text-[0.65rem] bg-neutral-300 dark:bg-neutral-600 rounded px-1 py-0.5"
                    >Instrumental</span
                  >
                  <template v-if="result.duration != null">
                    <span
                      v-if="Math.round(result.duration) - Math.round(searchingTrack.duration) > 2"
                      class="text-neutral-500 dark:text-neutral-400 text-[0.75rem]"
                    >
                      +{{
                        humanDuration(
                          Math.abs(result.duration - Math.round(searchingTrack.duration))
                        )
                      }}
                    </span>
                    <span
                      v-else-if="
                        Math.round(result.duration) - Math.round(searchingTrack.duration) < -2
                      "
                      class="text-neutral-500 dark:text-neutral-400 text-[0.75rem]"
                    >
                      -{{
                        humanDuration(
                          Math.abs(result.duration - Math.round(searchingTrack.duration))
                        )
                      }}
                    </span>
                  </template>
                </div>
                <div class="text-sm text-neutral-900 dark:text-neutral-400 truncate">
                  <span>{{ result.albumName }}</span> |
                  <span>{{ result.artistName }}</span>
                </div>
              </div>

              <div class="flex gap-2 items-center">
                <button
                  class="button-tiny"
                  title="Preview this lyrics"
                  :disabled="openingLyricsId !== null"
                  @click="preview(result)"
                >
                  <Loading v-if="openingLyricsId === result.id" class="animate-spin" />
                  <Eye v-else />
                </button>
                <button
                  class="button-tiny"
                  title="Apply this lyrics"
                  :disabled="openingLyricsId !== null"
                  @click="apply(result)"
                >
                  <ContentSave />
                </button>
              </div>
            </div>
          </div>

          <div v-else class="flex justify-center items-center h-full text-sm text-neutral-400">
            There is no lyrics record that matches your search
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { computed, ref, onMounted } from 'vue'
import Loading from '~icons/mdi/loading'
import Eye from '~icons/mdi/eye'
import ContentSave from '~icons/mdi/content-save'
import { useToast } from 'vue-toastification'
import Preview from './search-lyrics/Preview.vue'
import { useModal } from 'vue-final-modal'

const toast = useToast()
const props = defineProps(['searchingTrack'])
const emit = defineEmits(['close'])

const loading = ref(true)
const searchResult = ref([])
const previewingLyrics = ref(null)
const previewingTrack = ref(null)
const openingLyricsId = ref(null)
const lyricsCache = new Map()

const title = ref('')
const albumName = ref('')
const artistName = ref('')

const results = computed(() => (Array.isArray(searchResult.value) ? searchResult.value : []))

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
    },
  },
})

const humanDuration = seconds => {
  return new Date(seconds * 1000).toISOString().slice(14, 19)
}

const doSearchLyrics = async () => {
  loading.value = true
  lyricsCache.clear()
  try {
    searchResult.value = await invoke('search_lyrics', {
      title: title.value,
      albumName: albumName.value,
      artistName: artistName.value,
      q: '',
    })
  } catch (error) {
    console.error(error)
    toast.error(error)
  } finally {
    loading.value = false
  }
}

const fetchLyrics = async id => {
  if (lyricsCache.has(id)) {
    return lyricsCache.get(id)
  }

  const lyrics = await invoke('retrieve_lyrics_by_id', { id })
  lyricsCache.set(id, lyrics)
  return lyrics
}

const preview = async result => {
  if (openingLyricsId.value !== null) {
    return
  }

  openingLyricsId.value = result.id
  try {
    const lyrics = await fetchLyrics(result.id)
    previewingTrack.value = props.searchingTrack
    previewingLyrics.value = lyrics
    openPreviewModal()
  } catch (error) {
    console.error(error)
    toast.error('An error occurred while opening the lyrics. Please try again.')
  } finally {
    openingLyricsId.value = null
  }
}

const apply = async result => {
  try {
    const message = await invoke('apply_lyrics', {
      trackId: props.searchingTrack.id,
      lrclibId: result.id,
    })
    toast.success(message)
  } catch (error) {
    console.error(error)
    toast.error(error)
  }
}

const initialize = async () => {
  if (!props.searchingTrack) {
    return
  }

  searchResult.value = []
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
