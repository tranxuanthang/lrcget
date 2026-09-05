<template>
  <div
    class="flex w-full group hover:bg-neutral-50 hover:shadow hover:shadow-neutral-100/50 border border-transparent hover:border-neutral-100 transition rounded cursor-default dark:hover:bg-neutral-900 dark:hover:border-neutral-700 dark:hover:shadow-black/50"
  >
    <div v-if="artist" class="p-1 flex flex-col grow" @click="$emit('openArtist', artist)">
        <div class="font-bold text-sm text-neutral-800 dark:text-neutral-200">
          {{ artist.name }}
        </div>

        <div class="flex items-center gap-2">
          <div
            class="text-sm text-neutral-500 group-hover:text-neutral-500 transition dark:text-neutral-400 dark:group-hover:text-neutral-400"
          >
            {{ artist.tracks_count }} tracks
          </div>
        </div>
    </div>

    <div class="flex items-center gap-2 p-1">
      <div v-if="artist" class="transition gap-1">
        <button
          class="text-neutral-800 hover:bg-hoa-1100 hover:text-white rounded p-2 transition dark:text-white dark:hover:bg-hoa-1100 dark:hover:text-white"
          @click.stop.prevent="downloadLyricsMultiple"
          :disabled="!artist || artist.id !== props.artistId"
          :aria-label="`Download lyrics for ${artist?.name ?? ''}`"
        >
          <DownloadMultiple />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import DownloadMultiple from '~icons/mdi/download-multiple'
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDownloadTrigger } from '@/composables/download-options.js'

const props = defineProps(['artistId'])
defineEmits(['openArtist'])

const artist = ref(null)

const downloadLyricsMultiple = useDownloadTrigger(() =>
  artist.value?.id === props.artistId
    ? { type: 'artist', id: artist.value.id, name: artist.value.name }
    : null
)

const loadArtist = async id => {
  artist.value = null
  const loaded = await invoke('get_artist', { artistId: id })
  if (id === props.artistId) artist.value = loaded
}

watch(
  () => props.artistId,
  async newId => {
    if (newId) {
      await loadArtist(newId)
    }
  }
)

onMounted(async () => {
  await loadArtist(props.artistId)
})
</script>
