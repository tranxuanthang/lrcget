import { computed } from 'vue'
import { useModal } from 'vue-final-modal'
import PublishLyrics from '@/components/library/edit-lyrics/PublishLyrics.vue'
import PublishPlainText from '@/components/library/edit-lyrics/PublishPlainText.vue'
import { isSynchronizedLyrics } from '@/utils/lyrics.js'

export function useLyricsPublish({ editingTrack, unifiedLyrics, lyricsLintResult, plainTextLintResult }) {
  const { open: openPublishLyricsModal, close: closePublishLyricsModal, patchOptions: patchPublishLyricsModalOptions } = useModal({
    component: PublishLyrics,
    attrs: {
      onClose() {
        closePublishLyricsModal()
      }
    }
  })

  const { open: openPublishPlainTextModal, close: closePublishPlainTextModal, patchOptions: patchPublishPlainTextModalOptions } = useModal({
    component: PublishPlainText,
    attrs: {
      onClose() {
        closePublishPlainTextModal()
      }
    }
  })

  const publishMode = computed(() => isSynchronizedLyrics(unifiedLyrics.value) ? 'synced' : 'plain-text')
  const publishStatus = computed(() => {
    if (lyricsLintResult.value.length === 0) {
      return 'clean'
    }

    if (plainTextLintResult.value.length === 0) {
      return 'plain-text-only'
    }

    return 'error'
  })

  const publishButtonTooltip = computed(() => {
    return publishMode.value === 'synced'
      ? 'Publish synchronized lyrics to LRCLIB service'
      : 'Publish plain text lyrics to LRCLIB service'
  })

  const publishStatusTooltip = computed(() => {
    if (publishStatus.value === 'clean') {
      return 'No errors detected, you can publish it now'
    }

    if (publishStatus.value === 'plain-text-only') {
      return 'Lyrics not synchronized\nYou can still publish it, but consider synchronizing it to help others'
    }

    return 'Lyrics error detected\nPress the publish button for details'
  })

  const openPublishModal = () => {
    if (!editingTrack.value) {
      return
    }

    const attrs = {
      title: editingTrack.value.title,
      albumName: editingTrack.value.album_name,
      artistName: editingTrack.value.artist_name,
      duration: editingTrack.value.duration,
      lyrics: unifiedLyrics.value
    }

    if (publishMode.value === 'synced') {
      patchPublishLyricsModalOptions({
        attrs: {
          ...attrs,
          lintResult: lyricsLintResult.value
        }
      })
      openPublishLyricsModal()
      return
    }

    patchPublishPlainTextModalOptions({
      attrs: {
        ...attrs,
        lintResult: plainTextLintResult.value
      }
    })
    openPublishPlainTextModal()
  }

  return {
    publishMode,
    publishStatus,
    publishButtonTooltip,
    publishStatusTooltip,
    openPublishModal
  }
}
