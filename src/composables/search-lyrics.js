import { ref } from 'vue'
import { useModal } from 'vue-final-modal'
import SearchLyrics from '@/components/library/SearchLyrics.vue'

const searchingTrack = ref(null)

export function useSearchLyrics() {
  const searchLyrics = (track) => {
    searchingTrack.value = track
    openModal()
  }

  const { open: openModal, close: closeModal } = useModal({
    component: SearchLyrics,
    attrs: {
      searchingTrack,
      onClose() {
        closeModal()
      },
      onClosed() {
        searchingTrack.value = null
      }
    },
  })

  return {
    searchingTrack,
    searchLyrics
  }
}
