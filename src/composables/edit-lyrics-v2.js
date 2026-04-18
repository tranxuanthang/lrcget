import { ref } from 'vue'
import { useModal } from 'vue-final-modal'
import EditLyricsV2 from '@/components/library/EditLyricsV2.vue'

// State for the edit modal
const editingAudioSource = ref(null)
const editingLyricsfile = ref(null)
const editingTrackId = ref(null)

export function useEditLyricsV2() {
  const setEditingData = ({ audioSource, lyricsfile, trackId }) => {
    editingAudioSource.value = audioSource
    editingLyricsfile.value = lyricsfile
    editingTrackId.value = trackId
  }

  const editLyricsV2 = ({ audioSource, lyricsfile, trackId }) => {
    editingAudioSource.value = audioSource
    editingLyricsfile.value = lyricsfile
    editingTrackId.value = trackId
    openModal()
  }

  const { open: openModal, close: closeModal } = useModal({
    component: EditLyricsV2,
    attrs: {
      audioSource: editingAudioSource,
      lyricsfile: editingLyricsfile,
      trackId: editingTrackId,
      onClose() {
        closeModal()
      },
      onClosed() {
        editingAudioSource.value = null
        editingLyricsfile.value = null
        editingTrackId.value = null
      },
    },
  })

  return {
    editingAudioSource,
    editingLyricsfile,
    editingTrackId,
    setEditingData,
    editLyricsV2,
  }
}
