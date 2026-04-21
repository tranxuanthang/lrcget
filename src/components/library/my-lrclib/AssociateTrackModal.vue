<template>
  <BaseModal
    title="Choose a reference track"
    content-class="w-full max-w-xl h-full max-h-[80vh]"
    body-class="flex flex-col h-full"
    :close-button="true"
    @close="emit('close')"
  >
    <!-- Search Input -->
    <div class="relative flex-0 mb-4">
      <input
        v-model="searchQuery"
        type="text"
        class="w-full input px-10 py-2"
        placeholder="Search tracks in library..."
        autofocus
      />
      <div class="absolute top-0 left-0 w-10 h-full flex items-center justify-center">
        <Magnify class="text-brave-40 dark:text-brave-70" />
      </div>
      <button
        v-if="searchQuery"
        class="absolute top-0 right-0 w-10 h-full flex items-center justify-center text-brave-40 hover:text-brave-30 dark:text-brave-70 dark:hover:text-brave-80"
        @click="searchQuery = ''"
      >
        <Close />
      </button>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading || isPreparingQuery" class="flex justify-center py-8">
      <Loading class="animate-spin text-2xl text-brave-30 dark:text-brave-90" />
    </div>

    <!-- Results List -->
    <div
      v-else-if="!isPreparingQuery && filteredTracks.length > 0"
      class="overflow-y-auto h-full flex flex-col gap-1"
    >
      <button
        v-for="track in filteredTracks.slice(0, 10)"
        :key="track.id"
        class="w-full text-left px-2 py-1 rounded transition flex items-center gap-1"
        :class="[
          selectedTrack?.id === track.id
            ? 'bg-brave-30 text-white dark:bg-brave-80'
            : 'hover:bg-brave-95 dark:hover:bg-brave-10',
        ]"
        @click="selectedTrack = track"
      >
        <div class="flex-1 min-w-0">
          <div class="font-bold text-sm truncate">{{ track.title }}</div>
          <div class="text-xs opacity-75 truncate">
            {{ track.artist_name }} | {{ track.album_name }}
          </div>
        </div>
        <div class="text-xs opacity-75">{{ humanDuration(track.duration) }}</div>
      </button>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="!isPreparingQuery && searchQuery"
      class="h-full text-center py-8 text-brave-40 dark:text-brave-70"
    >
      <Magnify class="text-3xl mx-auto mb-2 opacity-50" />
      <div class="text-sm">No tracks found</div>
    </div>

    <div
      v-else-if="!isPreparingQuery"
      class="h-full text-center py-8 text-brave-40 dark:text-brave-70"
    >
      <Music class="text-3xl mx-auto mb-2 opacity-50" />
      <div class="text-sm">Type to search your library</div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center w-full gap-4">
        <button
          class="button button-normal px-2 py-1 rounded-lg text-xs flex items-center gap-2"
          :disabled="isSelectingFile"
          @click="selectAudioFile"
        >
          <FolderOpen v-if="!isSelectingFile" />
          <Loading v-else class="animate-spin" />
          <span>Choose file...</span>
        </button>

        <button
          class="button px-4 py-2 rounded-full text-sm"
          :class="canEditWithAudio ? 'button-primary' : 'button-disabled'"
          :disabled="!canEditWithAudio"
          @click="editWithAudio"
        >
          Edit with audio
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from 'vue-toastification'
import Loading from '~icons/mdi/loading'
import Magnify from '~icons/mdi/magnify'
import Close from '~icons/mdi/close'
import FolderOpen from '~icons/mdi/folder-open'
import Music from '~icons/mdi/music'
import { humanDuration } from '@/utils/human-duration.js'
import _debounce from 'lodash/debounce'

const props = defineProps({
  lrclibTrack: {
    type: Object,
    required: true,
  },
})

const emit = defineEmits(['close', 'selectTrack', 'selectFile'])

const toast = useToast()

const searchQuery = ref('')
const isLoading = ref(false)
const isPreparingQuery = ref(true)

// Prefill search query with prepared title + artist
const prepareSearchQuery = async () => {
  isPreparingQuery.value = true
  try {
    const prepared = await invoke('prepare_search_query', {
      title: props.lrclibTrack.name,
      artist: '',
    })
    searchQuery.value = prepared
  } catch (error) {
    console.error('Error preparing search query:', error)
    // Fallback to simple concatenation
    searchQuery.value = `${props.lrclibTrack.name}`.toLowerCase()
  } finally {
    isPreparingQuery.value = false
  }
}

prepareSearchQuery()
const allTracks = ref([])
const selectedTrack = ref(null)
const isSelectingFile = ref(false)

// Filter tracks based on search query
const filteredTracks = computed(() => {
  if (!searchQuery.value.trim()) {
    return allTracks.value
  }

  const query = searchQuery.value.toLowerCase()
  return allTracks.value.filter(
    track =>
      track.title.toLowerCase().includes(query) ||
      track.artist_name.toLowerCase().includes(query) ||
      track.album_name.toLowerCase().includes(query)
  )
})

const canEditWithAudio = computed(() => selectedTrack.value !== null)

// Load all tracks on mount
const loadTracks = async () => {
  isLoading.value = true
  try {
    allTracks.value = await invoke('get_tracks')
  } catch (error) {
    console.error('Error loading tracks:', error)
    toast.error('Failed to load tracks')
  } finally {
    isLoading.value = false
  }
}

loadTracks()

const selectAudioFile = async () => {
  isSelectingFile.value = true
  try {
    const filePath = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Audio Files',
          extensions: ['mp3', 'flac', 'ogg', 'm4a', 'wav', 'aac', 'wma', 'opus'],
        },
      ],
    })

    if (filePath) {
      const metadata = await invoke('get_audio_metadata', { filePath })
      emit('selectFile', {
        ...metadata,
        file_path: filePath,
        id: null, // Temporary track
        isTemporary: true,
      })
      emit('close')
    }
  } catch (error) {
    console.error('Error selecting file:', error)
    toast.error('Failed to read audio file')
  } finally {
    isSelectingFile.value = false
  }
}

const editWithAudio = () => {
  if (selectedTrack.value) {
    emit('selectTrack', selectedTrack.value)
    emit('close')
  }
}
</script>
