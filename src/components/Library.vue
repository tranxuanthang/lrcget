<template>
  <div v-if="!isLoading" class="flex flex-col w-full h-screen">
    <LibraryHeader
      :activeTab="activeTab"
      @changeActiveTab="changeActiveTab"
      @showConfig="openConfigModal"
      @showAbout="openAboutModal"
      @showDownloadViewer="openDownloadViewer"
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

      <MyLrclib
        :isActive="activeTab === 'my-lrclib'"
      />

      <!-- <div class="absolute top-0 left-0 w-full h-[20px] bg-gradient-to-b from-white pointer-events-none"></div> -->
    </div>

    <NowPlaying class="flex-none" />
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

  <Teleport to="body">
    <EditLyrics v-if="editingTrack" :is-show="!!editingTrack" />
  </Teleport>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Loading } from 'mdue'
import _ from 'lodash'
import LibraryHeader from './library/LibraryHeader.vue'
import NowPlaying from './NowPlaying.vue'
import TrackList from './library/TrackList.vue'
import AlbumList from './library/AlbumList.vue'
import ArtistList from './library/ArtistList.vue'
import MyLrclib from './library/MyLrclib.vue'
import DownloadViewer from './library/DownloadViewer.vue'
import Config from './library/Config.vue'
import About from './About.vue'
import EditLyrics from './library/EditLyrics.vue'
import { useToast } from 'vue-toastification'
import { useEditLyrics } from '../composables/edit-lyrics.js'
import { useModal } from 'vue-final-modal'

const toast = useToast()
const { editingTrack } = useEditLyrics()
const emit = defineEmits(['uninitializeLibrary'])

const isLoading = ref(true)
const isInitializing = ref(false)
const initializeProgress = ref(null)
const activeTab = ref('tracks')

const { open: openAboutModal, close: closeAboutModal } = useModal({
  component: About,
  attrs: {
    onClose() {
      closeAboutModal()
    }
  },
})

const { open: openConfigModal, close: closeConfigModal } = useModal({
  component: Config,
  attrs: {
    onClose() {
      closeConfigModal()
    },
    onRefreshLibrary() {
      refreshLibrary()
    },
    onUninitializeLibrary() {
      emit('uninitializeLibrary')
    }
  },
})

const { open: openDownloadViewer, close: closeDownloadViewer } = useModal({
  component: DownloadViewer,
  attrs: {
    onClose() {
      closeDownloadViewer()
    }
  },
})

const changeActiveTab = (tab) => {
  activeTab.value = tab
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
