<template>
  <div class="download-status">
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-bold">Lyrics Downloader</h3>
        <div class="flex gap-2">
          <button
            v-if="isDownloading && !isPaused"
            @click="pauseDownloading"
            class="button button-secondary button-sm"
            title="Pause downloading"
          >
            <Pause class="w-4 h-4" />
          </button>

          <button
            v-if="isDownloading && isPaused"
            @click="resumeDownloading"
            class="button button-primary button-sm"
            title="Resume downloading"
          >
            <Play class="w-4 h-4" />
          </button>

          <button
            v-if="isDownloading"
            @click="stopDownloading"
            class="button button-danger button-sm"
            title="Stop downloading"
          >
            <Stop class="w-4 h-4" />
          </button>

          <button
            v-if="downloadedCount > 0 && !isDownloading"
            @click="startOver"
            class="button button-secondary button-sm"
            title="Clear download history"
          >
            <Delete class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div
        v-if="isDownloading || downloadedCount > 0"
        class="download-progress"
      >
        <div class="flex items-center justify-between mb-1 text-sm">
          <span
            >{{ downloadedCount }} of {{ totalCount }} tracks ({{
              Math.floor(downloadProgress * 100)
            }}%)</span
          >
          <span v-if="isDownloading">{{ remainingTime }} remaining</span>
        </div>
        <div
          class="h-2 w-full bg-brave-95 dark:bg-brave-20 rounded-full overflow-hidden"
        >
          <div
            class="h-full bg-brave-40 dark:bg-brave-60"
            :style="`width: ${downloadProgress * 100}%`"
          ></div>
        </div>
      </div>

      <div v-if="downloadedCount > 0" class="download-stats flex gap-4 text-sm">
        <div class="text-green-600">
          <span class="font-bold">{{ successCount }}</span> successful
        </div>
        <div class="text-red-600">
          <span class="font-bold">{{ failureCount }}</span> failed
        </div>
        <div v-if="isDownloading" class="text-blue-600">
          <span class="font-bold">{{ downloadSpeed.toFixed(1) }}</span>
          tracks/min
        </div>
      </div>

      <div v-if="currentItem" class="current-item text-sm">
        <span class="text-brave-50">Currently downloading:</span>
        <span class="font-semibold">{{ currentItem.title }}</span>
        <span class="text-brave-40">by</span>
        <span>{{ currentItem.artist_name }}</span>
      </div>

      <div v-if="log.length > 0" class="download-log mt-2">
        <h4 class="text-sm font-bold mb-1">Recent Activity</h4>
        <div
          class="log-entries h-32 overflow-auto border border-brave-90 dark:border-brave-20 rounded p-1"
        >
          <div
            v-for="(entry, index) in log.slice(0, 50)"
            :key="index"
            class="log-entry text-xs py-0.5 px-1"
            :class="{
              'bg-brave-98 dark:bg-brave-10': index % 2 === 0,
              'text-green-600': entry.status === 'success',
              'text-red-600': entry.status === 'failure',
              'text-yellow-600': entry.status === 'error',
            }"
          >
            <div class="flex justify-between">
              <span class="font-semibold">{{ entry.title || "System" }}</span>
              <span class="text-brave-40">{{
                formatTime(entry.timestamp)
              }}</span>
            </div>
            <div>{{ entry.message }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Pause, Play, Stop, Delete } from "mdue";
import { useDownloader } from "@/composables/downloader";

const {
  isDownloading,
  isPaused,
  downloadQueue,
  downloadedItems,
  downloadProgress,
  successCount,
  failureCount,
  totalCount,
  downloadedCount,
  downloadSpeed,
  remainingTime,
  log,
  currentItem,
  pauseDownloading,
  resumeDownloading,
  stopDownloading,
  startOver,
} = useDownloader();

const formatTime = (timestamp) => {
  if (!timestamp) return "";
  const date = new Date(timestamp);
  return date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};
</script>
