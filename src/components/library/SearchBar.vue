<template>
  <div class="flex gap-2 items-center flex-row justify-center my-1 w-full px-2">
    <div class="flex flex-row items-center gap-2">
      <label for="autosearch" class="group-label">Auto</label>
      <input type="checkbox" id="autosearch" v-model="autoSearch"
        class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block min-w-fit p-1.5" />
    </div>
    <input type="text" id="searchInput" v-model="searchInput"
      class="bg-brave-95 border border-brave-95 focus:border-brave-90 text-gray-900 outline-none text-sm rounded transition block w-full max-w-2xl p-1.5"
      placeholder="Search" @keyup.enter="makeSearch" @input="handleAutoSearch" />
    <button class="button button-normal px-4 py-1.5 rounded-full h-full" @click="makeSearch">
      <Magnify />
    </button>
    <button class="button button-normal px-4 py-1.5 rounded-full h-full" @click="clearSearch">
      <Close />
    </button>
    <div class="animate-spin text-sm" v-show="cooldown || preClear">
      <Loading />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { Magnify, Close, Loading } from 'mdue';
import { useSearchLibrary } from '../../composables/search';
const autoSearch = ref(false);
const searchInput = ref('');
const cooldown = ref(false);
let preClear = null;

const handleAutoSearch = () => {
  if (autoSearch.value) {
    if (preClear) {
      clearTimeout(preClear);
    }
    preClear = setTimeout(() => {
      makeSearch();
      preClear = null;
    }, 500);
  }
};
const makeSearch = () => {
  cooldown.value = true;
  useSearchLibrary().setSearch(searchInput.value);
  setTimeout(() => {
    cooldown.value = false;
  }, 1000);
};
const clearSearch = () => {
  searchInput.value = '';
  makeSearch();
};
</script>
