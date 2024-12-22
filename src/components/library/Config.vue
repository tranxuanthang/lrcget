<template>
  <BaseModal
    title="Configuration"
    @beforeOpen="beforeOpenHandler"
    @close="emit('close')"
    body-class="flex flex-col h-full justify-between"
  >
    <div class="flex flex-col gap-8">
      <div>
        <label class="group-label mb-2">Common</label>

        <div class="flex items-center mb-2">
          <input id="skip-not-needed-tracks" type="checkbox" v-model="skipNotNeededTracks" class="checkbox">
          <label for="skip-not-needed-tracks" class="checkbox-label ml-2">Skip tracks that already have lyrics or are instrumental</label>
        </div>

        <label class="block mb-1 child-label">Theme mode</label>

        <div class="flex gap-2 button-group">
          <button
            @click="editingThemeMode = 'auto'"
            class="button grouped-button"
            :class="{ 'button-primary': editingThemeMode === 'auto', 'button-normal': editingThemeMode !== 'auto' }"
          >
            Auto
          </button>
          <button
            @click="editingThemeMode = 'light'"
            class="button grouped-button"
            :class="{ 'button-primary': editingThemeMode === 'light', 'button-normal': editingThemeMode !== 'light' }"
          >
            Light
          </button>
          <button
            @click="editingThemeMode = 'dark'"
            class="button grouped-button"
            :class="{ 'button-primary': editingThemeMode === 'dark', 'button-normal': editingThemeMode !== 'dark' }"
          >
            Dark
          </button>
        </div>
      </div>

      <div>
        <label class="group-label mb-2">Experimental</label>

        <div class="flex items-start">
          <input id="try-embed-lyrics" type="checkbox" v-model="tryEmbedLyrics" class="checkbox mt-0.5">
          <div class="flex flex-col ml-2">
            <label for="try-embed-lyrics" class="checkbox-label mb-0.5">Try to embed the lyrics to the track files when possible</label>
            <div class="text-xs text-yellow-700">This option could corrupt your track files. Make sure to backup your library before enabling it.</div>
          </div>
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <a href="#" class="link" @click="refreshLibrary">Refresh my library for new changes...</a>
      <a href="#" class="link" @click="uninitializeLibrary">Add and remove scanning directories...</a>
    </div>

    <template #footer>
      <button class="button button-primary px-8 py-2 rounded-full" @click="save">Save</button>
    </template>
  </BaseModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref } from 'vue'
import { useGlobalState } from '../../composables/global-state'

const { setThemeMode } = useGlobalState()

const emit = defineEmits(['close', 'refreshLibrary', 'uninitializeLibrary'])

const skipNotNeededTracks = ref(true)
const tryEmbedLyrics = ref(false)
const editingThemeMode = ref('auto')

const save = async () => {
  await invoke('set_config', {
    skipNotNeededTracks: skipNotNeededTracks.value,
    tryEmbedLyrics: tryEmbedLyrics.value,
    themeMode: editingThemeMode.value
  })
  setThemeMode(editingThemeMode.value)
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
  editingThemeMode.value = config.theme_mode
}
</script>

<style scoped>
.button-group {
  @apply flex gap-0.5 items-center;
}

.grouped-button {
  @apply first:rounded-l-full last:rounded-r-full  text-sm px-4 py-1 w-24;
}
</style>
