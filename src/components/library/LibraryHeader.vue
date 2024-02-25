<template>
  <div class="px-4 py-2 flex flex-col gap-4 flex-none justify-center items-center bg-white">
    <div class="flex justify-between w-full">
      <div class="flex-1"></div>

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

        <button v-if="isDownloading && downloadedCount !== totalCount" class="button button-working px-4 py-1.5 text-xs rounded-full" @click.prevent="$emit('showDownloadViewer')">
          <div class="animate-spin text-sm"><Loading /></div>
          <span>
            Downloading {{ downloadedCount }}/{{ totalCount }}
          </span>
        </button>

        <button v-else-if="isDownloading" class="button button-done px-4 py-1.5 text-xs rounded-full" @click.prevent="$emit('showDownloadViewer')">
          <div class="text-sm"><Check /></div>
          <span>
            Downloaded {{ downloadedCount }}/{{ totalCount }}
          </span>
        </button>

        <button v-else class="button button-primary px-4 py-1.5 text-xs rounded-full" @click.prevent="downloadAllLyrics">
          <div class="text-sm"><DownloadMultiple /></div>
          <span>
            Download all lyrics
          </span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { DownloadMultiple, Loading, Check, Cog, Information } from 'mdue'
import { useDownloader } from '@/composables/downloader.js'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps(['activeTab'])
defineEmits(['changeActiveTab', 'showConfig', 'showAbout', 'showDownloadViewer', 'downloadAllLyrics'])

const { isDownloading, totalCount, downloadedCount, addToQueue } = useDownloader()

const downloadAllLyrics = async () => {
  const config = await invoke('get_config')
  let downloadTrackIds
  if (config.skip_not_needed_tracks) {
    downloadTrackIds = await invoke('get_no_lyrics_track_ids')
  } else {
    downloadTrackIds = await invoke('get_track_ids')
  }
  addToQueue(downloadTrackIds)
}
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
