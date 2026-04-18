import { computed } from 'vue'
import { useModal } from 'vue-final-modal'
import EditLyricsV2PublishModal from '@/components/library/edit-lyrics-v2/EditLyricsV2PublishModal.vue'
import { serializeLyricsfile } from '@/utils/lyricsfile.js'

export function useEditLyricsV2Publish({
  audioSource,
  plainLyrics,
  syncedLines,
  lyricsfileDocument,
  isInstrumental,
  saveLyrics,
}) {
  const {
    open: openPublishModalRaw,
    close: closePublishModal,
    patchOptions: patchPublishModalOptions,
  } = useModal({
    component: EditLyricsV2PublishModal,
    attrs: {
      onClose() {
        closePublishModal()
      },
    },
  })

  const serializedLyricsfile = computed(() => {
    // Build track data from audioSource for serialization
    const trackData = {
      title: audioSource.value?.title ?? 'Unknown',
      artist_name: audioSource.value?.artist_name ?? 'Unknown',
      album_name: audioSource.value?.album_name ?? '',
      duration: audioSource.value?.duration ?? 0,
    }

    return (
      serializeLyricsfile({
        track: trackData,
        plainLyrics: plainLyrics.value,
        syncedLines: syncedLines.value,
        baseDocument: lyricsfileDocument.value,
        isInstrumental: isInstrumental.value,
      }) || ''
    )
  })

  const openPublishModal = () => {
    // Build track data from audioSource for the modal
    const trackData = {
      title: audioSource.value?.title ?? 'Unknown',
      artist_name: audioSource.value?.artist_name ?? 'Unknown',
      album_name: audioSource.value?.album_name ?? '',
      duration: audioSource.value?.duration ?? 0,
    }

    patchPublishModalOptions({
      attrs: {
        track: trackData,
        lyricsfile: serializedLyricsfile.value,
      },
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
    serializedLyricsfile,
  }
}
