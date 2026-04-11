<template>
  <div class="secondary-page">
    <div>
     <div class="mb-4">
        <button
          class="secondary-page-back-button"
          @click="$emit('back')"
        >
          <ArrowLeft />
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex justify-center items-center h-full">
      <Loading class="animate-spin" />
    </div>

    <div v-else class="mx-auto max-w-screen-sm">
      <div class="flex flex-col mb-8">
        <div class="text-thin text-xl">
          Searching for <span class="font-bold">{{ keyword }}</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="text-sm text-brave-30 dark:text-brave-80 group-hover:text-brave-20 transition">Found {{ tracks.length }} tracks</div>
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <div v-for="result in normalizedTracks" :key="result.track.id" class="rounded bg-brave-98 hover:bg-brave-95 dark:bg-brave-5 dark:hover:bg-brave-10 transition px-2 py-1 flex gap-2 justify-between items-center">
          <div class="flex flex-col gap-1">
            <div class="flex gap-2 items-center">
              <div class="text-sm font-bold text-brave-30 dark:text-brave-95">{{ result.track.name }}</div>
              <div class="text-[0.65rem] font-bold flex gap-1">
                <span class="bg-brave-90 text-brave-30 px-1 py-0.5 rounded">{{ humanDuration(result.track.duration) }}</span>
                <template v-if="showLineCount === true">
                  <span v-if="!!result.lyrics.syncedLyrics" class="bg-blue-800 text-blue-200 px-1 py-0.5 rounded">{{ countLines(result.lyrics.syncedLyrics) }} Lines</span>
                  <span v-else-if="!!result.lyrics.plainLyrics" class="bg-blue-800 text-blue-200 px-1 py-0.5 rounded">{{ countLines(result.lyrics.plainLyrics) }} Lines</span>
                </template>
                <span v-if="!!result.lyrics.syncedLyrics" class="bg-green-800 text-green-200 px-1 py-0.5 rounded">Synced</span>
                <span v-else-if="!!result.lyrics.plainLyrics" class="bg-gray-800 text-gray-200 px-1 py-0.5 rounded">Plain</span>
                <span v-else-if="!!result.lyrics.instrumental" class="bg-gray-300 text-gray-600 px-1 py-0.5 rounded">Instrumental</span>
              </div>
            </div>
            <div class="text-sm text-brave-35 dark:text-brave-80">{{ result.track.albumName }} - {{ result.track.artistName }}</div>
          </div>
          <div class="flex gap-1 items-center">
            <button
              class="button-tiny"
              type="button"
              @click="setShowingTrack(result.track)"
            >
              <Eye />
            </button>

            <button
              class="button-tiny"
              type="button"
              @click="setEditingTrack(result.track)"
            >
              <PlaylistEdit />
            </button>

            <button
              class="button-tiny"
              type="button"
              @click="flagLyrics(result.track)"
            >
              <Flag />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="isOpeningTrack" class="flex items-center justify-center w-full h-full fixed top-0 left-0 bg-white/50 dark:bg-brave-10/50 transition">
      <Loading class="animate-spin" />
    </div>
  </div>
</template>

<script setup>
import ArrowLeft from '~icons/mdi/arrow-left'
import Loading from '~icons/mdi/loading'
import Eye from '~icons/mdi/eye'
import PlaylistEdit from '~icons/mdi/playlist-edit'
import Flag from '~icons/mdi/flag'
import { computed, ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { countLines } from '@/utils/count-lines.js'
import { humanDuration } from '@/utils/human-duration.js'
import { useToast } from 'vue-toastification'
import { normalizeLrclibLyrics } from '@/utils/lyricsfile.js'
import EditLyrics from './EditLyrics.vue'
import PreviewLyrics from './PreviewLyrics.vue'
import FlagLyrics from './FlagLyrics.vue'
import { useModal } from 'vue-final-modal'

const toast = useToast()

const props = defineProps({
  keyword: {
    type: String,
    required: true
  }
})

const tracks = ref([])
const loading = ref(false)
const isOpeningTrack = ref(false)
const showingTrack = ref(null)
const editingTrack = ref(null)
const flagLyricsTrack = ref(null)
const showLineCount = ref(true)

const normalizedTracks = computed(() => tracks.value.map((track) => ({
  track,
  lyrics: normalizeLrclibLyrics(track)
})))

const { open: openPreviewModal, close: closePreviewModal } = useModal({
  component: PreviewLyrics,
  attrs: {
    track: showingTrack,
    onClose() {
      closePreviewModal()
    },
    onClosed() {
      showingTrack.value = null
    }
  }
})

const { open: openEditLyricsModal, close: closeEditLyricsModal } = useModal({
  component: EditLyrics,
  attrs: {
    editingTrack: editingTrack,
    onClose() {
      closeEditLyricsModal()
    },
    onClosed() {
      editingTrack.value = null
    }
  }
})

const { open: openFlagLyricsModal, close: closeFlagLyricsModal } = useModal({
  component: FlagLyrics,
  attrs: {
    track: flagLyricsTrack,
    onClose() {
      closeFlagLyricsModal()
    }
  }
})

onMounted(async () => {
  const config = await invoke('get_config')
  showLineCount.value = config.show_line_count
  loading.value = true
  try {
    tracks.value = await invoke('search_lyrics', { title: '', albumName: '', artistName: '', q: props.keyword })
  } catch (error) {
    toast.error('An error occurred while searching for lyrics. Please try again.')

    console.error(error)
  } finally {
    loading.value = false
  }
})

const setShowingTrack = async (track) => {
  isOpeningTrack.value = true
  try {
    const refreshedTrack = await invoke('retrieve_lyrics_by_id', { id: track.id })
    showingTrack.value = refreshedTrack
    openPreviewModal()
  } catch (error) {
    toast.error('An error occurred while opening the lyrics. Please try again.')
    console.error(error)
  } finally {
    isOpeningTrack.value = false
  }
}

const setEditingTrack = async (track) => {
  isOpeningTrack.value = true
  try {
    const refreshedTrack = await invoke('retrieve_lyrics_by_id', { id: track.id })
    editingTrack.value = refreshedTrack
    openEditLyricsModal()
    isOpeningTrack.value = false
  } catch (error) {
    toast.error('An error occurred while opening the lyrics. Please try again.')
    console.error(error)
  } finally {
    isOpeningTrack.value = false
  }
}

const flagLyrics = async (track) => {
  flagLyricsTrack.value = track
  openFlagLyricsModal()
}
</script>
