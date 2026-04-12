import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

export function useEditLyricsV2Export({ editingTrack, saveLyrics, serializedLyricsfile, toast }) {
  const isExporting = ref(false)

  const exportLyrics = async ({ plainText, syncedLrc, embedIntoTrack }) => {
    if (!editingTrack.value) {
      return false
    }

    const formats = []

    if (plainText) {
      formats.push('txt')
    }

    if (syncedLrc) {
      formats.push('lrc')
    }

    if (embedIntoTrack) {
      formats.push('embedded')
    }

    if (formats.length === 0) {
      toast.error('Select at least one export format.')
      return false
    }

    const didSave = await saveLyrics()
    if (!didSave) {
      return false
    }

    isExporting.value = true

    try {
      const results = await invoke('export_lyrics', {
        trackId: editingTrack.value.id,
        formats,
        lyricsfile: serializedLyricsfile.value,
      })

      const succeeded = results.filter(result => result.success)
      const failed = results.filter(result => !result.success)

      if (succeeded.length > 0 && failed.length === 0) {
        toast.success(
          succeeded.length === 1
            ? succeeded[0].message
            : `Exported ${succeeded.length} lyrics targets successfully.`
        )
        return true
      }

      if (succeeded.length > 0) {
        toast.warning(
          `Export completed with partial failures: ${failed.map(result => result.message).join('; ')}`
        )
        return true
      }

      toast.error(failed.map(result => result.message).join('; '))
      return false
    } catch (error) {
      console.error(error)
      toast.error(error)
      return false
    } finally {
      isExporting.value = false
    }
  }

  return {
    exportLyrics,
    isExporting,
  }
}
