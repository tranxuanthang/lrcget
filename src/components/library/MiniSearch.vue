<template>
  <div class="w-[16rem] relative h-full">
    <input
      v-model="searchInput"
      type="text"
      class="h-full input px-[2rem] py-1.5 pr-1.5 w-[16rem] dark:text-neutral-200"
      placeholder="Search for tracks..."
      autofocus
    />
    <div class="absolute top-0 left-0 w-[2rem] h-full flex justify-center items-center pl-0.5">
      <Magnify class="text-neutral-800 dark:text-neutral-500" />
    </div>
    <div class="absolute top-0 right-0 h-full flex items-center px-1 gap-1">
      <button
        v-if="searchInput !== ''"
          class="w-[1.5rem] h-[1.5rem] flex justify-center items-center text-neutral-800 group-hover:text-neutral-800 dark:text-neutral-500 dark:hover:text-neutral-300 rounded-full"
        @click="searchInput = ''"
      >
        <Close />
      </button>
      <VDropdown theme="lrcget-dropdown" placement="top-start">
        <button
        class="w-[1.5rem] h-[1.5rem] flex justify-center items-center text-neutral-800 group-hover:text-neutral-800 dark:text-neutral-500 dark:hover:text-neutral-300 rounded-full"
          :class="{ 'bg-neutral-200 dark:bg-neutral-700': isFilters }"
        >
          <Filter />
        </button>
        <template #popper>
          <div class="dropdown-container">
            <label class="dropdown-item">
              <CheckboxButton
                id="synced-lyrics"
                v-model="filters.syncedLyricsTracks"
                name="synced-lyrics"
              >
                <span class="dropdown-label">Synced lyrics tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                id="plain-lyrics"
                v-model="filters.plainLyricsTracks"
                name="plain-lyrics"
              >
                <span class="dropdown-label">Plain lyrics tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton
                id="instrumental"
                v-model="filters.instrumentalTracks"
                name="instrumental"
              >
                <span class="dropdown-label">Instrumental tracks</span>
              </CheckboxButton>
            </label>
            <label class="dropdown-item">
              <CheckboxButton id="no-lyrics" v-model="filters.noLyricsTracks" name="no-lyrics">
                <span class="dropdown-label">No lyrics tracks</span>
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
import Magnify from '~icons/mdi/magnify'
import Close from '~icons/mdi/close'
import Filter from '~icons/mdi/filter'
import _debounce from 'lodash/debounce'
import CheckboxButton from '@/components/common/CheckboxButton.vue'

const { setSearch, filters } = useSearchLibrary()

const searchInput = ref('')

const isFilters = computed(() => {
  return Object.values(filters.value).some(value => !value)
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
  @apply flex items-center px-2 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700 rounded cursor-pointer;
}

.dropdown-label {
  @apply text-neutral-800 dark:text-neutral-300 text-sm font-bold;
}

.dropdown-checkbox {
  @apply rounded text-hoa-1100 dark:text-hoa-1100 focus:ring-hoa-1100 dark:focus:ring-hoa-1100 mr-2;
}
</style>
