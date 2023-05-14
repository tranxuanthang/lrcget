<template>
  <div>
    <div class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-30" :class="{ 'hidden': !props.isShow }">
      <div class="w-full h-[80vh] max-w-screen-md rounded-lg m-4 bg-white flex flex-col gap-6">
        <div class="flex-none flex justify-between items-center px-6 py-4">
          <div v-if="props.downloadedCount === props.totalCount" class="text-thin text-xl text-brave-15">Downloaded</div>
          <div v-else class="text-thin text-xl text-brave-15">Downloading</div>
          <button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="checkAndClose"><Close /></button>
        </div>

        <div class="px-6 grow flex flex-col gap-4">
          <div class="flex flex-col items-center justify-center gap-1">
            <div class="w-full bg-brave-95 h-1 rounded">
              <div class="bg-brave-30 h-1" :style="{ width: progressWidth }"></div>
            </div>
            <div class="text-[0.7rem] text-brave-30/60 flex gap-3">
              <span>{{ props.successCount }} FOUND</span>
              <span>{{ props.failureCount }} NOT FOUND</span></div>
          </div>

          <OverlayScrollbars class="rounded-lg p-3 bg-brave-98 w-full overflow-auto text-xs grow">
            <div v-for="logItem in log" :key="logItem.title + logItem.artistName" :class="{ 'text-green-800': logItem.status === 'success', 'text-red-800': logItem.status === 'failure' }">
              <strong>{{ logItem.title }} - {{ logItem.artistName }}</strong>:
              <span>{{ logItem.message }}</span>
            </div>
          </OverlayScrollbars>
        </div>

        <div class="px-6 py-4 flex-none flex justify-center">
          <button v-if="finishable" class="button button-primary px-8 py-2 rounded-full" @click="checkAndClose">Finish</button>
          <button v-else class="button button-normal px-8 py-2 rounded-full" @click="stopDownloading">Stop</button>
        </div>
      </div>
    </div>

    <div class="fixed top-0 left-0 h-full w-full z-20 bg-black/30" :class="{ 'hidden': !props.isShow }" @click="checkAndClose">
    </div>
  </div>
</template>

<script setup>
import { computed } from '@vue/reactivity'
import { Close } from 'mdue'

const props = defineProps(['isShow', 'isDownloading', 'downloadQueue', 'downloadedItems', 'downloadProgress', 'successCount', 'failureCount', 'totalCount', 'downloadedCount', 'addToQueue', 'startOver', 'log'])
const emit = defineEmits(['startOver', 'stopDownloading', 'close'])

const progressWidth = computed(() => {
  if (!props.downloadQueue) {
    return '100%'
  }

  return `${props.downloadProgress * 100}%`
})

const finishable = computed(() => {
  return props.downloadedCount === props.totalCount
})

const stopDownloading = () => {
  emit('stopDownloading')
  emit('close')
}

const checkAndClose = () => {
  if (finishable.value) {
    emit('startOver')
    emit('close')
  } else {
    emit('close')
  }
}
</script>
