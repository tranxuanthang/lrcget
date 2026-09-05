<template>
  <VDropdown
    theme="lrcget-dropdown"
    placement="bottom-end"
    :reference-node="() => session.anchor"
    :shown="true"
    :triggers="[]"
    :delay="0"
    handle-resize
    @hide="dismiss"
  >
    <span aria-hidden="true" />
    <template #popper>
      <div
        ref="content"
        class="dropdown-container export-options w-80"
        @keydown.esc.stop.prevent="dismiss"
      >
        <div
          v-if="session.target.type !== 'library'"
          class="px-2 pt-1 pb-2 text-sm font-bold text-neutral-800 dark:text-neutral-300 break-words max-h-24 overflow-y-auto"
        >
          {{ session.target.type === 'album' ? 'Album' : 'Artist' }} · {{ session.target.name }}
        </div>
        <fieldset :disabled="downloadLoading || submittingDownload">
          <legend class="dropdown-section-label">Download lyrics for</legend>
          <div class="flex flex-col gap-2 px-2 py-1 text-sm">
            <RadioButton
              id="popup-download-all"
              v-model="downloadDraft.downloadLyricsFor"
              name="popup-download-lyrics-for"
              value="all"
            >
              <span class="dropdown-label">All tracks (overwrite existing lyrics)</span>
            </RadioButton>
            <RadioButton
              id="popup-download-skip-synced"
              v-model="downloadDraft.downloadLyricsFor"
              name="popup-download-lyrics-for"
              value="skipSynced"
            >
              <span class="dropdown-label">Only tracks without synced lyrics</span>
            </RadioButton>
            <RadioButton
              id="popup-download-skip-plain"
              v-model="downloadDraft.downloadLyricsFor"
              name="popup-download-lyrics-for"
              value="skipPlain"
            >
              <span class="dropdown-label">Only tracks without any lyrics</span>
            </RadioButton>
          </div>
        </fieldset>
        <hr class="my-2 border-neutral-200 dark:border-neutral-700" />
        <fieldset :disabled="downloadLoading || submittingDownload">
          <legend class="dropdown-section-label">Export options</legend>
          <label class="dropdown-item">
            <CheckboxButton
              id="auto-export"
              v-model="downloadDraft.autoExportEnabled"
              name="auto-export"
            >
              <span class="dropdown-label">Automatically export lyrics</span>
            </CheckboxButton>
          </label>
          <template v-if="downloadDraft.autoExportEnabled">
            <label class="dropdown-item">
              <CheckboxButton
                id="download-txt"
                v-model="downloadDraft.plainText"
                name="download-txt"
              >
                <span class="dropdown-label">Plain lyrics (.txt)</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                id="download-lrc"
                v-model="downloadDraft.syncedLrc"
                name="download-lrc"
              >
                <span class="dropdown-label">Synced lyrics (.lrc)</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                id="download-embedded"
                v-model="downloadDraft.embedIntoTrack"
                name="download-embedded"
                :disabled="!tryEmbedLyrics"
              >
                <span class="dropdown-label">Embed into track</span>
              </CheckboxButton>
            </label>
          </template>
        </fieldset>
        <p v-if="downloadError" role="alert" class="options-error">
          {{ downloadError }}
        </p>
        <div class="px-2 py-2">
          <button
            class="button w-full text-sm h-8 rounded"
            :class="canDownload ? 'button-primary' : 'button-disabled'"
            :disabled="!canDownload"
            @click="downloadAllLyrics"
          >
            Download
          </button>
        </div>
      </div>
    </template>
  </VDropdown>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import CheckboxButton from '@/components/common/CheckboxButton.vue'
import RadioButton from '@/components/common/RadioButton.vue'
import { useDownloadOptions } from '@/composables/download-options.js'

const props = defineProps({ session: { type: Object, required: true } })
const content = ref(null)
const {
  downloadDraft,
  downloadLoading,
  submittingDownload,
  downloadError,
  tryEmbedLyrics,
  canDownload,
  downloadAllLyrics,
  close,
} = useDownloadOptions()
const dismiss = () => close(props.session)
// Rows may recycle while scrolling; never leave a popup anchored to another target.
const onScroll = event => {
  if (!(event.target instanceof Node) || !content.value?.contains(event.target)) dismiss()
}
const onKeydown = event => {
  if (event.key === 'Escape') dismiss()
}
onMounted(() => {
  window.addEventListener('scroll', onScroll, true)
  window.addEventListener('keydown', onKeydown, true)
})
onBeforeUnmount(() => {
  window.removeEventListener('scroll', onScroll, true)
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<style scoped>
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
