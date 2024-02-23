<template>
  <div v-if="!isLoading" class="flex flex-col w-full h-full">
    <div class="px-4 py-2 flex flex-col gap-4 flex-none justify-center items-center border-b border-brave-90">
      <!-- <div class="text-thin text-xl text-brave-15">Found {{ tracks.length }} tracks</div> -->
      <div class="flex justify-between w-full">
        <div class="flex-1"></div>
        <div class="flex-1 flex gap-5 justify-center text-sm">
          <button
            class="tab"
            :class="{'active-tab': activeTab === 'tracks', 'inactive-tab': activeTab !== 'tracks'}"
            @click.prevent="activeTab = 'tracks'"
          >
            Tracks
          </button>
          <button
            class="tab"
            :class="{'active-tab': activeTab === 'albums', 'inactive-tab': activeTab !== 'albums'}"
            @click.prevent="activeTab = 'albums'"
          >
            Albums
          </button>
          <button
            class="tab"
            :class="{'active-tab': activeTab === 'artists', 'inactive-tab': activeTab !== 'artists'}"
            @click.prevent="activeTab = 'artists'"
          >
            Artists
          </button>
        </div>
        <div class="flex-1 flex justify-end items-center gap-1">
          <button class="button button-normal px-4 py-1.5 rounded-full h-full" @click="isShowConfig = true"><Cog /></button>

          <button v-if="isDownloading && downloadedCount !== totalCount" class="button button-working px-4 py-1.5 text-xs rounded-full" @click="isShowDownloadViewer = true">
            <div class="animate-spin text-sm"><Loading /></div>
            <span>
              Downloading {{ downloadedCount }}/{{ totalCount }}
            </span>
          </button>

          <button v-else-if="isDownloading" class="button button-done px-4 py-1.5 text-xs rounded-full" @click="isShowDownloadViewer = true">
            <div class="text-sm"><Check /></div>
            <span>
              Downloaded {{ downloadedCount }}/{{ totalCount }}
            </span>
          </button>

          <button v-else class="button button-primary px-4 py-1.5 text-xs rounded-full" @click="downloadAllLyrics">
            <div class="text-sm"><DownloadMultiple /></div>
            <span>
              Download all lyrics
            </span>
          </button>
        </div>
      </div>
    </div>

    <OverlayScrollbars class="grow p-4 bg-brave-99 h-full overflow-y-auto">
      <div v-show="activeTab === 'tracks'" class="flex flex-col h-full gap-1">
        <TrackList :tracks="tracks" @play-track="playTrack" @download-lyrics="downloadLyrics" />
        <div class="pb-4"></div>
      </div>

      <div v-show="activeTab === 'albums'" class="flex flex-col h-full">
        <AlbumList :albums="albums" @play-track="playTrack" @download-lyrics="downloadLyrics" @download-lyrics-multiple="downloadLyricsMultiple" />
        <div class="pb-4"></div>
      </div>

      <div v-show="activeTab === 'artists'" class="flex flex-col h-full">
        <ArtistList :artists="artists" @play-track="playTrack" @download-lyrics="downloadLyrics" @download-lyrics-multiple="downloadLyricsMultiple" />
        <div class="pb-4"></div>
      </div>
    </OverlayScrollbars>

    <NowPlaying class="border-t border-brave-90" :playingTrack="playingTrack" :status="status" :duration="duration" :progress="progress" @pause="pause" @resume="resume" @seek="seek" />
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

  <DownloadViewer :is-downloading="isDownloading" :download-queue="downloadQueue" :downloaded-items="downloadedItems" :download-progress="downloadProgress" :success-Count="successCount" :failure-count="failureCount" :total-count="totalCount" :downloaded-count="downloadedCount" :log="log" :is-show="isShowDownloadViewer" @start-over="startOver" @stop-downloading="stopDownloading" @close="closeDownloadViewer" />
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
const { isDownloading, downloadQueue, downloadedItems, downloadProgress, successCount, failureCount, totalCount, downloadedCount, addToQueue, startOver, stopDownloading, log } = useDownloader()
const { searchingTrack } = useSearchLyrics()
const { editingTrack } = useEditLyrics()
defineEmits(['uninitializeLibrary'])

const tracks = ref([])
const albums = ref([])
const artists = ref([])
const isLoading = ref(true)
const isInitializing = ref(false)
const initializeProgress = ref(null)
const activeTab = ref('tracks')
const isShowDownloadViewer = ref(false)
const isShowConfig = ref(false)

const downloadLyrics = (track) => {
  addToQueue([track])
}

const downloadLyricsMultiple = (tracks) => {
  addToQueue(tracks)
}

const downloadAllLyrics = async () => {
  const config = await invoke('get_config')

  let downloadTracks = []
  if (config.skip_not_needed_tracks) {
    downloadTracks = tracks.value.filter((track) => !track.lrc_lyrics)
  } else {
    downloadTracks = tracks.value
  }
  addToQueue(downloadTracks)
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
    await retrieveData()
  } catch (error) {
    console.error(error)
    toast.error(`Unknown error happened when initializing the library. Error: ${error}`)
  } finally {
    isLoading.value = false
    isInitializing.value = false
  }
}

const retrieveData = async () => {
  const tracksPromise = invoke('get_tracks')
  const albumsPromise = invoke('get_albums')
  const artistsPromise = invoke('get_artists')

  const libraryResult = await Promise.all([tracksPromise, albumsPromise, artistsPromise])
  tracks.value = libraryResult[0]
  albums.value = libraryResult[1]
  artists.value = libraryResult[2]
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
      await retrieveData()
    } catch (error) {
      console.error(error)
      toast.error(`Unknown error happened when initializing the library. Error: ${error}`)
    } finally {
      isLoading.value = false
      isInitializing.value = false
    }
  } else {
    isLoading.value = true

    try {
      await retrieveData()
    } catch (error) {
      console.error(error)
      toast.error(`Unknown error happened when loading the library. Error: ${error}`)
    } finally {
      isLoading.value = false
    }
  }

  listen('reload-database', async (event) => {
    retrieveData()
  })
})

onUnmounted(() => {
  stop()
})
</script>

<style scoped>
.active-tab {
  @apply text-brave-15 border-brave-15;
}

.inactive-tab {
  @apply text-brave-15/50 hover:text-brave-15/80 border-transparent;
}

.tab {
  @apply transition font-extrabold border-b-2 outline-none py-1;
}
</style>
