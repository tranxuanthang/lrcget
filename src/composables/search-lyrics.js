import { ref, onMounted, onUnmounted } from 'vue'

const searchingTrack = ref(null)

export function useSearchLyrics() {
  const searchLyrics = (track) => {
    searchingTrack.value = track
  }

  return {
    searchingTrack,
    searchLyrics
  }
}
