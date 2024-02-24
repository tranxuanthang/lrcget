<template>
  <div ref="parentRef" class="p-4 bg-brave-99 overflow-y-auto grow" v-show="props.isActive">
    <div
      :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }"
    >
      <div class="w-full">
        <div class="w-full flex">
          <div class="text-xs text-brave-30/70 font-bold flex w-full">
            <div class="text-left flex-none w-[65%] p-1">Album</div>
            <div class="text-right flex-none w-[15%] p-1"></div>
          </div>
        </div>
        <div class="w-full flex flex-col">
          <div
            v-for="virtualRow in virtualRows"
            :key="virtualRow.index"
            class="group flex flex-col w-full absolute top-0 left-0 w-full"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
              }"
          >
            <AlbumItem
              :albumId="virtualRow.key"
              @open-album="openAlbum"
              @download-lyrics="downloadLyricsMultiple"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { DownloadMultiple } from 'mdue'
import { ref, computed, onMounted, watch } from 'vue'
import AlbumItem from './album-list/AlbumItem.vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

const props = defineProps(['isActive'])
const emit = defineEmits(['downloadLyricsMultiple'])

const albumIds = ref([])
const parentRef = ref(null)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: albumIds.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 48,
    overscan: 5,
    paddingStart: 32,
    getItemKey: (index) => albumIds.value[index]
  }))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const openAlbum = async (album) => {
  // TODO
}

const downloadLyricsMultiple = async (album) => {
  const tracks = await invoke('get_album_tracks', { albumId: album.id })
  emit('downloadLyricsMultiple', tracks)
}

onMounted(async () => {
  if (props.isActive) {
    albumIds.value = await invoke('get_album_ids')
    console.log(albumIds.value)
  }
})

watch(() => props.isActive, async () => {
  if (props.isActive) {
    albumIds.value = await invoke('get_album_ids')
  }
})
</script>
