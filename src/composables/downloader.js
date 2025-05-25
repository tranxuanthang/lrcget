import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const delay = (time) =>
  new Promise((resolve, reject) => setTimeout(resolve, time));

const downloadQueue = ref([]);
const downloadedItems = ref([]);
const currentItem = ref(null);
const log = ref([]);
const successCount = ref(0);
const failureCount = ref(0);
const isDownloading = ref(false);
const totalCount = ref(0);

const downloadedCount = computed(() => {
  return downloadedItems.value.length;
});

const addLog = (logObj) => {
  log.value.unshift(logObj);
  if (log.value.length > 1000) {
    log.value.pop();
  }
};

const downloadLyrics = async (track) => {
  try {
    const result = await invoke("download_lyrics", { trackId: track.id });

    if (!isDownloading.value) {
      return;
    }

    addLog({
      status: "success",
      title: track.title,
      artistName: track.artist_name,
      message: result,
    });
    successCount.value++;
  } catch (error) {
    if (!isDownloading.value) {
      return;
    }

    addLog({
      status: "failure",
      title: track.title,
      artistName: track.artist_name,
      message: error,
    });
    failureCount.value++;
  }

  downloadedItems.value.push(currentItem.value);
  currentItem.value = null;
};

const downloadNext = async () => {
  while (true) {
    if (downloadQueue.value.length === 0) {
      await delay(1000);
      continue;
    }

    const trackId = downloadQueue.value.shift();
    const track = await invoke("get_track", { trackId: trackId });
    currentItem.value = track;
    await downloadLyrics(track);
  }
};

const downloadProgress = computed(() => {
  if (!downloadQueue.value) {
    return 0.0;
  }

  if (downloadedCount.value >= totalCount.value) {
    return 1.0;
  }

  return downloadedCount.value / totalCount.value;
});

const addToQueue = (trackIds) => {
  isDownloading.value = true;

  for (let i = 0; i < trackIds.length; i++) {
    downloadQueue.value.push(trackIds[i]);
  }

  totalCount.value += trackIds.length;

  console.log(`Added ${totalCount.value} tracks to download queue`);
};

const startOver = () => {
  downloadedItems.value = [];
  log.value = [];
  successCount.value = 0;
  failureCount.value = 0;
  totalCount.value = 0;
  isDownloading.value = false;
};

const stopDownloading = () => {
  downloadQueue.value = [];
  downloadedItems.value = [];
  log.value = [];
  successCount.value = 0;
  failureCount.value = 0;
  totalCount.value = 0;
  isDownloading.value = false;
};

export function useDownloader() {
  return {
    isDownloading,
    downloadQueue,
    downloadedItems,
    downloadProgress,
    successCount,
    failureCount,
    totalCount,
    downloadedCount,
    log,
    addToQueue,
    startOver,
    stopDownloading,
    downloadNext,
  };
}
