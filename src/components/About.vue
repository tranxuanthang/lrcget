<template>
  <BaseModal title="About" @opened="openedHandler" @close="emit('close')">
    <div class="overflow-auto">
      <div class="mb-6">
        <label class="group-label mb-2">Update</label>

        <div class="text-sm">
          <div class="mb-1">Current version: <span class="font-bold">{{ version }}</span></div>
          <template v-if="!isCheckingForUpdate && newestVersion">
            <div class="mb-1">Newest version: <span class="font-bold">{{ newestVersion }}</span></div>
            <div v-if="isUpdateAvailable" class="font-bold text-yellow-600 flex flex-wrap gap-1 items-center">
              <Alert class="text-lg" />
              New update is available. See what's new and download at
              <span><a :href="newestReleaseUrl" class="link" target="_blank" @click="downloadUpdate">LRCGET GitHub
                  Release Page</a>!</span>
            </div>
            <div v-else class="font-bold text-green-600 flex flex-wrap gap-1 items-center">
              <CheckCircle class="text-lg" />
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
      </div>

      <div class="mb-6">
        <label class="group-label mb-2">Donation</label>

        <div class="text-sm">
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
            <CopyablePre>43ZN5qDdGQhPGthFnngD8rjCHYLsEFBcyJjDC1GPZzVxWSfT8R48QCLNGyy6Z9LvatF5j8kSgv23DgJpixJg8bnmMnKm3b7
            </CopyablePre>
          </div>

          <div class="mb-2">
            <div class="mb-1">Litecoin (LTC):</div>
            <CopyablePre>ltc1q7texq5qsp59gclqlwf6asrqmhm98gruvz94a48</CopyablePre>
          </div>
        </div>
      </div>

      <div>
        <label class="group-label mb-2">Home page, contact and support</label>

        <div class="text-sm">
          <div class="mb-1">
            Github URL: <a href="https://github.com/tranxuanthang/lrcget" class="link" target="_blank"
              @click="downloadUpdate">github.com/tranxuanthang/lrcget</a>
          </div>

          <div>
            Email: <a href="mailto:hoangtudevops@protonmail.com" class="link" target="_blank"
              @click="downloadUpdate">hoangtudevops@protonmail.com</a>
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { Alert, CheckCircle } from 'mdue'
import { ref, computed } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import semver from 'semver'
import CopyablePre from '@/components/CopyablePre.vue'
import BaseModal from '@/components/common/BaseModal.vue'

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

const openedHandler = async () => {
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
</script>
