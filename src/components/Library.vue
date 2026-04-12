<template>
  <div v-if="!isLoading" class="flex flex-col w-full h-screen">
    <LibraryHeader
      :active-tab="activeTab"
      @change-active-tab="changeActiveTab"
      @show-config="openConfigModal"
      @show-about="openAboutModal"
      @show-download-viewer="openDownloadViewer"
      @refresh-library="refreshLibrary"
      @uninitialize-library="$emit('uninitializeLibrary')"
      @manage-directories="$emit('manageDirectories')"
      @export-all-lyrics="handleExportAllLyrics"
    />

    <div class="relative grow overflow-hidden">
      <TrackList :is-active="activeTab === 'tracks'" />

      <AlbumList :is-active="activeTab === 'albums'" />

      <ArtistList :is-active="activeTab === 'artists'" />

      <MyLrclib :is-active="activeTab === 'my-lrclib'" />
    </div>

    <NowPlaying class="flex-none" />
  </div>

  <div v-else class="flex flex-col justify-center items-center w-full h-full">
    <div class="animate-spin text-xl text-brave-30">
      <Loading />
    </div>
    <div v-if="isScanning" class="flex flex-col items-center justify-center text-sm text-brave-40">
      <div>Scanning library...</div>
      <div v-if="scanProgress" class="mt-1 font-medium">
        {{ scanProgress.message }}
      </div>
    </div>

    <div v-else class="flex flex-col items-center justify-center text-sm text-brave-40">
      <div>Loading library...</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Loading from '~icons/mdi/loading'
import _ from 'lodash'
import LibraryHeader from './library/LibraryHeader.vue'
import NowPlaying from './NowPlaying.vue'
import TrackList from './library/TrackList.vue'
import AlbumList from './library/AlbumList.vue'
import ArtistList from './library/ArtistList.vue'
import MyLrclib from './library/MyLrclib.vue'
import DownloadViewer from './library/DownloadViewer.vue'
import ExportViewer from './library/ExportViewer.vue'
import Config from './library/Config.vue'
import About from './About.vue'
import { useToast } from 'vue-toastification'
import { useModal } from 'vue-final-modal'
import { useExporter } from '@/composables/export.js'

const props = defineProps({
  shouldScan: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['uninitializeLibrary', 'scanComplete', 'manageDirectories'])

const toast = useToast()

const isLoading = ref(true)
const isScanning = ref(false)
const scanProgress = ref(null)
const scanResult = ref(null)
const activeTab = ref('tracks')
let unlistenScanProgress = null
let unlistenScanComplete = null

const { open: openAboutModal, close: closeAboutModal } = useModal({
  component: About,
  attrs: {
    onClose() {
      closeAboutModal()
    },
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
    onManageDirectories() {
      emit('manageDirectories')
    },
  },
})

const { open: openDownloadViewer, close: closeDownloadViewer } = useModal({
  component: DownloadViewer,
  attrs: {
    onClose() {
      closeDownloadViewer()
    },
  },
})

const { open: openExportViewer, close: closeExportViewer } = useModal({
  component: ExportViewer,
  attrs: {
    onClose() {
      closeExportViewer()
    },
  },
})

const {
  addToQueue: addToExportQueue,
  setupEventListeners: setupExportListeners,
  cleanupEventListeners: cleanupExportListeners,
} = useExporter()

const changeActiveTab = tab => {
  activeTab.value = tab
}

const handleExportAllLyrics = async formats => {
  try {
    openExportViewer()
    await setupExportListeners()

    // Get all track IDs that have lyrics
    const trackIds = await invoke('get_track_ids_with_lyrics')

    if (trackIds.length === 0) {
      toast.info('No tracks with lyrics found to export')
      closeExportViewer()
      return
    }

    addToExportQueue(trackIds, formats)
  } catch (error) {
    console.error(error)
    toast.error(`Failed to start export: ${error}`)
    closeExportViewer()
  }
}

const setupScanListeners = async () => {
  // Clean up any existing listeners
  if (unlistenScanProgress) {
    await unlistenScanProgress()
  }
  if (unlistenScanComplete) {
    await unlistenScanComplete()
  }

  // Listen for scan progress updates
  unlistenScanProgress = await listen('scan-progress', event => {
    scanProgress.value = event.payload
  })

  // Listen for scan completion
  unlistenScanComplete = await listen('scan-complete', event => {
    scanResult.value = event.payload
    isScanning.value = false
    isLoading.value = false
    emit('scanComplete')
  })
}

const cleanupScanListeners = async () => {
  if (unlistenScanProgress) {
    await unlistenScanProgress()
    unlistenScanProgress = null
  }
  if (unlistenScanComplete) {
    await unlistenScanComplete()
    unlistenScanComplete = null
  }
}

const scanLibrary = async (isRefresh = false) => {
  isLoading.value = true
  isScanning.value = true
  scanProgress.value = null
  scanResult.value = null

  try {
    await setupScanListeners()
    // Use hash detection by default for accuracy
    await invoke('scan_library', { useHashDetection: false })
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when scanning the library. Error: ${error}`)
    isScanning.value = false
    isLoading.value = false
    emit('scanComplete')
  }
}

const refreshLibrary = async () => {
  await scanLibrary(true)
}

onMounted(async () => {
  const init = await invoke('get_init')
  if (!init || props.shouldScan) {
    // First time initialization or directories changed - run a full scan
    await scanLibrary(false)
  } else {
    isLoading.value = false
  }
})

// Watch for changes to shouldScan prop
watch(
  () => props.shouldScan,
  newValue => {
    if (newValue && !isScanning.value) {
      scanLibrary(false)
    }
  }
)

onUnmounted(async () => {
  await cleanupScanListeners()
  await cleanupExportListeners()
})
</script>
