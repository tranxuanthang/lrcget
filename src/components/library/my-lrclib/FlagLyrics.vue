<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="modal-content max-w-[500px] max-h-[60vh] flex flex-col gap-4 p-6"
    overlay-transition="fade"
    content-transition="pop-fade"
    :click-to-close="!isFlagging"
    :esc-to-close="!isFlagging"
  >
    <div class="flex flex-col items-center">
      <div v-if="!isFlagging">
        <div class="text-brave-10 mb-4">Do you want to flag the lyrics of the song <strong>{{ track.name }} - {{ track.artistName }}</strong>?</div>

        <label for="flagReason" class="text-brave-10 mb-2 text-xs font-bold">Please explain why you want to flag the lyrics:</label>
        <textarea id="flagReason" v-model="flagReason" class="w-full p-2 rounded textarea" placeholder="Explain why you want to flag the lyrics..." />
      </div>
      <div v-else class="text-brave-10 mb-4">Flagging the lyrics of the song <strong>{{ track.name }} - {{ track.artistName }}</strong>...</div>

      <table v-if="isFlagging" class="text-xs table-auto text-brave-20 font-mono uppercase">
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
            <td class="px-2 py-1">Flag lyrics...</td>
            <td class="text-right px-2 py-1">{{ progress.flagLyrics }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="!isFlagging" class="flex gap-2 justify-center w-full">
      <button class="button button-primary px-8 py-2 rounded-full" @click="flagLyrics">Confirm Flag</button>
      <button class="button button-normal px-8 py-2 rounded-full" @click="$emit('close')">Cancel</button>
    </div>

    <div v-else class="flex gap-2 justify-center w-full">
      <button class="button button-disabled px-8 py-2 rounded-full flex gap-3" disabled><div class="animate-spin"><Loading /></div><div>Flagging</div></button>
    </div>
  </VueFinalModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted, watch } from 'vue'
import { Loading } from 'mdue'
import { listen } from '@tauri-apps/api/event'
import { useToast } from 'vue-toastification'

const toast = useToast()
const emit = defineEmits(['close'])
const props = defineProps(['track'])

const isFlagging = ref(false)
const isError = ref(false)
const flagReason = ref('')

const progress = ref({
  requestChallenge: 'Pending',
  solveChallenge: 'Pending',
  flagLyrics: 'Pending'
})

const flagLyrics = async () => {
  isFlagging.value = true

  try {
    await invoke('flag_lyrics', { trackId: props.track.id, flagReason: flagReason.value })
    toast.success('The lyrics has been flagged successfully!')
  } catch (error) {
    isError.value = true
    console.error(error)
    toast.error(error)
  } finally {
    isFlagging.value = false
    emit('close')
  }
}

onMounted(() => {
  listen('flag-lyrics-progress', (event) => {
    progress.value = event.payload
  })
})
</script>
