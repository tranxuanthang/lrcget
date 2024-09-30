<template>
  <div>
    <Transition name="pop-fade">
      <div v-if="isShow" class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-30" :class="{ 'hidden': !props.isShow }">
        <div class="w-full h-[80vh] max-w-screen-sm rounded-lg m-4 bg-white dark:bg-black flex flex-col">
          <div class="flex-none flex justify-between items-center px-6 py-2">
            <div class="text-thin text-xl text-brave-15 dark:text-brave-40">About</div>
            <button class="text-brave-20 dark:text-brave-35 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="close"><Close /></button>
          </div>

          <div class="px-6 grow gap-4 flex flex-col text-brave-20 dark:text-brave-35 py-6 overflow-hidden">
            <div class="overflow-auto">
              <div class="text-sm mb-6">
                <label class="group-label mb-2">Update</label>
                <div class="mb-1">Current version: <span class="font-bold">{{ version }}</span></div>
                <template v-if="!isCheckingForUpdate && newestVersion">
                  <div class="mb-1">Newest version: <span class="font-bold">{{ newestVersion }}</span></div>
                  <div v-if="isUpdateAvailable" class="font-bold text-yellow-600 flex flex-wrap gap-1 items-center">
                    <Alert class="text-lg"/>
                    New update is available. See what's new and download at
                    <span><a :href="newestReleaseUrl" class="link" target="_blank" @click="downloadUpdate">LRCGET GitHub Release Page</a>!</span>
                  </div>
                  <div v-else class="font-bold text-green-600 dark:text-green-500 flex flex-wrap gap-1 items-center">
                    <CheckCircle class="text-lg"/>
                    You are using newest version ;-)
                  </div>
                </template>
                <div v-else-if="!isCheckingForUpdate && !newestVersion">
                  Failed to check update from GitHub.
                </div>
                <div v-else>
                  Fetching newest version data from GitHub...
                </div>
              </div>

              <div class="text-sm mb-6">
                <label class="group-label mb-2">Donation</label>

                <div class="mb-2">
                  <div class="mb-1">GitHub Sponsors (Recommended):</div>
                  <div>
                    <a href="https://github.com/sponsors/tranxuanthang" class="link" target="_blank">
                      https://github.com/sponsors/tranxuanthang
                    </a>
                  </div>
                </div>

                <div class="mb-2">
                  <div class="mb-1">Buy Me a Coffee:</div>
                  <div>
                    <a href="https://www.buymeacoffee.com/thangtran" class="link" target="_blank">
                      https://www.buymeacoffee.com/thangtran
                    </a>
                  </div>
                </div>

                <div class="mb-2">
                  <div class="mb-1">Paypal:</div>
                  <div>
                    <a href="https://paypal.me/tranxuanthang98" class="link" target="_blank">
                      https://paypal.me/tranxuanthang98
                    </a>
                  </div>
                </div>

                <div class="mb-2">
                  <div class="mb-1">Monero (XMR):</div>
                  <CopyablePre>43ZN5qDdGQhPGthFnngD8rjCHYLsEFBcyJjDC1GPZzVxWSfT8R48QCLNGyy6Z9LvatF5j8kSgv23DgJpixJg8bnmMnKm3b7</CopyablePre>
                </div>

                <div class="mb-2">
                  <div class="mb-1">Litecoin (LTC):</div>
                  <CopyablePre>ltc1q7texq5qsp59gclqlwf6asrqmhm98gruvz94a48</CopyablePre>
                </div>
              </div>

              <div class="text-sm">
                <label class="group-label mb-2">Home page, contact and support</label>

                <div class="mb-1">
                  Github URL: <a href="https://github.com/tranxuanthang/lrcget" class="link" target="_blank" @click="downloadUpdate">github.com/tranxuanthang/lrcget</a>
                </div>

                <div>
                  Email: <a href="mailto:hoangtudevops@protonmail.com" class="link" target="_blank" @click="downloadUpdate">hoangtudevops@protonmail.com</a>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="isShow" class="fixed top-0 left-0 h-full w-full z-20 bg-black/30"></div>
    </Transition>
  </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { Close, Alert, CheckCircle } from 'mdue'
import { ref, watch, computed } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import semver from 'semver'
import CopyablePre from '@/components/CopyablePre.vue'

const props = defineProps(['isShow'])
const emit = defineEmits(['close'])

const version = ref(null)
const newestVersion = ref(null)
const newestReleaseUrl = ref(null)
const isUpdateAvailable = computed(() => {
  if (!version.value || !newestVersion.value) {
    return false
  }
  return semver.gt(newestVersion.value, version.value)
})
const isCheckingForUpdate = ref(true)

const close = () => {
  emit('close')
}

async function getLatestReleaseInfo() {
  const repo = 'tranxuanthang/lrcget'
  const apiUrl = `https://api.github.com/repos/${repo}/releases/latest`

  const response = await fetch(apiUrl)
  if (!response.ok) {
    throw new Error(`Error: ${response.status}`)
  }
  const data = await response.json()
  return {
    version: data.tag_name,
    releaseUrl: data.html_url
  }
}

watch(() => props.isShow, async (to, from) => {
  if (to === true && from === false) {
    version.value = await getVersion()
    isCheckingForUpdate.value = true
    try {
      const latestReleaseInfo = await getLatestReleaseInfo()
      newestVersion.value = latestReleaseInfo.version
      newestReleaseUrl.value = latestReleaseInfo.releaseUrl
    } catch (error) {
      console.error(error)
    } finally {
      isCheckingForUpdate.value = false
    }
  }
})
</script>
