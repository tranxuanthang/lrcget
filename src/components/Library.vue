<template>
  <div v-if="!isLoading" class="flex flex-col w-full h-screen">
    <LibraryHeader
      :activeTab="activeTab"
      @changeActiveTab="changeActiveTab"
      @showConfig="isShowConfig = true"
      @showDownloadViewer="isShowDownloadViewer = true"
      @downloadAllLyrics="downloadAllLyrics"
    />

    <div class="relative grow overflow-hidden">
      <TrackList
        :isActive="activeTab === 'tracks'"
      />

      <AlbumList
        :isActive="activeTab === 'albums'"
      />

      <ArtistList
        :isActive="activeTab === 'artists'"
      />
    </div>

    <NowPlaying class="flex-none border-t border-brave-90" :playingTrack="playingTrack" :status="status" :duration="duration" :progress="progress" @pause="pause" @resume="resume" @seek="seek" />
  </div>

  <div v-else class="flex flex-col justify-center items-center w-full h-full">
    <div class="animate-spin text-xl text-brave-30"><Loading /></div>
    <div v-if="isInitializing" class="flex flex-col items-center justify-center text-sm text-brave-40">
      <div>Initializing library...</div>
      <div v-if="initializeProgress">{{ initializeProgress.filesScanned }}/{{ initializeProgress.filesCount }} files scanned</div>
    </div>

    <div v-else class="flex flex-col items-center justify-center text-sm text-brave-40">
      <div>Loading library...</div>
    </div>
  </div>

  <DownloadViewer :is-show="isShowDownloadViewer" @close="closeDownloadViewer" />
  <Config :is-show="isShowConfig" @close="isShowConfig = false" @refreshLibrary="refreshLibrary" @uninitialize-library="$emit('uninitializeLibrary')" />

  <Teleport to="body">
    <SearchLyrics v-if="searchingTrack" :is-show="!!searchingTrack" />
  </Teleport>

  <Teleport to="body">
    <EditLyrics v-if="editingTrack" :is-show="!!editingTrack" />
  </Teleport>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { DownloadMultiple, Loading, Check, Cog } from 'mdue'
import _ from 'lodash'
import LibraryHeader from './library/LibraryHeader.vue'
import NowPlaying from './NowPlaying.vue'
import TrackList from './library/TrackList.vue'
import AlbumList from './library/AlbumList.vue'
import ArtistList from './library/ArtistList.vue'
import DownloadViewer from './library/DownloadViewer.vue'
import Config from './library/Config.vue'
import SearchLyrics from './library/SearchLyrics.vue'
import EditLyrics from './library/EditLyrics.vue'
import { useToast } from 'vue-toastification'
import { useDownloader } from '../composables/downloader.js'
import { useSearchLyrics } from '../composables/search-lyrics.js'
import { useEditLyrics } from '../composables/edit-lyrics.js'
import { usePlayer } from '@/composables/player.js'

const toast = useToast()
const { playingTrack, status, duration, progress, playTrack, pause, resume, stop, seek } = usePlayer()
const { addToQueue } = useDownloader()
const { searchingTrack } = useSearchLyrics()
const { editingTrack } = useEditLyrics()
defineEmits(['uninitializeLibrary'])

const isLoading = ref(true)
const isInitializing = ref(false)
const initializeProgress = ref(null)
const activeTab = ref('tracks')
const isShowDownloadViewer = ref(false)
const isShowConfig = ref(false)

const changeActiveTab = (tab) => {
  activeTab.value = tab
}

const downloadAllLyrics = async () => {
  // TODO implement skip not needed tracks
  // const config = await invoke('get_config')
  //
  // if (config.skip_not_needed_tracks) {
  //
  // } else {
  //
  // }

  const downloadTrackIds = await invoke('get_track_ids')
  addToQueue(downloadTrackIds)
}

const closeDownloadViewer = () => {
  isShowDownloadViewer.value = false
}

const refreshLibrary = async () => {
  isLoading.value = true
  isInitializing.value = true

  try {
    listen('initialize-progress', async (event) => {
      initializeProgress.value = event.payload
    })
    await invoke('refresh_library')
    isInitializing.value = false
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when initializing the library. Error: ${error}`)
  } finally {
    isLoading.value = false
    isInitializing.value = false
  }
}

onMounted(async () => {
  const init = await invoke('get_init')
  if (!init) {
    isLoading.value = true
    isInitializing.value = true

    try {
      listen('initialize-progress', async (event) => {
        initializeProgress.value = event.payload
      })
      await invoke('initialize_library')
      isInitializing.value = false
    } catch (error) {
      console.error(error)
      toast.error(`Unknown error happened when initializing the library. Error: ${error}`)
    } finally {
      isLoading.value = false
      isInitializing.value = false
    }
  } else {
    isLoading.value = false
  }
})

onUnmounted(() => {
  stop()
})
</script>
