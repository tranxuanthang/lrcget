<template>
  <BaseModal
    title="Existing Lyrics Found"
    content-class="w-full max-w-md"
    :close-button="true"
    @close="emit('close')"
  >
    <div>Lyrics for this track were previously downloaded.</div>

    <template #footer>
      <div class="w-full flex justify-end gap-2">
        <button
          class="button px-4 h-8 rounded-full text-sm"
          :class="{
            'button-normal': !isLoading,
            'button-disabled': isLoading,
          }"
          :disabled="isLoading"
          @click="redownload"
        >
          <Loading v-if="isLoading" class="animate-spin mr-1" />
          <Refresh v-else class="mr-1" />
          Redownload
        </button>
        <button
          class="button button-primary px-4 h-8 rounded-full text-sm"
          @click="continueEditing"
        >
          <Pencil class="mr-1" />
          Continue Editing
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'vue-toastification'
import BaseModal from '@/components/common/BaseModal.vue'
import Loading from '~icons/mdi/loading'
import Refresh from '~icons/mdi/refresh'
import Pencil from '~icons/mdi/pencil'

const props = defineProps({
  lrclibId: {
    type: Number,
    required: true,
  },
  existingResult: {
    type: Object,
    required: true,
  },
})

const emit = defineEmits(['close', 'redownload', 'continue'])

const toast = useToast()
const isLoading = ref(false)

const redownload = async () => {
  isLoading.value = true
  try {
    const result = await invoke('refresh_lrclib_lyricsfile', { lrclibId: props.lrclibId })
    emit('redownload', result)
    emit('close')
  } catch (error) {
    console.error('Error refreshing lyrics:', error)
    toast.error('Failed to redownload lyrics from LRCLIB')
  } finally {
    isLoading.value = false
  }
}

const continueEditing = () => {
  emit('continue', props.existingResult)
  emit('close')
}
</script>
