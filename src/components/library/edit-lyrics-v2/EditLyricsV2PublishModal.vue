<template>
  <BaseModal
    content-class="max-w-[560px] max-h-[60vh]"
    body-class="flex flex-col h-full min-h-0"
    :title="modalTitle"
    :click-to-close="!isPublishing"
    :esc-to-close="!isPublishing"
    :close-button="!isPublishing"
    @close="handleClose"
  >
    <div class="flex flex-col items-center">
      <div v-if="!isPublishing" class="mb-4 text-center">
        Publish lyrics for <strong>{{ track?.title }} - {{ track?.artist_name }}</strong> to your
        current LRCLIB instance?
      </div>
      <div v-else class="mb-4 text-center">
        Publishing lyrics for <strong>{{ track?.title }} - {{ track?.artist_name }}</strong
        >...
      </div>

      <table v-if="isPublishing" class="text-xs table-auto font-mono uppercase">
        <tbody>
          <tr>
            <td class="px-2 py-1">Request challenge...</td>
            <td class="text-right px-2 py-1">
              {{ progress.requestChallenge }}
            </td>
          </tr>

          <tr>
            <td class="px-2 py-1">Solve challenge...</td>
            <td class="text-right px-2 py-1">
              {{ progress.solveChallenge }}
            </td>
          </tr>

          <tr>
            <td class="px-2 py-1">Publish lyrics...</td>
            <td class="text-right px-2 py-1">
              {{ progress.publishLyrics }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <template #footer>
      <div v-if="!isPublishing" class="flex gap-2 justify-center w-full">
        <button class="button button-primary px-8 py-2 rounded-full" @click="publishLyrics">
          Publish Now
        </button>
        <button class="button button-normal px-8 py-2 rounded-full" @click="handleClose">
          Cancel
        </button>
      </div>

      <div v-else class="flex gap-2 justify-center w-full">
        <button class="button button-disabled px-8 py-2 rounded-full flex gap-3" disabled>
          <Loading class="animate-spin" />
          <span>Publishing</span>
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import Loading from '~icons/mdi/loading'
import { useToast } from 'vue-toastification'
import BaseModal from '@/components/common/BaseModal.vue'

const emit = defineEmits(['close'])
const props = defineProps({
  track: {
    type: Object,
    required: true,
  },
  lyricsfile: {
    type: String,
    required: true,
  },
})

const toast = useToast()
const isPublishing = ref(false)
const progress = ref({
  requestChallenge: 'Pending',
  solveChallenge: 'Pending',
  publishLyrics: 'Pending',
})
let unlistenPublishProgress = null

const modalTitle = computed(() => {
  if (!props.track) {
    return 'Publish lyrics'
  }

  return `${props.track.title} - ${props.track.artist_name}`
})

const publishLyrics = async () => {
  if (!props.track) {
    return
  }

  if (!props.lyricsfile?.trim()) {
    toast.error('Nothing to publish. Please add lyrics content first.')
    return
  }

  isPublishing.value = true

  try {
    await invoke('publish_lyrics', {
      title: props.track.title,
      albumName: props.track.album_name,
      artistName: props.track.artist_name,
      duration: props.track.duration,
      lyricsfile: props.lyricsfile,
    })

    toast.success(
      'Your lyrics has been published successfully! It might take up to 24 hours to be visible on the search results.'
    )
    emit('close')
  } catch (error) {
    console.error(error)
    toast.error(error)
  } finally {
    isPublishing.value = false
  }
}

const handleClose = () => {
  if (isPublishing.value) {
    return
  }

  emit('close')
}

onMounted(async () => {
  unlistenPublishProgress = await listen('publish-lyrics-progress', event => {
    progress.value = event.payload
  })
})

onUnmounted(() => {
  if (typeof unlistenPublishProgress === 'function') {
    unlistenPublishProgress()
  }
})
</script>
