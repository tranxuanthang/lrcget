import { ref } from 'vue'

const searchValue = ref("")
const filters = ref({
  syncedLyricsTracks: true,
  plainLyricsTracks: true,
  instrumentalTracks: true,
  noLyricsTracks: true,
})

export function useSearchLibrary() {
  const setSearch = (text, filters) => {
    searchValue.value = text
    filters.value = filters
  }

  return {
    searchValue,
    filters,
    setSearch,
  }
}
