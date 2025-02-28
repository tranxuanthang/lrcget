<template>
  <div ref="parentRef" class="p-4 overflow-y-auto h-full" v-show="props.isActive">
    <div
      :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }"
    >
      <div class="w-full">
        <div class="w-full flex">
          <div class="text-xs text-brave-30/70 font-bold flex w-full dark:text-brave-95">
            <div class="text-left flex-none w-[65%] p-1">Album</div>
            <div class="text-right flex-none w-[15%] p-1"></div>
          </div>
        </div>
        <div class="w-full flex flex-col">
          <div
            v-for="virtualRow in virtualRows"
            :key="virtualRow.index"
            class="group flex flex-col w-full absolute top-0 left-0"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
            }"
          >
            <AlbumItem
              :albumId="virtualRow.key"
              @open-album="openAlbum"
            />
          </div>
        </div>
      </div>
    </div>

    <Transition name="slide-fade">
      <AlbumTrackList v-if="currentAlbum" :album="currentAlbum" @back="currentAlbum = null" />
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import AlbumItem from './album-list/AlbumItem.vue'
import AlbumTrackList from './album-list/AlbumTrackList.vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { invoke } from '@tauri-apps/api/core'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps({
  isActive: {
    type: Boolean,
    default: false
  }
})

const albumIds = ref([])
const parentRef = ref(null)
const currentAlbum = ref(null)
const loading = ref(false)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: albumIds.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 52,
    overscan: 5,
    paddingStart: 32,
    getItemKey: (index) => albumIds.value[index]
  }))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())
const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const openAlbum = async (album) => {
  currentAlbum.value = album
}

const loadAlbums = async () => {
  if (loading.value) return
  loading.value = true

  try {
    albumIds.value = await invoke('get_album_ids')
  } catch (error) {
    console.error('Failed to load albums:', error)
  } finally {
    loading.value = false
  }
}

const { addToQueue } = useDownloader()

const downloadLyricsMultiple = async (albumId) => {
  const config = await invoke('get_config')
  const trackIds = await invoke('get_album_track_ids', {
    albumId: albumId,
    withoutPlainLyrics: config.skip_tracks_with_plain_lyrics,
    withoutSyncedLyrics: config.skip_tracks_with_synced_lyrics
  })
  addToQueue(trackIds)
}

onMounted(async () => {
  if (props.isActive) {
    await loadAlbums()
  }
})

watch(() => props.isActive, async (isActive) => {
  if (isActive) {
    await loadAlbums()
  }
})
</script>

<style scoped>
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s ease;
}
.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
</style>
