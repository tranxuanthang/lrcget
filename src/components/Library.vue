<template>
  <div v-if="!isLoading && !isInitializing" class="flex flex-col w-full h-screen">
    <LibraryHeader
      :activeTab="activeTab"
      @changeActiveTab="changeActiveTab"
      @showConfig="openConfigModal"
      @showAbout="openAboutModal"
      @showDownloadViewer="openDownloadViewer"
      @refreshLibrary="refreshLibrary"
      @rebuildLibrary="rebuildLibrary"
      @uninitializeLibrary="$emit('uninitializeLibrary')"
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
    </div>

    <NowPlaying class="flex-none" />
  </div>

  <div v-else class="flex flex-col justify-center items-center w-full h-full gap-4">
    <div class="animate-spin text-xl text-brave-30"><Loading /></div>
    <div v-if="isInitializing" class="flex flex-col items-center justify-center text-sm text-brave-40 gap-2">
      <div>{{ scanMessage }}</div>
      <div v-if="initializeProgress && initializeProgress.filesCount">{{ initializeProgress.filesScanned }}/{{ initializeProgress.filesCount }} files scanned</div>
      <div v-else-if="initializeProgress && initializeProgress.filesChanged">Found {{ initializeProgress.filesChanged }} changed files, processing {{ initializeProgress.filesProcessed }}/{{ initializeProgress.filesChanged }}</div>
      
      <button 
        v-if="showSkipButton"
        class="button button-normal px-4 py-2 rounded-full text-sm mt-2"
        @click="skipScan"
      >
        Skip scan
      </button>
    </div>

    <div v-else class="flex flex-col items-center justify-center text-sm text-brave-40">
      <div>Loading library...</div>
    </div>
  </div>
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
import { useToast } from 'vue-toastification'
import { useModal } from 'vue-final-modal'

const toast = useToast()
const emit = defineEmits(['uninitializeLibrary'])

const isLoading = ref(true)
const isInitializing = ref(false)
const initializeProgress = ref(null)
const activeTab = ref('tracks')
const scanMessage = ref('Initializing library...')
const scanAborted = ref(false)
const showSkipButton = ref(false)

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

const skipScan = async () => {
  scanAborted.value = true
  showSkipButton.value = false
  isInitializing.value = false
  
  try {
    // Signal backend to cancel the scan
    await invoke('cancel_scan')
    // Backend will rollback transaction and return
  } catch (error) {
    console.error('Failed to cancel scan:', error)
  }
}

const refreshLibrary = async () => {
  isLoading.value = true
  isInitializing.value = true
  scanMessage.value = 'Refreshing library...'

  try {
    const unlisten = await listen('scan-progress', async (event) => {
      initializeProgress.value = event.payload
    })
    
    await invoke('refresh_library')
    unlisten()
    isInitializing.value = false
    
    // Mark that we just completed a scan to skip auto-scan on reload
    sessionStorage.setItem('skip_auto_scan', 'true')
    
    // Notification will be queued and displayed after reload via drain_notifications
    window.location.reload()
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when refreshing the library. Error: ${error}`)
  } finally {
    isLoading.value = false
    isInitializing.value = false
  }
}

const rebuildLibrary = async () => {
  isLoading.value = true
  isInitializing.value = true
  scanMessage.value = 'Rebuilding library...'

  try {
    const unlisten = await listen('initialize-progress', async (event) => {
      initializeProgress.value = event.payload
    })
    await invoke('rebuild_library')
    unlisten()
    isInitializing.value = false
    
    // Mark that we just completed a scan to skip auto-scan on reload
    sessionStorage.setItem('skip_auto_scan', 'true')
    
    window.location.reload()
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when rebuilding the library. Error: ${error}`)
  } finally {
    isLoading.value = false
    isInitializing.value = false
  }
}

onMounted(async () => {
  // Check if we should skip auto-scan (just came from a manual refresh/rebuild)
  const skipAutoScan = sessionStorage.getItem('skip_auto_scan')
  if (skipAutoScan) {
    sessionStorage.removeItem('skip_auto_scan')
    isLoading.value = false
    return
  }

  const init = await invoke('get_init')
  if (!init) {
    // First time initialization - full scan
    isLoading.value = true
    isInitializing.value = true
    scanMessage.value = 'Initializing library...'

    try {
      const unlisten = await listen('initialize-progress', async (event) => {
        initializeProgress.value = event.payload
      })
      await invoke('initialize_library')
      unlisten()
      isInitializing.value = false
    } catch (error) {
      console.error(error)
      toast.error(`Unknown error happened when initializing the library. Error: ${error}`)
    } finally {
      isLoading.value = false
      isInitializing.value = false
    }
  } else {
    // Library already initialized - load UI in background, show scan blocker with skip option
    isLoading.value = false  // Allow components to load data from old DB
    isInitializing.value = true  // Show blocker screen
    showSkipButton.value = true  // Show skip button
    scanMessage.value = 'Checking for library changes...'

    try {
      const unlisten = await listen('scan-progress', async (event) => {
        if (scanAborted.value) {
          unlisten()
          return
        }
        initializeProgress.value = event.payload
      })
      
      await invoke('refresh_library')
      unlisten()
      
      if (!scanAborted.value) {
        // Scan completed successfully - reload to show updated data
        // Toast will appear via queued notifications
        sessionStorage.setItem('skip_auto_scan', 'true')
        window.location.reload()
      }
    } catch (error) {
      if (!scanAborted.value) {
        console.error(error)
        toast.error(`Unknown error happened when scanning the library. Error: ${error}`)
      }
    } finally {
      showSkipButton.value = false
      isInitializing.value = false
    }
  }
})

onUnmounted(() => {
  stop()
})
</script>
