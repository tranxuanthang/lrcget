import { useModal } from 'vue-final-modal'
import EditLyricsV2PublishModal from '@/components/library/edit-lyrics-v2/EditLyricsV2PublishModal.vue'

export function useEditLyricsV2Publish({
  audioSource,
  lyricsfileDocument,
  serializedLyricsfile,
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

  const openPublishModal = () => {
    // Build track data for the modal
    // Prefer existing lyricsfile metadata first, then fall back to audioSource/track data
    const trackData = {
      title: lyricsfileDocument?.value?.metadata?.title ?? audioSource.value?.title ?? null,
      artist_name:
        lyricsfileDocument?.value?.metadata?.artist ?? audioSource.value?.artist_name ?? null,
      album_name:
        lyricsfileDocument?.value?.metadata?.album ?? audioSource.value?.album_name ?? null,
      duration:
        (lyricsfileDocument?.value?.metadata?.duration_ms != null
          ? lyricsfileDocument.value.metadata.duration_ms / 1000
          : null) ??
        audioSource.value?.duration ??
        null,
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
  }
}
