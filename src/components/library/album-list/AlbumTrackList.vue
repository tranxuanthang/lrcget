<template>
  <div ref="parentRef" class="absolute top-0 left-0 w-full h-full bg-white p-4 shadow-lg overflow-y-auto">
    <div
      :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }"
    >
      <div class="mb-4">
        <button
          class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4"
          @click="$emit('back')"
        >
          <ArrowLeft />
        </button>
      </div>

      <div class="flex justify-between">
        <div class="flex flex-col mb-8">
          <div class="text-thin text-xl text-brave-10">
            {{ album.name }}
          </div>
          <div class="flex items-center gap-2">
            <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ album.tracks_count }} tracks</div>
            <div class="border-r border-brave-80 h-3 flex-none"></div>
            <div class="text-sm text-brave-30 group-hover:text-brave-20 transition">{{ album.artist_name }}</div>
          </div>
        </div>

        <div class="">
          <button class="button button-normal px-4 py-1.5 text-xs rounded-full" @click.prevent="downloadAlbumLyrics">
          <div class="text-sm"><DownloadMultiple /></div>
          <span>
            Download album lyrics
          </span>
        </button>
        </div>
      </div>

      <div class="w-full">
        <div class="w-full flex">
          <div class="text-xs text-brave-30/70 font-bold flex w-full">
            <div class="text-left flex-none w-[65%] p-1">Track</div> <!-- Adjusted width percentage -->
            <div class="text-right flex-none w-[10%] p-1">Duration</div>
            <div class="text-center flex-none w-[10%] p-1">Lyrics</div>
            <div class="text-right flex-none w-[15%] p-1"></div>
          </div>
        </div>
        <div class="w-full flex flex-col">
          <div
            v-for="virtualRow in virtualRows"
            :key="virtualRow.index"
            class="group flex flex-col w-full"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
              }"
          >
            <TrackItem
              :trackId="virtualRow.key"
              @play-track="playTrack"
              @download-lyrics="downloadLyrics"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ArrowLeft, DownloadMultiple } from 'mdue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import TrackItem from '../track-list/TrackItem.vue'
import { useDownloader } from '@/composables/downloader.js'

const props = defineProps(['album'])
const emit = defineEmits(['back', 'playTrack', 'downloadLyrics'])

const { addToQueue } = useDownloader()

const trackIds = ref([])
const parentRef = ref(null)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: trackIds.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 52,
    overscan: 5,
    paddingStart: 175,
    getItemKey: (index) => trackIds.value[index]
  }))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const playTrack = (track) => {
  emit('playTrack', track)
}

const downloadLyrics = (track) => {
  emit('downloadLyrics', track)
}

const downloadAlbumLyrics = async () => {
  addToQueue(trackIds.value)
}

onMounted(async () => {
  trackIds.value = await invoke('get_album_track_ids', { albumId: props.album.id })
})
</script>
