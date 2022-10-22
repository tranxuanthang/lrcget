<template>
  <div v-if="!directoryForScan" class="flex justify-center items-center w-full h-full p-4">
    <button type="button" @click="chooseDirectory" class="p-3 w-48 h-48 bg-brave-primary rounded-full uppercase font-bold text-white hover:bg-brave-35 hover:scale-110 active:bg-brave-30 text-sm transition">
      Choose directory
    </button>
  </div>

  <div v-else class="w-full h-full flex flex-col">
    <div class="p-2 bg-brave-primary flex rounded items-stretch m-6">
      <div class="flex-grow text-sm font-bold px-2 py-2 text-white">{{ directoryForScan }}</div>
      <button type="button" @click="chooseDirectory" class="bg-brave-30 hover:bg-brave-25 active:bg-brave-20 transition text-white rounded py-2 px-4 text-xs font-bold uppercase h-full">Change directory</button>
    </div>

    <div v-if="loading" class="p-6 text-hoa-1500 rounded flex items-center justify-center h-full">
      <Loading class="animate-spin text-xl" />
    </div>
    <div v-else-if="musicItems && musicItems.length > 0" class="flex-grow flex flex-col items-center overflow-hidden w-full">
      <div class="mt-3 text-brave-5 text-sm font-bold uppercase">Found <strong>{{ musicItems.length }}</strong> tracks</div>

      <div class="flex-grow p-6 w-full">
        <OverlayScrollbars class="text-hoa-200 h-full overflow-y-auto rounded-lg">
          <table class="table-auto w-full">
            <thead class="">
              <tr class="bg-brave-90 text-brave-10 w-full">
                <th class="p-3">Track</th>
                <th class="p-3">Artist</th>
                <th class="p-3">Album</th>
                <th class="p-3">Duration</th>
                <th class="p-3">Lyrics</th>
              </tr>
            </thead>

            <tbody class="">
              <tr v-for="musicItem in musicItems" :key="musicItem.file_path" class="p-3 bg-white border-2 border-brave-98 last:border-none text-brave-10 text-sm">
                <td class="p-3">{{ musicItem.track_name }}</td>
                <td class="p-3">{{ musicItem.artist_name }}</td>
                <td class="p-3">{{ musicItem.album_name }}</td>
                <td v-if="musicItem.duration" class="p-3 text-center">{{ humanizeDuration(Math.round(musicItem.duration)) }}</td>
                <td v-else class="p-3"></td>
                <td v-if="musicItem.lyrics" class="p-3 text-center"><div class="flex items-center justify-center"><CheckCircle class="text-green-700" /></div></td>
                <td v-else class="p-3"></td>
              </tr>
            </tbody>
          </table>
        </OverlayScrollbars>
      </div>

      <div class="p-6 w-full flex justify-between bg-brave-90">
        <div class="flex items-center">
          <input id="skip-tracks" type="checkbox" v-model="skipTracksHaveExistingLyrics" class="w-4 h-4 text-hoa-600 bg-hoa-1100 accent-hoa-1100 rounded border-hoa-300">
          <label for="skip-tracks" class="ml-2 text-sm font-medium text-hoa-1500">Skip tracks that already have lyrics</label>
        </div>

        <button type="button" @click="progressStep" class="px-6 py-2 bg-brave-primary rounded-full uppercase font-bold text-white hover:bg-brave-35 active:bg-brave-30 transition">
          Confirm and download lyrics
        </button>
      </div>
    </div>

    <div v-else class="p-3 text-brave-10 flex items-center justify-center h-full">
      Could not find any music files
    </div>

  </div>
</template>

<script setup>
import { open } from "@tauri-apps/api/dialog"
import { audioDir } from '@tauri-apps/api/path'
import { ref, watch, defineEmits } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { CheckCircle, Loading } from 'mdue'
import path from 'path'

const emit = defineEmits(['progressStep'])

const directoryForScan = ref(null)
const musicItems = ref(null)
const loading = ref(false)
const skipTracksHaveExistingLyrics = ref(true)

const progressStep = () => {
  emit('progressStep', musicItems.value, true, false, skipTracksHaveExistingLyrics.value)
}

const humanizeDuration = (duration) => {
  return new Date(duration * 1000).toISOString().substring(14, 19)
}

const chooseDirectory = async () => {
  const dirPath = await audioDir();

  const selected = await open({
    defaultPath: dirPath,
    directory: true,
    recursive: true
  })

  if (selected) {
    directoryForScan.value = selected
  }
}

watch(directoryForScan, async (newDirectory) => {
  loading.value = true
  try {
    const result = await invoke("get_music_files", { directory: newDirectory })
    if (result && result.length > 0) {
      musicItems.value = result.sort((item1, item2) => {
        if (item1.track_name < item2.track_name) {
          return -1
        }
        if ( item1.track_name > item2.track_name) {
          return 1
        }
        return 0
      })
    } else {
      musicItems.value = []
    }
  } catch (error) {
    throw error
  } finally {
    loading.value = false
  }
})
</script>
