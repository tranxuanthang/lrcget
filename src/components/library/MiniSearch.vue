<template>
  <div class="w-[16rem] relative h-full">
    <input
      v-model="searchInput"
      type="text"
      class="h-full input px-[2rem] py-1.5 pr-1.5 w-[16rem] dark:text-brave-95"
      placeholder="Search for tracks..."
      autofocus
    >
    <div class="absolute top-0 left-0 w-[2rem] h-full flex justify-center items-center pl-0.5">
      <Magnify class="text-brave-30 dark:text-brave-95" />
    </div>
    <div class="absolute top-0 right-0 h-full flex items-center px-1 gap-1">
      <button
        v-if="searchInput !== ''"
        @click="searchInput = ''"
        class="w-[1.5rem] h-[1.5rem] flex justify-center items-center text-brave-30 group-hover:text-brave-20 dark:text-brave-95 dark:hover:text-brave-90 rounded-full"
      >
        <Close />
      </button>
      <VDropdown theme="lrcget-dropdown" placement="top-start">
        <button
          class="w-[1.5rem] h-[1.5rem] flex justify-center items-center text-brave-30 group-hover:text-brave-20 dark:text-brave-95 dark:hover:text-brave-90 rounded-full"
          :class="{ 'bg-brave-80 dark:bg-brave-40': isFilters }"
        >
          <Filter />
        </button>
        <template #popper>
          <div class="dropdown-container">
            <label class="dropdown-item">
              <CheckboxButton
                v-model="filters.syncedLyricsTracks"
                name="synced-lyrics"
                id="synced-lyrics"
              >
                <span class="dropdown-label">Synced Lyrics Tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="filters.plainLyricsTracks"
                name="plain-lyrics"
                id="plain-lyrics"
              >
                <span class="dropdown-label">Plain Lyrics Tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="filters.instrumentalTracks"
                name="instrumental"
                id="instrumental"
              >
                <span class="dropdown-label">Instrumental Tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                v-model="filters.noLyricsTracks"
                name="no-lyrics"
                id="no-lyrics"
              >
                <span class="dropdown-label">No Lyrics Tracks</span>
              </CheckboxButton>
            </label>
          </div>
        </template>
      </VDropdown>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useSearchLibrary } from '@/composables/search-library.js'
import { Magnify, Close, Filter } from 'mdue'
import _debounce from 'lodash/debounce'
import CheckboxButton from '@/components/common/CheckboxButton.vue'

const { setSearch, filters } = useSearchLibrary()

const searchInput = ref('')

const isFilters = computed(() => {
  return Object.values(filters.value).some((value) => !value)
})

const debouncedSearch = _debounce(async () => {
  setSearch(searchInput.value, filters.value)
}, 200)

watch(searchInput, debouncedSearch)
</script>

<style scoped>
.dropdown-container {
  @apply p-1 min-w-[10rem];
}

.dropdown-item {
  @apply flex items-center px-2 py-1 hover:bg-brave-95 dark:hover:bg-brave-15 rounded cursor-pointer;
}

.dropdown-label {
  @apply text-brave-20 dark:text-brave-90 text-sm font-bold;
}

.dropdown-checkbox {
  @apply rounded text-brave-primary dark:text-brave-60 focus:ring-brave-primary dark:focus:ring-brave-60 mr-2;
}
</style>
