import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const searchValue = ref("")

export function useSearchLibrary() {
  const setSearch = (text) => {
    searchValue.value = text
    invoke('set_search', { query: text })
  }

  return {
    searchValue,
    setSearch,
  }
}
