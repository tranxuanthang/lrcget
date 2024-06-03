<template>
  <div class="h-screen w-screen flex flex-col select-none">
    <div class="fixed top-0 left-0 flex justify-end items-start text-sm flex-none z-50">
      <div class="p-0.5 m-1 rounded-full text-sm text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition"
        @click="openDevtools">
        <Bug />
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

<style></style>
