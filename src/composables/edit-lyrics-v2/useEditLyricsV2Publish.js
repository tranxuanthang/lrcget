import { computed } from 'vue'
import { useModal } from 'vue-final-modal'
import EditLyricsV2PublishModal from '@/components/library/edit-lyrics-v2/EditLyricsV2PublishModal.vue'
import { serializeLyricsfile } from '@/utils/lyricsfile.js'

export function useEditLyricsV2Publish({
  editingTrack,
  plainLyrics,
  syncedLines,
  lyricsfileDocument,
  isInstrumental,
  saveLyrics
}) {
  const { open: openPublishModalRaw, close: closePublishModal, patchOptions: patchPublishModalOptions } = useModal({
    component: EditLyricsV2PublishModal,
    attrs: {
      onClose() {
        closePublishModal()
      }
    }
  })

  const serializedLyricsfile = computed(() => {
    if (!editingTrack.value) {
      return ''
    }

    return serializeLyricsfile({
      track: editingTrack.value,
      plainLyrics: plainLyrics.value,
      syncedLines: syncedLines.value,
      baseDocument: lyricsfileDocument.value,
      isInstrumental: isInstrumental.value
    }) || ''
  })

  const openPublishModal = () => {
    if (!editingTrack.value) {
      return
    }

    patchPublishModalOptions({
      attrs: {
        track: editingTrack.value,
        lyricsfile: serializedLyricsfile.value
      }
    })

    openPublishModalRaw()
  }

  const saveAndPublish = async () => {
    const didSave = await saveLyrics()
    if (!didSave) {
      return
    }

    openPublishModal()
  }

  return {
    saveAndPublish,
    openPublishModal,
    serializedLyricsfile
  }
}
