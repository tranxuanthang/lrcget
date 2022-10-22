<template>
  <div class="h-screen w-screen flex flex-col select-none rounded-lg">
    <div data-tauri-drag-region class="flex justify-end items-start h-10 text-sm">
      <div v-if="!isProd" class="py-2 px-4 text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition" @click="openDevtools">
        <Bug />
      </div>
      <div class="py-2 px-4 text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition" @click="minimizeWindow">
        <WindowMinimize />
      </div>
      <div class="py-2 px-4 text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition"  @click="maximizeWindow">
        <WindowMaximize />
      </div>
      <div class="py-2 px-4 text-hoa-1400 hover:text-hoa-200 hover:bg-hoa-1400 active:bg-hoa-1500 active:text-hoa-200 transition"  @click="closeWindow">
        <WindowClose />
      </div>
    </div>
    <div class="flex-grow overflow-hidden">
      <ChooseDirectory v-if="step === 1" @progressStep="toStep3" />
      <SelectStrategy v-else-if="step === 2" @progressStep="toStep3" />
      <ApplyLyrics :musicItems="musicItems" :isCreateLRC="isCreateLRC" :isEmbed="isEmbed" :skipTracksHaveExistingLyrics="skipTracksHaveExistingLyrics" @startOver="startOver" v-else-if="step === 3" />
    </div>
  </div>
</template>

<script setup>
import { Bug } from 'mdue'
import { WindowMinimize } from 'mdue'
import { WindowMaximize } from 'mdue'
import { WindowClose } from 'mdue'
import ChooseDirectory from "./components/ChooseDirectory.vue";
import SelectStrategy from "./components/SelectStrategy.vue";
import ApplyLyrics from "./components/ApplyLyrics.vue";
import { ref } from 'vue'
import { appWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/tauri'

const step = ref(1)
const musicItems = ref(null)
const isCreateLRC = ref(true)
const isEmbed = ref(false)
const skipTracksHaveExistingLyrics = ref(true)
const isProd = ref(import.meta.env.PROD)

const toStep2 = (musicItems2) => {
  console.log(musicItems2)
  musicItems.value = musicItems2
  step.value = 2
}

const toStep3 = (musicItems2, isCreateLRC2, isEmbed2, skipTracksHaveExistingLyrics2) => {
  musicItems.value = musicItems2
  isCreateLRC.value = isCreateLRC2
  isEmbed.value = isEmbed2
  skipTracksHaveExistingLyrics.value = skipTracksHaveExistingLyrics2
  step.value = 3
}

const startOver = () => {
  musicItems.value = null
  isCreateLRC.value = true
  isEmbed.value = false
  skipTracksHaveExistingLyrics.value = true
  step.value = 1
}

const openDevtools = () => {
  invoke("open_devtools");
}

const minimizeWindow = () => {
  appWindow.minimize()
}

const maximizeWindow = () => {
  appWindow.toggleMaximize()
}

const closeWindow = () => {
  appWindow.close()
}
</script>

<style>
</style>
