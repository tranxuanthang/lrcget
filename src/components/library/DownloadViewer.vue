<template>
  <BaseModal
    title="Configuration"
    @close="checkAndClose"
    content-class="w-full h-[80vh] max-w-screen-md"
    body-class="flex flex-col h-full min-h-0 justify-between gap-6"
    :title="isFinished ? 'Downloaded' : 'Downloading'"
  >
    <div class="flex flex-col items-center justify-center gap-1">
      <div class="w-full bg-brave-95 h-1 rounded">
        <div class="bg-brave-30 h-1" :style="{ width: progressWidth }"></div>
      </div>
      <div class="text-[0.7rem] text-brave-30/60 dark:text-brave-95/60 flex gap-3">
        <span>{{ successCount }} FOUND</span>
        <span>{{ failureCount }} NOT FOUND</span>
      </div>
    </div>

    <div class="rounded-lg p-3 bg-brave-98 dark:bg-brave-1 w-full text-xs grow overflow-auto">
      <div
        v-for="logItem in log"
        :key="logItem.title + logItem.artistName"
        :class="{ 'text-green-800 dark:text-green-400': logItem.status === 'success', 'text-red-800 dark:text-red-400': logItem.status === 'failure' }"
      >
        <strong>{{ logItem.title }} - {{ logItem.artistName }}</strong>:
        <span>{{ logItem.message }}</span>
      </div>
    </div>

    <template #footer>
      <div class="flex-none flex justify-center">
        <button v-if="isFinished" class="button button-primary px-8 py-2 rounded-full" @click="checkAndClose">Finish</button>
        <button v-else class="button button-normal px-8 py-2 rounded-full" @click="handleStop">Stop</button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { onUnmounted, computed } from 'vue'
import { useDownloader } from '@/composables/downloader.js'

const {
  downloadQueue,
  downloadProgress,
  successCount,
  failureCount,
  totalCount,
  downloadedCount,
  startOver,
  stopDownloading,
  log
} = useDownloader()

const emit = defineEmits(['close'])

const progressWidth = computed(() => {
  if (!downloadQueue.value) {
    return '100%'
  }

  if (downloadProgress.value > 1.0) {
    return '100%'
  }

  return `${downloadProgress.value * 100}%`
})

const isFinished = computed(() => {
  return downloadedCount.value >= totalCount.value
})

const handleStop = () => {
  stopDownloading()
  emit('close')
}

const checkAndClose = () => {
  if (isFinished.value) {
    startOver()
    emit('close')
  } else {
    emit('close')
  }
}

onUnmounted(() => {
  checkAndClose()
})
</script>
