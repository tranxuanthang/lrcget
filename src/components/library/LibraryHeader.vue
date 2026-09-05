<template>
  <div
    id="library-header"
    class="px-4 py-2 h-12 flex justify-between gap-4 flex-none items-stretch"
  >
    <div class="flex-1 ml-2">
      <MiniSearch :active-tab="props.activeTab" />
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
      <div class="w-[2px] h-[70%] bg-neutral-700/50" />

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
        disabled
        @click.prevent="$emit('showDownloadViewer')"
      >
        <div class="animate-spin text-sm">
          <Loading />
        </div>
        <div class="flex gap-1">
          <div>Preparing</div>
        </div>
      </button>

      <button
        v-else-if="isDownloading && downloadedCount !== downloadTotalCount"
        class="button button-working h-full min-w-[12rem] px-2 text-xs rounded-full"
        @click.prevent="$emit('showDownloadViewer')"
      >
        <div class="animate-spin text-sm">
          <Loading />
        </div>
        <div class="flex gap-1">
          <div>Downloading</div>
          <div>{{ downloadedCount }}/{{ downloadTotalCount }}</div>
        </div>
      </button>

      <button
        v-else-if="isDownloading"
        class="button button-done h-full min-w-[12rem] px-2 text-xs rounded-full"
        @click.prevent="$emit('showDownloadViewer')"
      >
        <div class="text-sm">
          <Check />
        </div>
        <span> Downloaded {{ downloadedCount }}/{{ downloadTotalCount }} </span>
      </button>

      <button
        v-else
        class="button button-primary h-full min-w-[12rem] px-2 text-xs rounded-full"
        @click.stop.prevent="openDownloadOptions"
      >
        <DownloadMultiple />
        <span>Download all lyrics</span>
      </button>

      <button
        v-if="isExporting && exportedCount + skippedCount + errorCount < exportTotalCount"
        class="button button-working h-full min-w-[7rem] px-2 text-xs rounded-full"
        @click.prevent="$emit('showExportViewer')"
      >
        <div class="animate-spin text-sm">
          <Loading />
        </div>
        <span>Exporting</span>
      </button>

      <button
        v-else-if="isExporting"
        class="button button-done h-full min-w-[7rem] px-2 text-xs rounded-full"
        @click.prevent="$emit('showExportViewer')"
      >
        <div class="text-sm">
          <Check />
        </div>
        <span>Exported</span>
      </button>

      <VDropdown
        v-else
        ref="exportDropdown"
        theme="lrcget-dropdown"
        placement="bottom-end"
        class="h-full aspect-square"
        @show="openExportOptions"
      >
        <button
          class="button button-normal h-full min-w-[6rem] px-2 text-xs rounded-full"
          title="Export all lyrics"
        >
          <Export />
          <span>Export</span>
        </button>
        <template #popper>
          <div
            class="dropdown-container export-options min-w-[17rem]"
            @keydown.esc="exportDropdown?.hide()"
          >
            <div class="dropdown-section-label">Export all lyrics to tracks' directory:</div>

            <fieldset :disabled="exportLoading || submittingExport">
              <label class="dropdown-item">
                <CheckboxButton
                  id="export-plain-text"
                  v-model="exportDraft.plainText"
                  name="export-plain-text"
                >
                  <span class="dropdown-label">Plain lyrics (.txt)</span>
                </CheckboxButton>
              </label>
              <label class="dropdown-item">
                <CheckboxButton
                  id="export-synced-lrc"
                  v-model="exportDraft.syncedLrc"
                  name="export-synced-lrc"
                >
                  <span class="dropdown-label">Synced lyrics (.lrc)</span>
                </CheckboxButton>
              </label>

              <label class="dropdown-item" :class="{ 'cursor-not-allowed': !tryEmbedLyrics }">
                <CheckboxButton
                  id="embed-into-track"
                  v-model="exportDraft.embedIntoTrack"
                  name="embed-into-track"
                  :disabled="!tryEmbedLyrics"
                >
                  <span class="dropdown-label">Embed into track</span>
                </CheckboxButton>
              </label>
            </fieldset>
            <p v-if="exportError" role="alert" class="options-error">
              {{ exportError }}
            </p>
            <div class="px-2 py-2">
              <button
                class="button w-full text-sm h-8 rounded"
                :class="canExport ? 'button-primary' : 'button-disabled'"
                :disabled="!canExport"
                type="button"
                @click="handleExportClick"
              >
                Export
              </button>
            </div>
          </div>
        </template>
      </VDropdown>

      <VDropdown theme="lrcget-dropdown" placement="top-end" class="h-full aspect-square">
        <button class="button button-normal h-full aspect-square rounded-full">
          <DotsVertical />
        </button>
        <template #popper>
          <div class="dropdown-container">
            <button v-close-popper class="dropdown-item" @click="$emit('refreshLibrary')">
              <Refresh class="text-neutral-800 dark:text-neutral-300" />
              <span class="text-neutral-800 dark:text-neutral-300 text-sm font-bold"
                >Refresh library</span
              >
            </button>
            <button v-close-popper class="dropdown-item" @click="$emit('manageDirectories')">
              <FolderMultiple class="text-neutral-800 dark:text-neutral-300" />
              <span class="text-neutral-800 dark:text-neutral-300 text-sm font-bold"
                >Manage directories</span
              >
            </button>
            <button v-close-popper class="dropdown-item" @click="$emit('showConfig')">
              <Cog class="text-neutral-800 dark:text-neutral-300" />
              <span class="text-neutral-800 dark:text-neutral-300 text-sm font-bold">Settings</span>
            </button>
            <button v-close-popper class="dropdown-item" @click="$emit('showAbout')">
              <Information class="text-neutral-800 dark:text-neutral-300" />
              <span class="text-neutral-800 dark:text-neutral-300 text-sm font-bold">About</span>
            </button>
          </div>
        </template>
      </VDropdown>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import DownloadMultiple from '~icons/mdi/download-multiple'
import Loading from '~icons/mdi/loading'
import Check from '~icons/mdi/check'
import Cog from '~icons/mdi/cog'
import Information from '~icons/mdi/information'
import DotsVertical from '~icons/mdi/dots-vertical'
import Refresh from '~icons/mdi/refresh'
import FolderMultiple from '~icons/mdi/folder-multiple'
import Export from '~icons/mdi/export'
import CheckboxButton from '@/components/common/CheckboxButton.vue'
import { useDownloader } from '@/composables/downloader.js'
import { useExporter } from '@/composables/export.js'
import { useExportPreferences } from '@/composables/export-preferences.js'
import MiniSearch from './MiniSearch.vue'
import { useDownloadOptions, useDownloadTrigger } from '@/composables/download-options.js'

const props = defineProps(['activeTab'])
const emit = defineEmits([
  'changeActiveTab',
  'showConfig',
  'showAbout',
  'showDownloadViewer',
  'refreshLibrary',
  'manageDirectories',
  'exportAllLyrics',
  'showExportViewer',
])

const { refresh: refreshPreferences, save: savePreferences } = useExportPreferences()
const exportDropdown = ref(null)
const exportDraft = ref({ plainText: false, syncedLrc: true, embedIntoTrack: false })
const tryEmbedLyrics = ref(false)
const exportLoading = ref(false)
const submittingExport = ref(false)
const exportError = ref('')

const { isBuildingQueue } = useDownloadOptions()
const openDownloadOptions = useDownloadTrigger(() => ({ type: 'library' }))

const openExportOptions = async () => {
  exportLoading.value = true
  exportError.value = ''
  try {
    const config = await refreshPreferences()
    exportDraft.value = {
      plainText: config.export_txt,
      syncedLrc: config.export_lrc,
      embedIntoTrack: config.export_embedded,
    }
    tryEmbedLyrics.value = config.try_embed_lyrics
  } catch (error) {
    exportError.value = String(error)
  } finally {
    exportLoading.value = false
  }
}

const canExport = computed(
  () =>
    !exportLoading.value &&
    !submittingExport.value &&
    !exportError.value &&
    (exportDraft.value.plainText ||
      exportDraft.value.syncedLrc ||
      (exportDraft.value.embedIntoTrack && tryEmbedLyrics.value))
)

const handleExportClick = async () => {
  if (!canExport.value) return
  submittingExport.value = true
  const draft = { ...exportDraft.value }
  const embedEnabled = tryEmbedLyrics.value
  try {
    await savePreferences({
      exportTxt: draft.plainText,
      exportLrc: draft.syncedLrc,
      ...(embedEnabled ? { exportEmbedded: draft.embedIntoTrack } : {}),
    })
    exportDropdown.value?.hide()
    emit('exportAllLyrics', { ...draft, embedIntoTrack: draft.embedIntoTrack && embedEnabled })
  } catch (error) {
    exportError.value = String(error)
  } finally {
    submittingExport.value = false
  }
}

const { isDownloading, totalCount: downloadTotalCount, downloadedCount } = useDownloader()

const {
  isExporting,
  exportedCount,
  skippedCount,
  errorCount,
  totalCount: exportTotalCount,
} = useExporter()
</script>

<style scoped>
.active-tab {
  @apply text-neutral-900 border-neutral-900 dark:text-white dark:border-neutral-300;
}

.inactive-tab {
  @apply text-neutral-700/50 hover:text-neutral-700/80 border-transparent dark:text-white/50 dark:hover:text-white/80;
}

.tab {
  @apply transition font-extrabold border-b-2 outline-none py-1;
}

.dropdown-container {
  @apply p-1 min-w-[10rem];
}

.export-options {
  @apply max-w-[calc(100vw-1rem)] max-h-[calc(100vh-4rem)] overflow-y-auto text-neutral-900 dark:text-neutral-100;
}

.options-error {
  @apply px-2 py-1 text-xs leading-relaxed break-words text-red-700 dark:text-red-400;
}

.export-options .button-primary {
  @apply bg-hoa-1400 hover:bg-hoa-1500 active:bg-hoa-1500;
}

.export-options .button-disabled {
  @apply text-neutral-500 dark:text-neutral-400;
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700 rounded cursor-pointer h-8 gap-1 w-full;
}

.dropdown-label {
  @apply text-neutral-800 dark:text-neutral-300 text-sm font-bold;
}

.dropdown-section-label {
  @apply text-xs uppercase font-bold text-neutral-900 dark:text-neutral-400 px-2 py-1;
}
</style>
