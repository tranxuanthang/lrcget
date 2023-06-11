<template>
  <div class="h-screen w-screen flex flex-col select-none border border-brave-90">
    <div data-tauri-drag-region class="flex justify-end items-start text-sm flex-none z-40">
      <div class="py-2 px-4 text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition" @click="openDevtools">
        <Bug />
      </div>
      <div class="py-2 px-4 text-brave-35 hover:bg-brave-90 active:bg-brave-80 transition" @click="minimizeWindow">
        <WindowMinimize />
      </div>
      <div class="py-2 px-4 text-brave-35 hover:bg-brave-90 active:bg-brave-80 transition"  @click="maximizeWindow">
        <WindowMaximize />
      </div>
      <div class="py-2 px-4 text-brave-35 hover:bg-brave-35 active:bg-brave-30 hover:text-white active:text-white transition"  @click="closeWindow">
        <WindowClose />
      </div>
    </div>
    <div v-if="!loading" class="grow overflow-hidden">
      <ChooseDirectory v-if="!init" @progressStep="init = true" />
      <Library v-else @uninitialize-library="uninitializeLibrary" />
    </div>
  </div>
</template>

<script setup>
import { Bug, WindowMinimize, WindowMaximize, WindowClose } from 'mdue'
import ChooseDirectory from "./components/ChooseDirectory.vue";
import Library from "./components/Library.vue";
import { ref, onMounted } from 'vue'
import { appWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/tauri'

const loading = ref(true)
const init = ref(false)
const isProd = ref(import.meta.env.PROD)

const uninitializeLibrary = async () => {
  loading.value = true
  await invoke('uninitialize_library')
  init.value = await invoke('get_init')
  loading.value = false
}

onMounted(async () => {
  init.value = await invoke('get_init')
  loading.value = false
})

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
