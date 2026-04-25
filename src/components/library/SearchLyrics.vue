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
          <div v-if="normalizedSearchResult.length" class="flex flex-col gap-1 overflow-auto">
            <div
              v-for="result in normalizedSearchResult"
              :key="result.item.id"
              class="rounded bg-white dark:bg-neutral-900 hover:bg-neutral-50 dark:hover:bg-neutral-800 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700 transition px-2 py-1 flex gap-2"
            >
              <div class="h-full overflow-hidden grow">
                <div class="font-bold flex gap-1">
                  <span class="mr-1 text-sm text-neutral-800 dark:text-neutral-200">{{
                    result.item.name
                  }}</span>
                  <template v-if="showLineCount === true">
                    <span
                      v-if="result.lyrics.syncedLyrics"
                      class="text-neutral-800 dark:text-neutral-300 font-bold text-[0.65rem] bg-neutral-200 dark:bg-neutral-700 rounded px-1 py-0.5"
                      >{{ countLines(result.lyrics.syncedLyrics) }} Lines</span
                    >
                    <span
                      v-else-if="result.lyrics.plainLyrics"
                      class="text-neutral-800 dark:text-neutral-300 font-bold text-[0.65rem] bg-neutral-200 dark:bg-neutral-700 rounded px-1 py-0.5"
                      >{{ countLines(result.lyrics.plainLyrics) }} Lines</span
                    >
                  </template>
                  <span
                    v-if="result.lyrics.syncedLyrics"
                    class="text-white font-bold text-[0.65rem] bg-hoa-1100 rounded px-1 py-0.5"
                    >Synced</span
                  >
                  <span
                    v-else-if="result.lyrics.plainLyrics"
                    class="text-neutral-800 dark:text-neutral-300 font-bold text-[0.65rem] bg-neutral-200 dark:bg-neutral-700 rounded px-1 py-0.5"
                    >Plain</span
                  >
                  <span
                    v-else-if="result.lyrics.instrumental"
                    class="text-neutral-900 dark:text-neutral-200 font-bold text-[0.65rem] bg-neutral-300 dark:bg-neutral-600 rounded px-1 py-0.5"
                    >Instrumental</span
                  >
                  <span
                    v-if="
                      Math.round(result.item.duration) - Math.round(searchingTrack.duration) > 2
                    "
                    class="text-neutral-500 dark:text-neutral-400 text-[0.75rem]"
                  >
                    +{{
                      humanDuration(
                        Math.abs(result.item.duration - Math.round(searchingTrack.duration))
                      )
                    }}
                  </span>
                  <span
                    v-else-if="
                      Math.round(result.item.duration) - Math.round(searchingTrack.duration) < -2
                    "
                    class="text-neutral-500 dark:text-neutral-400 text-[0.75rem]"
                  >
                    -{{
                      humanDuration(
                        Math.abs(result.item.duration - Math.round(searchingTrack.duration))
                      )
                    }}
                  </span>
                </div>
                <div class="text-sm text-neutral-900 dark:text-neutral-400 truncate">
                  <span>{{ result.item.albumName }}</span> |
                  <span>{{ result.item.artistName }}</span>
                </div>
              </div>

              <div class="flex gap-2 items-center">
                <button
                  class="button-tiny"
                  title="Preview this lyrics"
                  @click="preview(result.item)"
                >
                  <Eye />
                </button>
                <button class="button-tiny" title="Apply this lyrics" @click="apply(result.item)">
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
import { countLines } from '@/utils/count-lines.js'
import { normalizeLrclibLyrics } from '@/utils/lyricsfile.js'

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
const showLineCount = ref(true)

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

const normalizedSearchResult = computed(() => {
  if (!Array.isArray(searchResult.value)) {
    return []
  }

  return searchResult.value.map(item => ({
    item,
    lyrics: normalizeLrclibLyrics(item),
  }))
})

const humanDuration = seconds => {
  return new Date(seconds * 1000).toISOString().slice(14, 19)
}

const doSearchLyrics = async () => {
  loading.value = true
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

const preview = lyricsItem => {
  previewingTrack.value = props.searchingTrack
  previewingLyrics.value = lyricsItem
  openPreviewModal()
}

const apply = async lyricsItem => {
  try {
    const result = await invoke('apply_lyrics', {
      trackId: props.searchingTrack.id,
      lrclibResponse: lyricsItem,
    })
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

  const config = await invoke('get_config')
  searchResult.value = null
  loading.value = true

  title.value = props.searchingTrack.title
  albumName.value = props.searchingTrack.album_name
  artistName.value = props.searchingTrack.artist_name
  showLineCount.value = config.show_line_count
}

onMounted(async () => {
  await initialize()
  doSearchLyrics()
})
</script>
