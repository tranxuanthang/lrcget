<template>
  <div ref="parentRef" class="p-4 bg-brave-99 overflow-y-auto h-full" v-show="props.isActive">
    <div
      :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }"
    >
      <div class="w-full">
        <div class="w-full flex">
          <div class="text-xs text-brave-30/70 font-bold flex w-full">
            <div class="text-left flex-none w-[65%] p-1">Artist</div>
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
            <ArtistItem
              :artistId="virtualRow.key"
              @open-artist="openArtist"
            />
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
import { DownloadMultiple } from 'mdue'
import { ref, computed, onMounted, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
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
    estimateSize: () => 48,
    overscan: 5,
    paddingStart: 32,
    getItemKey: (index) => artistIds.value[index]
  }))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const openArtist = async (artist) => {
  currentArtist.value = artist
}

onMounted(async () => {
  if (props.isActive) {
    artistIds.value = await invoke('get_artist_ids')
    console.log(artistIds.value)
  }
})

watch(() => props.isActive, async () => {
  if (props.isActive) {
    artistIds.value = await invoke('get_artist_ids')
  }
})
</script>
