<template>
  <div class="px-4 py-2 h-12 flex justify-between gap-4 flex-none items-stretch">
    <div class="flex-1 ml-2">
      <MiniSearch v-if="props.activeTab === 'tracks'" />
    </div>

    <div class="flex-1 flex gap-5 justify-center text-sm">
      <button
        class="tab"
        :class="{'active-tab': props.activeTab === 'tracks', 'inactive-tab': activeTab !== 'tracks'}"
        @click.prevent="$emit('changeActiveTab', 'tracks')"
      >
        Tracks
      </button>
      <button
        class="tab"
        :class="{'active-tab': props.activeTab === 'albums', 'inactive-tab': activeTab !== 'albums'}"
        @click.prevent="$emit('changeActiveTab', 'albums')"
      >
        Albums
      </button>
      <button
        class="tab"
        :class="{'active-tab': props.activeTab === 'artists', 'inactive-tab': activeTab !== 'artists'}"
        @click.prevent="$emit('changeActiveTab', 'artists')"
      >
        Artists
      </button>
    </div>

    <div class="flex-1 flex justify-end items-center gap-1">
      <button
        class="button button-normal px-4 py-1.5 rounded-full h-full"
        @click="$emit('showAbout')"
      >
        <Information />
      </button>

      <button
        class="button button-normal px-4 py-1.5 rounded-full h-full"
        @click="$emit('showConfig')"
      >
        <Cog />
      </button>

      <button v-if="isBuildingQueue" class="button button-disabled px-4 py-1.5 h-full min-w-[12rem] text-xs rounded-full" @click.prevent="$emit('showDownloadViewer')" disabled>
        <div class="animate-spin text-sm"><Loading /></div>
        <div class="flex gap-1">
          <div>Preparing</div>
        </div>
      </button>

      <button v-else-if="isDownloading && downloadedCount !== totalCount" class="button button-working h-full min-w-[12rem] px-4 py-1.5 text-xs rounded-full" @click.prevent="$emit('showDownloadViewer')">
        <div class="animate-spin text-sm"><Loading /></div>
        <div class="flex gap-1">
          <div>Downloading</div>
          <div>{{ downloadedCount }}/{{ totalCount }}</div>
        </div>
      </button>

      <button v-else-if="isDownloading" class="button button-done h-full min-w-[12rem] px-4 py-1.5 text-xs rounded-full" @click.prevent="$emit('showDownloadViewer')">
        <div class="text-sm"><Check /></div>
        <span>
          Downloaded {{ downloadedCount }}/{{ totalCount }}
        </span>
      </button>

      <button v-else class="button button-primary px-4 py-1.5 h-full min-w-[12rem] text-xs rounded-full" @click.prevent="downloadAllLyrics">
        <div class="text-sm"><DownloadMultiple /></div>
        <span>
          Download all lyrics
        </span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { DownloadMultiple, Loading, Check, Cog, Information, Magnify } from 'mdue'
import { useDownloader } from '@/composables/downloader.js'
import MiniSearch from './MiniSearch.vue'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps(['activeTab'])
defineEmits(['changeActiveTab', 'showConfig', 'showAbout', 'showDownloadViewer', 'showSearch'])

const { isDownloading, totalCount, downloadedCount, addToQueue } = useDownloader()

const isBuildingQueue = ref(false)

const downloadAllLyrics = async () => {
  isBuildingQueue.value = true

  try {
    const config = await invoke('get_config')
    let downloadTrackIds
    if (config.skip_not_needed_tracks) {
      downloadTrackIds = await invoke('get_no_lyrics_track_ids')
    } else {
      downloadTrackIds = await invoke('get_track_ids')
    }
    addToQueue(downloadTrackIds)
  } catch (error) {
    // TODO handle error by showing an error popup, etc...
    console.error(error)
  } finally {
    isBuildingQueue.value = false
  }
}
</script>

<style scoped>
.active-tab {
  @apply text-brave-15 border-brave-15 dark:text-white dark:border-brave-30;
}

.inactive-tab {
  @apply text-brave-15/50 hover:text-brave-15/80 border-transparent dark:text-white/50 dark:hover:text-brave-95/80;
}

.tab {
  @apply transition font-extrabold border-b-2 outline-none py-1;
}
</style>
