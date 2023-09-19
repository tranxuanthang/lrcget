import { ref } from 'vue'

const editingTrack = ref(null)

export function useEditLyrics() {
  const editLyrics = (track) => {
    editingTrack.value = track
  }

  return {
    editingTrack,
    editLyrics
  }
}
