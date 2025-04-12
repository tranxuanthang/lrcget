<template>
  <BaseModal
    :click-to-close="!isPublishing"
    :esc-to-close="!isPublishing"
    :closeButton="!isPublishing"
    @close="close"
    content-class="max-w-screen-sm max-h-[60vh] flex flex-col"
  >
    <template #default>
      <div
        v-if="lintResult.length"
        class="grow flex flex-col h-full overflow-hidden"
      >
        <div class="mb-4">
          Please fix the following problem(s) before publishing
        </div>

        <div class="grow overflow-y-scroll h-full">
          <table class="lint-result table">
            <thead class="text-xs font-bold">
              <tr>
                <th class="p-1 text-right">Line</th>
                <th class="p-1 text-center">Severity</th>
                <th class="p-1">Message</th>
              </tr>
            </thead>
            <tbody class="text-xs">
              <tr v-for="(problem, index) in lintResult" :key="index">
                <td class="p-1 text-right">{{ problem.line }}</td>
                <td class="p-1 text-center">
                  <span
                    :class="[
                      problem.severity === 'error'
                        ? 'bg-red-200 text-red-800 dark:bg-red-900 dark:text-red-100'
                        : 'bg-yellow-200 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100',
                      'font-bold text-xs px-1 py-0.5 rounded',
                    ]"
                    >{{
                      problem.severity.charAt(0).toUpperCase() +
                      problem.severity.slice(1)
                    }}</span
                  >
                </td>
                <td class="p-1">{{ problem.message }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div v-else class="flex flex-col items-center">
        <div v-if="!isPublishing" class="mb-4">
          Do you want to publish your unsynchronized lyrics of the song
          <strong>{{ title }} - {{ artistName }}</strong> to your current LRCLIB
          instance?
        </div>
        <div v-else class="mb-4">
          Publishing your unsynchronized lyrics of the song
          <strong>{{ title }} - {{ artistName }}</strong
          >...
        </div>

        <table
          v-if="isPublishing"
          class="text-xs table-auto font-mono uppercase"
        >
          <tbody>
            <tr>
              <td class="px-2 py-1">Request challenge...</td>
              <td class="text-right px-2 py-1">
                {{ progress.requestChallenge }}
              </td>
            </tr>

            <tr>
              <td class="px-2 py-1">Solve challenge...</td>
              <td class="text-right px-2 py-1">
                {{ progress.solveChallenge }}
              </td>
            </tr>

            <tr>
              <td class="px-2 py-1">Publish unsynced lyrics...</td>
              <td class="text-right px-2 py-1">{{ progress.publishLyrics }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template #footer>
      <div v-if="lintResult.length" class="flex gap-2 justify-center w-full">
        <button
          class="button button-primary px-8 py-2 rounded-full"
          @click="close"
        >
          Close
        </button>
      </div>

      <div v-else-if="!isPublishing" class="flex gap-2 justify-center w-full">
        <button
          class="button button-primary px-8 py-2 rounded-full"
          @click="publishPlainText"
        >
          Publish Now
        </button>
        <button
          class="button button-secondary px-8 py-2 rounded-full"
          @click="close"
        >
          Cancel
        </button>
      </div>

      <div v-else class="flex gap-2 justify-center w-full">
        <button
          class="button button-disabled px-8 py-2 rounded-full flex gap-3"
          disabled
        >
          <div class="animate-spin"><Loading /></div>
          <div>Publishing</div>
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted, onUnmounted } from "vue";
import { Loading } from "mdue";
import { listen } from "@tauri-apps/api/event";
import { useToast } from "vue-toastification";
import BaseModal from "@/components/common/BaseModal.vue";

const toast = useToast();
const emit = defineEmits(["close"]);
const props = defineProps({
  lintResult: {
    type: Array,
    required: true,
  },
  title: {
    type: String,
    required: true,
  },
  albumName: {
    type: String,
    required: true,
  },
  artistName: {
    type: String,
    required: true,
  },
  duration: {
    type: Number,
    required: true,
  },
  lyrics: {
    type: String,
    required: true,
  },
});

const isPublishing = ref(false);
const isError = ref(false);
const progress = ref({
  requestChallenge: "Pending",
  solveChallenge: "Pending",
  publishLyrics: "Pending",
});

let unlistenPublish = null;

const publishPlainText = async () => {
  isPublishing.value = true;
  const plainLyrics = props.lyrics;
  const syncedLyrics = "";
  try {
    await invoke("publish_lyrics", {
      title: props.title,
      albumName: props.albumName,
      artistName: props.artistName,
      duration: props.duration,
      plainLyrics,
      syncedLyrics,
    });
    toast.success(
      "Your unsynced lyrics has been published successfully! It might take up to 24 hours to be visible on the search results.",
    );
  } catch (error) {
    isError.value = true;
    console.error(error);
    toast.error(typeof error === "string" ? error : "Failed to publish lyrics");
  } finally {
    isPublishing.value = false;
    close();
  }
};

onMounted(async () => {
  unlistenPublish = await listen("publish-lyrics-progress", (event) => {
    progress.value = event.payload;
  });
});

onUnmounted(() => {
  if (unlistenPublish) {
    unlistenPublish();
  }
});

const close = () => {
  if (!isPublishing.value) {
    emit("close");
  }
};
</script>
