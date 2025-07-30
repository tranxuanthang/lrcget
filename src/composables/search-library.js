import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const searchValue = ref("")
const hideTracksWithLyrics = ref(false)

export function useSearchLibrary() {
  const setSearch = (text) => {
    searchValue.value = text
  }

  const setHideTracksWithLyrics = (value) => {
    hideTracksWithLyrics.value = value
  }

  return {
    searchValue,
    hideTracksWithLyrics,
    setSearch,
    setHideTracksWithLyrics,
  }
}
