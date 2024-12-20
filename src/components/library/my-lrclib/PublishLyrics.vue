<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="modal-content max-w-screen-sm max-h-[60vh] flex flex-col gap-4 p-6"
    overlay-class="modal-overlay"
    overlay-transition="fade"
    content-transition="pop-fade"
    :click-to-close="!isPublishing"
    :esc-to-close="!isPublishing"
  >
    <template v-if="lintResult.length">
      <div class="grow flex flex-col h-full overflow-hidden">
        <div class="mb-4">Please fix the following problem(s) before publishing</div>

        <div class="grow overflow-y-scroll h-full">
          <table class="lint-result table">
            <thead class="text-xs font-bold">
              <tr>
                <th class="p-1 text-right">Line</th>
                <th class="p-1 text-center">Severity</th>
                <th class="p-1">Message</th>
              </tr>
            </thead>
            <tbody class="text-xs">
              <tr v-for="(problem, index) in lintResult" :key="index">
                <td class="p-1 text-right">{{ problem.line }}</td>
                <td class="p-1 text-center">
                  <span v-if="problem.severity === 'error'" class="bg-red-200 text-red-800 font-bold text-xs px-1 py-0.5 rounded">Error</span>
                </td>
                <td class="p-1">{{ problem.message }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="flex gap-2 justify-center w-full">
        <button class="button button-primary px-8 py-2 rounded-full" @click="close">Close</button>
      </div>
    </template>

    <template v-else>
      <div class="flex flex-col items-center">
        <div v-if="!isPublishing" class="mb-4">
          Do you want to publish your lyrics of the song <strong>{{ track.name }} - {{ track.artistName }}</strong> to your current LRCLIB instance?
        </div>
        <div v-else class="mb-4">
          Publishing your lyrics of the song <strong>{{ track.name }} - {{ track.artistName }}</strong>...
        </div>

        <table v-if="isPublishing" class="text-xs table-auto font-mono uppercase">
          <tbody>
            <tr>
              <td class="px-2 py-1">Request challenge...</td>
              <td class="text-right px-2 py-1">{{ progress.requestChallenge }}</td>
            </tr>

            <tr>
              <td class="px-2 py-1">Solve challenge...</td>
              <td class="text-right px-2 py-1">{{ progress.solveChallenge }}</td>
            </tr>

            <tr>
              <td class="px-2 py-1">Publish lyrics...</td>
              <td class="text-right px-2 py-1">{{ progress.publishLyrics }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="!isPublishing" class="flex gap-2 justify-center w-full">
        <button class="button button-primary px-8 py-2 rounded-full" @click="publishLyrics">Publish Now</button>
        <button class="button button-normal px-8 py-2 rounded-full" @click="close">Cancel</button>
      </div>

      <div v-else class="flex gap-2 justify-center w-full">
        <button class="button button-disabled px-8 py-2 rounded-full flex gap-3" disabled><div class="animate-spin"><Loading /></div><div>Publishing</div></button>
      </div>
    </template>
  </VueFinalModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted } from 'vue'
import { Loading } from 'mdue'
import { listen } from '@tauri-apps/api/event'
import { useToast } from 'vue-toastification'

const toast = useToast()
const emit = defineEmits(['close'])
const props = defineProps(['lintResult', 'track', 'lyrics'])

const isPublishing = ref(false)
const isError = ref(false)
const progress = ref({
  requestChallenge: 'Pending',
  solveChallenge: 'Pending',
  publishLyrics: 'Pending'
})

const publishLyrics = async () => {
  isPublishing.value = true
  const plainLyrics = props.lyrics.replace(/^\[(.*)\] */mg, '')
  const syncedLyrics = props.lyrics
  try {
    await invoke('publish_lyrics', {
      title: props.track.name,
      albumName: props.track.albumName,
      artistName: props.track.artistName,
      duration: props.track.duration,
      plainLyrics,
      syncedLyrics
    })
    toast.success('Your lyrics has been published successfully! It might take up to 24 hours to be visible on the search results.')
  } catch (error) {
    isError.value = true
    console.error(error)
    toast.error(error)
  } finally {
    isPublishing.value = false
    close()
  }
}

onMounted(() => {
  console.log('lintResult', props.lintResult)
  console.log('track', props.track)
  listen('publish-lyrics-progress', (event) => {
    progress.value = event.payload
  })
})

const close = () => {
  if (!isPublishing.value) {
    emit('close')
  }
}
</script>
