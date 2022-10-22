<template>
<div class="flex flex-col w-full h-full">
  <div class="flex-grow flex flex-col justify-center items-center w-full p-6">
    <div v-if="progress < 1.0" class="m-4 text-hoa-1200 text-sm font-bold uppercase">Applying lyrics...</div>
    <div v-else class="m-4 text-hoa-1200 text-sm font-bold uppercase">Done! Happy singing!</div>
    <div class="flex flex-col items-center justify-center mb-4 gap-4 text-gray-200 w-full">
        <div class="w-full bg-hoa-300 h-1 mb-6 rounded">
            <div class="bg-hoa-800 h-1" :style="{ width: `${Math.round(progress * 100)}%` }"></div>
        </div>

        <div class="text-hoa-800 w-full flex justify-start text-xs mb-4">
          <div class="grid w-auto gap-3 grid-cols-2">
            <div class="uppercase font-bold text-hoa-800">Total items</div>
            <div class="font-bold text-right text-hoa-800">{{ totalItemsCount }}</div>
            <div class="uppercase font-bold text-green-800">Successed items</div>
            <div class="font-bold text-right text-green-800">{{ successedItemsCount }}</div>
            <div class="uppercase font-bold text-red-800">Not found/failed items</div>
            <div class="font-bold text-right text-red-800">{{ failedItemsCount }}</div>
          </div>
        </div>

        <OverlayScrollbars class="rounded p-3 h-64 bg-brave-98 w-full font-mono overflow-auto flex flex-col-reverse text-sm"
          :options="{
            scrollbars: { theme: 'os-theme-light' }
          }"
        >
          <div v-for="log in applyLog" :key="log.trackName" :class="{ 'text-green-800': log.status === 'Success', 'text-red-800': log.status === 'Failure' }">
            <strong>{{ log.trackName }} - {{ log.artistName }}</strong>:
            <span>{{ log.message }}</span>
          </div>
        </OverlayScrollbars>
    </div>
  </div>

  <div class="p-6 w-full flex justify-center bg-brave-90">
    <button type="button" @click="startOver"
      class="px-6 py-2 bg-brave-primary rounded-full uppercase font-bold text-white transition"
      :class="{ 'bg-opacity-30': progress < 1.0, 'hover:bg-brave-35 active:bg-brave-30': progress >= 1.0 }"
      :disabled="progress < 1.0"
    >
      Start Over
    </button>
  </div>
</div>
</template>

<script setup>
import { ref, onMounted, defineProps, defineEmits } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

const props = defineProps({
  musicItems: Object,
  isCreateLRC: Boolean,
  isEmbed: Boolean,
  skipTracksHaveExistingLyrics: Boolean
})

const emit = defineEmits(['startOver'])

const progress = ref(0.0)
const processedItemsCount = ref(0)
const successedItemsCount = ref(0)
const failedItemsCount = ref(0)
const totalItemsCount = ref(0)
const applyLog = ref([])

const startOver = () => {
  emit('startOver')
}

onMounted(async () => {
  let musicItems

  if (props.skipTracksHaveExistingLyrics) {
    musicItems = props.musicItems.filter((item) => item.lyrics === null || item.lyrics.length === 0)
  } else {
    musicItems = props.musicItems
  }

  invoke("apply_lyrics", {
    musicItems: musicItems,
    isCreateLrc: props.isCreateLRC,
    isEmbed: props.isEmbed,
    skipTracksHaveExistingLyrics: props.skipTracksHaveExistingLyrics
  }).then(() => {
    console.log('done')
    progress.value = 1.0
  })

  listen('apply-lyrics-progress', (event) => {
    const payload = event.payload
    console.log(payload)
    progress.value = payload.progress
    processedItemsCount.value = payload.processed_items_count
    successedItemsCount.value = payload.successed_items_count
    failedItemsCount.value = payload.failed_items_count
    totalItemsCount.value = payload.total_items_count

    applyLog.value.unshift({
      status: payload.current_status,
      message: payload.current_message,
      trackName: payload.current_track.track_name,
      artistName: payload.current_track.artist_name
    })
  })
})
</script>
