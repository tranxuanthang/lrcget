<template>
  <div class="h-screen w-screen flex flex-col select-none">
    <div class="fixed top-0 left-0 flex justify-end items-start text-sm flex-none z-50">
      <div
        class="p-0.5 m-1 rounded-full text-sm text-hoa-1400 hover:bg-hoa-600 active:bg-hoa-800 transition"
        @click="openDevtools"
      >
        <Bug />
      </div>
    </div>
    <div v-if="!loading" class="grow overflow-hidden bg-white dark:bg-brave-background-dark">
      <ChooseDirectory
        v-if="!init"
        @progress-step="onProgressStep"
        @directories-changed="onDirectoriesChanged"
      />
      <Library
        v-else
        :should-scan="shouldScan"
        @uninitialize-library="uninitializeLibrary"
        @manage-directories="manageDirectories"
        @scan-complete="onScanComplete"
      />
    </div>
  </div>

  <ModalsContainer />
</template>

<script setup>
import Bug from '~icons/mdi/bug'
import WindowMinimize from '~icons/mdi/window-minimize'
import WindowMaximize from '~icons/mdi/window-maximize'
import WindowClose from '~icons/mdi/window-close'
import ChooseDirectory from './components/ChooseDirectory.vue'
import Library from './components/Library.vue'
import { ref, onMounted, watch } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import { ModalsContainer } from 'vue-final-modal'
import { useGlobalState } from './composables/global-state'
import { useDownloader } from '@/composables/downloader.js'
import { usePlayer } from '@/composables/player.js'
import { useToast } from 'vue-toastification'

const appWindow = getCurrentWebviewWindow()
const toast = useToast()
const { themeMode, setThemeMode, setLrclibInstance } = useGlobalState()
const { downloadNext } = useDownloader()
const { setVolume } = usePlayer()

const loading = ref(true)
const init = ref(false)
const shouldScan = ref(false)
const isProd = ref(import.meta.env.PROD)

const uninitializeLibrary = async () => {
  loading.value = true
  await invoke('uninitialize_library')
  init.value = await invoke('get_init')
  loading.value = false
}

const manageDirectories = () => {
  // Just show the directory chooser without clearing the database
  // The incremental scan will handle any directory changes
  init.value = false
}

const onProgressStep = () => {
  init.value = true
}

const onDirectoriesChanged = () => {
  // Directories were changed, trigger a scan in Library
  shouldScan.value = true
}

const onScanComplete = () => {
  // Reset the scan flag after completion
  shouldScan.value = false
}

onMounted(async () => {
  init.value = await invoke('get_init')
  loading.value = false
  await loadGlobalState()
  darkModeHandle(themeMode.value)
  downloadNext()
  drainNotifications()
})

const loadGlobalState = async () => {
  const config = await invoke('get_config')
  setThemeMode(config.theme_mode)
  setLrclibInstance(config.lrclib_instance)
  // Set initial volume from config (default to 1.0 if not set)
  const volume = config.volume !== undefined ? config.volume : 1.0
  setVolume(volume)
}

const drainNotifications = async () => {
  setInterval(async () => {
    const notifications = await invoke('drain_notifications')
    notifications.forEach(notification => {
      toast(notification.message, {
        type: notification.type,
      })
    })
  }, 100)
}

const darkModeHandle = async themeMode => {
  if (themeMode !== 'dark' && themeMode !== 'light') {
    themeMode = await appWindow.theme()
  }
  // check and insert the `dark` class to html tag
  if (themeMode === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

const openDevtools = () => {
  invoke('open_devtools')
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

appWindow.onThemeChanged(({ payload: theme }) => {
  if (themeMode.value === 'auto') {
    darkModeHandle(theme)
  }
})

watch(themeMode, () => {
  darkModeHandle(themeMode.value)
})
</script>

<style></style>
