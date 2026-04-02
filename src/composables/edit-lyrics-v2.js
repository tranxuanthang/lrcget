import { ref } from 'vue'
import { useModal } from 'vue-final-modal'
import EditLyricsV2 from '@/components/library/EditLyricsV2.vue'

const editingTrack = ref(null)

export function useEditLyricsV2() {
  const editLyricsV2 = (track) => {
    editingTrack.value = track
    openModal()
  }

  const { open: openModal, close: closeModal } = useModal({
    component: EditLyricsV2,
    attrs: {
      onClose() {
        closeModal()
      },
      onClosed() {
        editingTrack.value = null
      }
    }
  })

  return {
    editingTrack,
    editLyricsV2
  }
}
