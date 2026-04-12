<template>
  <div
    class="px-4 py-2 h-12 flex justify-between gap-4 flex-none items-stretch"
  >
    <div class="flex-1 ml-2">
      <MiniSearch v-if="props.activeTab === 'tracks'" />
    </div>

    <div class="flex-1 flex gap-4 justify-center items-center text-sm">
      <button
        class="tab"
        :class="{
          'active-tab': props.activeTab === 'tracks',
          'inactive-tab': activeTab !== 'tracks',
        }"
        @click.prevent="$emit('changeActiveTab', 'tracks')"
      >
        Tracks
      </button>
      <button
        class="tab"
        :class="{
          'active-tab': props.activeTab === 'albums',
          'inactive-tab': activeTab !== 'albums',
        }"
        @click.prevent="$emit('changeActiveTab', 'albums')"
      >
        Albums
      </button>
      <button
        class="tab"
        :class="{
          'active-tab': props.activeTab === 'artists',
          'inactive-tab': activeTab !== 'artists',
        }"
        @click.prevent="$emit('changeActiveTab', 'artists')"
      >
        Artists
      </button>

      <!-- Create a separator -->
      <div class="w-[2px] h-[70%] bg-brave-30/30" />

      <button
        class="tab"
        :class="{
          'active-tab': props.activeTab === 'my-lrclib',
          'inactive-tab': activeTab !== 'my-lrclib',
        }"
        @click.prevent="$emit('changeActiveTab', 'my-lrclib')"
      >
        LRCLIB
      </button>
    </div>

    <div class="flex-1 flex justify-end items-center gap-1">
      <button
        v-if="isBuildingQueue"
        class="button button-disabled px-4 py-1.5 h-full min-w-[12rem] text-xs rounded-full"
        @click.prevent="$emit('showDownloadViewer')"
        disabled
      >
        <div class="animate-spin text-sm"><Loading /></div>
        <div class="flex gap-1">
          <div>Preparing</div>
        </div>
      </button>

      <button
        v-else-if="isDownloading && downloadedCount !== totalCount"
        class="button button-working h-full min-w-[12rem] px-4 py-1.5 text-xs rounded-full"
        @click.prevent="$emit('showDownloadViewer')"
      >
        <div class="animate-spin text-sm"><Loading /></div>
        <div class="flex gap-1">
          <div>Downloading</div>
          <div>{{ downloadedCount }}/{{ totalCount }}</div>
        </div>
      </button>

      <button
        v-else-if="isDownloading"
        class="button button-done h-full min-w-[12rem] px-4 py-1.5 text-xs rounded-full"
        @click.prevent="$emit('showDownloadViewer')"
      >
        <div class="text-sm"><Check /></div>
        <span> Downloaded {{ downloadedCount }}/{{ totalCount }} </span>
      </button>

      <button
        v-else
        class="button button-primary px-4 py-1.5 h-full min-w-[12rem] text-xs rounded-full"
        @click.prevent="downloadAllLyrics"
      >
        <div class="text-sm"><DownloadMultiple /></div>
        <span>Download all lyrics</span>
      </button>

      <VDropdown
        theme="lrcget-dropdown"
        placement="bottom-end"
        class="h-full aspect-square"
        @show="refreshEmbedConfig"
      >
        <button
          class="button button-normal h-full aspect-square rounded-full"
          title="Export all lyrics"
        >
          <Export />
        </button>
        <template #popper>
          <div class="dropdown-container min-w-[17rem]">
            <div class="dropdown-section-label">
              Export all lyrics to tracks' directory:
            </div>

            <label class="dropdown-item">
              <CheckboxButton
                v-model="exportPlainText"
                name="export-plain-text"
                id="export-plain-text"
              >
                <span class="dropdown-label">Plain lyrics (.txt)</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="exportSyncedLrc"
                name="export-synced-lrc"
                id="export-synced-lrc"
              >
                <span class="dropdown-label">Synced lyrics (.lrc)</span>
              </CheckboxButton>
            </label>

            <label class="dropdown-item" :class="{ 'opacity-50 cursor-not-allowed': !tryEmbedLyrics }">
              <CheckboxButton
                v-model="embedIntoTrack"
                name="embed-into-track"
                id="embed-into-track"
                :disabled="!tryEmbedLyrics"
              >
                <span class="dropdown-label"
                  >Embed into track (best-effort)</span
                >
              </CheckboxButton>
            </label>

            <div class="px-2 py-2">
              <button
                class="button w-full text-sm h-8 rounded"
                :class="
                  hasSelectedExportFormat ? 'button-primary' : 'button-disabled'
                "
                :disabled="!hasSelectedExportFormat"
                type="button"
                @click="handleExportClick"
                v-close-popper
              >
                Export
              </button>
            </div>
          </div>
        </template>
      </VDropdown>

      <VDropdown
        theme="lrcget-dropdown"
        placement="top-end"
        class="h-full aspect-square"
      >
        <button class="button button-normal h-full aspect-square rounded-full">
          <DotsVertical />
        </button>
        <template #popper>
          <div class="dropdown-container">
            <button
              class="dropdown-item"
              @click="$emit('refreshLibrary')"
              v-close-popper
            >
              <Refresh class="text-brave-20 dark:text-brave-90" />
              <span class="text-brave-20 dark:text-brave-90 text-sm font-bold"
                >Refresh library</span
              >
            </button>
            <button
              class="dropdown-item"
              @click="$emit('manageDirectories')"
              v-close-popper
            >
              <FolderMultiple class="text-brave-20 dark:text-brave-90" />
              <span class="text-brave-20 dark:text-brave-90 text-sm font-bold"
                >Manage directories</span
              >
            </button>
            <button
              class="dropdown-item"
              @click="$emit('showConfig')"
              v-close-popper
            >
              <Cog class="text-brave-20 dark:text-brave-90" />
              <span class="text-brave-20 dark:text-brave-90 text-sm font-bold"
                >Settings</span
              >
            </button>
            <button
              class="dropdown-item"
              @click="$emit('showAbout')"
              v-close-popper
            >
              <Information class="text-brave-20 dark:text-brave-90" />
              <span class="text-brave-20 dark:text-brave-90 text-sm font-bold"
                >About</span
              >
            </button>
          </div>
        </template>
      </VDropdown>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import DownloadMultiple from "~icons/mdi/download-multiple";
import Loading from "~icons/mdi/loading";
import Check from "~icons/mdi/check";
import Cog from "~icons/mdi/cog";
import Information from "~icons/mdi/information";
import DotsVertical from "~icons/mdi/dots-vertical";
import Refresh from "~icons/mdi/refresh";
import FolderMultiple from "~icons/mdi/folder-multiple";
import Export from "~icons/mdi/export";
import CheckboxButton from "@/components/common/CheckboxButton.vue";
import { useDownloader } from "@/composables/downloader.js";
import MiniSearch from "./MiniSearch.vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps(["activeTab"]);
const emit = defineEmits([
  "changeActiveTab",
  "showConfig",
  "showAbout",
  "showDownloadViewer",
  "refreshLibrary",
  "manageDirectories",
  "exportAllLyrics",
]);

const exportPlainText = ref(false);
const exportSyncedLrc = ref(false);
const embedIntoTrack = ref(false);
const tryEmbedLyrics = ref(false);

const refreshEmbedConfig = async () => {
  const config = await invoke("get_config");
  tryEmbedLyrics.value = config.try_embed_lyrics;
};

onMounted(refreshEmbedConfig);

const hasSelectedExportFormat = computed(
  () => exportPlainText.value || exportSyncedLrc.value || embedIntoTrack.value,
);

const handleExportClick = () => {
  if (!hasSelectedExportFormat.value) {
    return;
  }

  emit("exportAllLyrics", {
    plainText: exportPlainText.value,
    syncedLrc: exportSyncedLrc.value,
    embedIntoTrack: embedIntoTrack.value,
  });
};

const { isDownloading, totalCount, downloadedCount, addToQueue } =
  useDownloader();

const isBuildingQueue = ref(false);

const downloadAllLyrics = async () => {
  isBuildingQueue.value = true;

  try {
    const config = await invoke("get_config");
    let downloadTrackIds = await invoke("get_track_ids", {
      searchQuery: "",
      syncedLyricsTracks: !config.skip_tracks_with_synced_lyrics,
      plainLyricsTracks: !config.skip_tracks_with_plain_lyrics,
      instrumentalTracks:
        !config.skip_tracks_with_synced_lyrics &&
        !config.skip_tracks_with_plain_lyrics, // Treat instrumental tracks as either synced or plain lyrics tracks
      noLyricsTracks: true,
    });
    addToQueue(downloadTrackIds);
  } catch (error) {
    // TODO handle error by showing an error popup, etc...
    console.error(error);
  } finally {
    isBuildingQueue.value = false;
  }
};
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

.dropdown-container {
  @apply p-1 min-w-[10rem];
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-brave-90 dark:hover:bg-brave-15 rounded cursor-pointer h-8 gap-1 w-full;
}

.dropdown-label {
  @apply text-brave-20 dark:text-brave-90 text-sm font-bold;
}

.dropdown-section-label {
  @apply text-xs uppercase font-bold text-brave-35 dark:text-brave-70 px-2 py-1;
}
</style>
