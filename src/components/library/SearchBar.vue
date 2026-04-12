<template>
  <div class="flex gap-2 items-center flex-row justify-center my-1 w-full px-2">
    <div class="flex flex-row items-center gap-2">
      <label for="autosearch" class="group-label">Auto</label>
      <input
        id="autosearch"
        v-model="autoSearch"
        type="checkbox"
        class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block min-w-fit p-1.5"
      />
    </div>
    <input
      id="searchInput"
      v-model="searchInput"
      type="text"
      class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block w-full max-w-2xl p-1.5"
      placeholder="Search"
      @keyup.enter="makeSearch"
      @input="handleAutoSearch"
    />
    <button
      class="button button-normal px-4 py-1.5 rounded-full h-full"
      :disabled="cooldown"
      :class="{
        'cursor-not-allowed': cooldown,
        // TODO: Make button greyed out when cooldown is active
      }"
      @click="makeSearch"
    >
      <Magnify />
    </button>
    <button class="button button-normal px-4 py-1.5 rounded-full h-full" @click="clearSearch">
      <Close />
    </button>
    <div v-show="cooldown || preClear" class="animate-spin text-sm">
      <Loading />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import Magnify from '~icons/mdi/magnify'
import Close from '~icons/mdi/close'
import Loading from '~icons/mdi/loading'
import { useSearchLibrary } from '@/composables/search-library.js'

const DEFAULT_WAIT = 500
const DEFAULT_COOLDOWN = DEFAULT_WAIT * 0.75

const autoSearch = ref(false)
const searchInput = ref('')
const cooldown = ref(false)
let preClear = null

const handleAutoSearch = () => {
  if (autoSearch.value) {
    if (preClear) {
      clearTimeout(preClear)
    }
    preClear = setTimeout(() => {
      makeSearch()
      preClear = null
    }, 1000)
  }
}
const makeSearch = () => {
  if (cooldown.value) {
    return
  }
  cooldown.value = true
  useSearchLibrary().setSearch(searchInput.value)
  setTimeout(() => {
    cooldown.value = false
  }, 750)
}
const clearSearch = () => {
  searchInput.value = ''
  makeSearch()
}
</script>
