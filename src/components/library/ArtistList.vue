<template>
  <div v-show="props.isActive" ref="parentRef" class="p-4 overflow-y-auto h-full">
    <div :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }">
      <div class="w-full">
        <div class="w-full flex">
          <div class="text-xs text-brave-30/70 font-bold flex w-full dark:text-brave-95">
            <div class="text-left flex-none w-[65%] p-1">Artist</div>
            <div class="text-right flex-none w-[15%] p-1" />
          </div>
        </div>
        <div class="w-full flex flex-col">
          <div
            v-for="virtualRow in virtualRows"
            :key="virtualRow.index"
            class="group flex flex-col"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
            }"
          >
            <ArtistItem :artist-id="virtualRow.key" @open-artist="openArtist" />
          </div>
        </div>
      </div>
    </div>

    <Transition name="slide-fade">
      <ArtistTrackList v-if="currentArtist" :artist="currentArtist" @back="currentArtist = null" />
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { invoke } from '@tauri-apps/api/core'
import ArtistItem from './artist-list/ArtistItem.vue'
import ArtistTrackList from './artist-list/ArtistTrackList.vue'

const props = defineProps(['isActive'])

const artistIds = ref([])
const parentRef = ref(null)
const currentArtist = ref(null)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: artistIds.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 52,
    overscan: 5,
    paddingStart: 32,
    getItemKey: index => artistIds.value[index],
  }))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const openArtist = async artist => {
  currentArtist.value = artist
}

onMounted(async () => {
  if (props.isActive) {
    artistIds.value = await invoke('get_artist_ids')
  }
})

watch(
  () => props.isActive,
  async () => {
    if (props.isActive) {
      artistIds.value = await invoke('get_artist_ids')
    }
  }
)
</script>
