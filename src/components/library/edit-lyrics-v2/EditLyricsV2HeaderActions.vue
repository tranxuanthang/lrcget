<template>
  <div class="flex-none flex gap-2 items-center">
    <div class="flex-none flex">
      <VTooltip theme="lrcget-tooltip">
        <button
          class="button text-sm px-5 h-8 w-24 rounded-l-full rounded-r-none border-r inline-flex items-center justify-center gap-1.5"
          :class="{ 'button-primary': isDirty, 'button-disabled': !isDirty }"
          :disabled="!isDirty"
          @click="emit('save')"
        >
          <ContentSave class="text-base" />
          Save
        </button>

        <template #popper>
          <div class="text-xs font-bold">
            Save lyrics
            <span
              class="text-[0.65rem] text-brave-30 bg-brave-95 px-1 rounded-full"
              >Ctrl+S</span
            >
          </div>
        </template>
      </VTooltip>

      <VDropdown
        theme="lrcget-dropdown"
        placement="bottom-start"
        @show="refreshEmbedConfig"
      >
        <button
          class="button text-sm px-2 py-1.5 h-8 rounded-r-full rounded-l-none button-normal"
          type="button"
        >
          <ChevronDown class="text-base" />
        </button>

        <template #popper>
          <div class="dropdown-container">
            <div class="dropdown-section-label">
              Publish lyrics into your LRCLIB instance:
            </div>
            <div class="px-2 py-2">
              <button
                class="button button-primary w-full text-sm h-8 rounded"
                @click="emit('save-and-publish')"
                v-close-popper
              >
                Save and Publish
              </button>
            </div>

            <div class="dropdown-divider" />
            <div class="dropdown-section-label">
              Export to track's directory:
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

            <label
              class="dropdown-item"
              :class="{ 'opacity-50 cursor-not-allowed': !tryEmbedLyrics }"
            >
              <CheckboxButton
                v-model="embedIntoTrack"
                name="embed-into-track"
                id="embed-into-track"
                :disabled="!tryEmbedLyrics"
              >
                <span class="dropdown-label">Embed into track</span>
              </CheckboxButton>
            </label>

            <div class="px-2 py-2">
              <button
                class="button w-full text-sm h-8 rounded"
                :class="
                  hasSelectedExportFormat && !isExporting
                    ? 'button-primary'
                    : 'button-disabled'
                "
                :disabled="!hasSelectedExportFormat || isExporting"
                type="button"
                @click="handleExportClick"
                v-close-popper
              >
                {{
                  isExporting ? "Saving and exporting..." : "Save and export"
                }}
              </button>
            </div>
          </div>
        </template>
      </VDropdown>
    </div>

    <VTooltip theme="lrcget-tooltip">
      <button
        class="button text-sm px-3 py-1.5 h-8 rounded-full button-normal inline-flex items-center justify-center"
        @click="emit('debug')"
      >
        <Bug class="text-base" />
      </button>

      <template #popper>
        <div class="text-xs font-bold">View YAML debug</div>
      </template>
    </VTooltip>
  </div>
</template>

<script setup>
import { computed, ref, onMounted } from "vue";
import ChevronDown from "~icons/mdi/chevron-down";
import ContentSave from "~icons/mdi/content-save";
import Bug from "~icons/mdi/bug";
import CheckboxButton from "@/components/common/CheckboxButton.vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits(["save", "save-and-publish", "export", "debug"]);

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

  emit("export", {
    plainText: exportPlainText.value,
    syncedLrc: exportSyncedLrc.value,
    embedIntoTrack: embedIntoTrack.value,
  });
};

defineProps({
  isDirty: {
    type: Boolean,
    required: true,
  },
  isExporting: {
    type: Boolean,
    default: false,
  },
});
</script>

<style scoped>
.dropdown-container {
  @apply p-1 min-w-[17rem];
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-brave-90 dark:hover:bg-brave-15 rounded cursor-pointer h-8 gap-2 w-full;
}

.dropdown-divider {
  @apply h-px bg-brave-90 dark:bg-brave-15 my-1;
}

.dropdown-label {
  @apply text-brave-20 dark:text-brave-90 text-sm font-bold;
}

.dropdown-section-label {
  @apply text-xs uppercase font-bold text-brave-35 dark:text-brave-70 px-2 py-1;
}
</style>
