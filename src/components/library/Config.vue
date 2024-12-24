<template>
  <BaseModal
    title="Configuration"
    @beforeOpen="beforeOpenHandler"
    @close="emit('close')"
    body-class="flex flex-col h-full justify-between"
  >
    <div class="flex flex-col gap-8">
      <div>
        <label class="group-label mb-4">Common</label>

        <div class="flex items-center mb-4">
          <CheckboxButton
            v-model="skipNotNeededTracks"
            name="skip-not-needed-tracks"
            id="skip-not-needed-tracks"
          >
            Skip tracks that already have lyrics or are instrumental
          </CheckboxButton>
        </div>

        <div class="flex flex-col mb-4">
          <label class="block mb-1 child-label">Theme mode</label>

          <div class="flex gap-4">
            <RadioButton
              v-model="editingThemeMode"
              name="theme-mode"
              id="theme-auto"
              value="auto"
            >
              Auto
            </RadioButton>

            <RadioButton
              v-model="editingThemeMode"
              name="theme-mode"
              id="theme-light"
              value="light"
            >
              Light
            </RadioButton>

            <RadioButton
              v-model="editingThemeMode"
              name="theme-mode"
              id="theme-dark"
              value="dark"
            >
              Dark
            </RadioButton>
          </div>
        </div>

        <div class="flex flex-col">
          <label class="block mb-1 child-label" for="lrclib-instance">LRCLIB instance</label>
          <input id="lrclib-instance" type="text" v-model="editingLrclibInstance" placeholder="https://" class="input px-4 h-8">
        </div>
      </div>

      <div>
        <label class="group-label mb-4">Experimental</label>

        <div class="flex items-start">
          <CheckboxButton
            v-model="tryEmbedLyrics"
            name="try-embed-lyrics"
            id="try-embed-lyrics"
          >
            <div class="flex flex-col">
              <span class="mb-0.5">Try to embed the lyrics to the track files when possible</span>
              <span class="text-xs text-yellow-700">This option could corrupt your track files. Make sure to backup your library before enabling it.</span>
            </div>
          </CheckboxButton>
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
import RadioButton from '@/components/common/RadioButton.vue'
import CheckboxButton from '@/components/common/CheckboxButton.vue'

const { setThemeMode } = useGlobalState()

const emit = defineEmits(['close', 'refreshLibrary', 'uninitializeLibrary'])

const skipNotNeededTracks = ref(true)
const tryEmbedLyrics = ref(false)
const editingThemeMode = ref('auto')
const editingLrclibInstance = ref('')

const save = async () => {
  await invoke('set_config', {
    skipNotNeededTracks: skipNotNeededTracks.value,
    tryEmbedLyrics: tryEmbedLyrics.value,
    themeMode: editingThemeMode.value,
    lrclibInstance: editingLrclibInstance.value
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
  editingLrclibInstance.value = config.lrclib_instance
}
</script>
