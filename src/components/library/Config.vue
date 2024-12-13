<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="modal-content w-full h-[80vh] max-w-screen-sm flex flex-col"
    @beforeOpen="beforeOpenHandler"
    overlay-transition="fade"
    content-transition="pop-fade"
  >
    <div class="modal-title-bar">
      <div class="modal-title">Configuration</div>
      <button class="modal-button" @click="emit('close')"><Close /></button>
    </div>

    <div class="px-6 grow flex flex-col justify-between gap-4">
      <div class="flex flex-col gap-6">
        <div>
          <label class="group-label mb-2">Common</label>

          <div class="flex items-center">
            <input id="skip-not-needed-tracks" type="checkbox" v-model="skipNotNeededTracks" class="checkbox">
            <label for="skip-not-needed-tracks" class="checkbox-label ml-2">Skip tracks that already have lyrics or are instrumental</label>
          </div>
        </div>

        <div>
          <label class="group-label mb-2">Experimental</label>

          <div class="flex items-center">
          <input id="try-embed-lyrics" type="checkbox" v-model="tryEmbedLyrics" class="checkbox">
          <label for="try-embed-lyrics" class="checkbox-label ml-2">Try to embed the lyrics to the track files when possible</label>
          </div>
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <a href="#" class="link" @click="refreshLibrary">Refresh my library for new changes...</a>
        <a href="#" class="link" @click="uninitializeLibrary">Add and remove scanning directories...</a>
      </div>
    </div>

    <div class="px-6 py-4 flex-none flex justify-center">
      <button class="button button-primary px-8 py-2 rounded-full" @click="save">Save</button>
    </div>
  </VueFinalModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri';
import { Close } from 'mdue'
import { ref, watch } from 'vue'
import { VueFinalModal } from 'vue-final-modal'

const emit = defineEmits(['close', 'refreshLibrary', 'uninitializeLibrary', 'chooseDirectory'])

const skipNotNeededTracks = ref(true)
const tryEmbedLyrics = ref(false)

const save = async () => {
  await invoke('set_config', {
    skipNotNeededTracks: skipNotNeededTracks.value,
    tryEmbedLyrics: tryEmbedLyrics.value
  })
  emit('close')
}

const refreshLibrary = () => {
  emit('refreshLibrary')
  emit('close')
}

const uninitializeLibrary = () => {
  emit('uninitializeLibrary')
  emit('close')
}

const beforeOpenHandler = async () => {
  const config = await invoke('get_config')
  skipNotNeededTracks.value = config.skip_not_needed_tracks
  tryEmbedLyrics.value = config.try_embed_lyrics
}
</script>
