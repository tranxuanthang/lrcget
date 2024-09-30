<template>
  <div class="w-[16rem] relative h-full">
    <input v-model="searchInput" type="text" class="h-full input px-[2rem] py-1.5 pr-1.5 w-[16rem]" placeholder="Search for tracks..." autofocus>
    <div class="absolute top-0 left-0 w-[2rem] h-full flex justify-center items-center pl-0.5">
      <Magnify class="text-brave-30 dark:text-brave-60" />
    </div>
    <button v-if="searchInput !== ''" @click="searchInput = ''" class="absolute top-0 right-0 w-[2rem] h-full flex justify-center items-center pr-0.5 group">
      <Close class="text-brave-30 group-hover:text-brave-20 dark:group-hover:text-brave-30" />
    </button>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useSearchLibrary } from '@/composables/search-library.js'
import { Magnify, Close } from 'mdue'
import _debounce from 'lodash/debounce'

const { searchValue } = useSearchLibrary()

const searchInput = ref('')

const debouncedSearch = _debounce(async () => {
  searchValue.value = searchInput.value
}, 200)

watch(searchInput, debouncedSearch)
</script>
