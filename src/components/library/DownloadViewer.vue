<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="w-full h-[80vh] max-w-screen-md rounded-lg m-4 bg-white flex flex-col gap-2 overflow-hidden"
    overlay-transition="fade"
    content-transition="pop-fade"
  >
    <div class="modal-title-bar">
      <div v-if="downloadedCount === totalCount" class="modal-title">Downloaded</div>
      <div v-else class="modal-title">Downloading</div>
      <button class="modal-button" @click.prevent="emit('close')"><Close /></button>
    </div>

    <div class="px-6 flex flex-col gap-4 grow overflow-hidden">
      <div class="flex flex-col items-center justify-center gap-1">
        <div class="w-full bg-brave-95 h-1 rounded">
          <div class="bg-brave-30 h-1" :style="{ width: progressWidth }"></div>
        </div>
        <div class="text-[0.7rem] text-brave-30/60 flex gap-3">
          <span>{{ successCount }} FOUND</span>
          <span>{{ failureCount }} NOT FOUND</span>
        </div>
      </div>

      <div class="rounded-lg p-3 bg-brave-98 w-full text-xs grow overflow-auto">
        <div v-for="logItem in log" :key="logItem.title + logItem.artistName" :class="{ 'text-green-800': logItem.status === 'success', 'text-red-800': logItem.status === 'failure' }">
          <strong>{{ logItem.title }} - {{ logItem.artistName }}</strong>:
          <span>{{ logItem.message }}</span>
        </div>
      </div>
    </div>

    <div class="px-6 py-4 flex-none flex justify-center">
      <button v-if="finishable" class="button button-primary px-8 py-2 rounded-full" @click="checkAndClose">Finish</button>
      <button v-else class="button button-normal px-8 py-2 rounded-full" @click="handleStop">Stop</button>
    </div>
  </VueFinalModal>
</template>

<script setup>
import { onUnmounted, computed } from 'vue'
import { Close } from 'mdue'
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

  return `${downloadProgress.value * 100}%`
})

const finishable = computed(() => {
  return downloadedCount.value === totalCount.value
})

const handleStop = () => {
  stopDownloading()
  emit('close')
}

const checkAndClose = () => {
  if (finishable.value) {
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
