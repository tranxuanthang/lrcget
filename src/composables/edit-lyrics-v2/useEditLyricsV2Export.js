import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

export function useEditLyricsV2Export({ audioSource, saveLyrics, serializedLyricsfile, toast }) {
  const isExporting = ref(false)

  const exportLyrics = async ({ plainText, syncedLrc, embedIntoTrack }) => {
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
      // Only library tracks can use embedIntoTrack
      const isLibraryTrack = audioSource.value?.type === 'library'

      const results = await invoke('export_lyrics', {
        trackId: isLibraryTrack ? audioSource.value.id : null,
        formats,
        lyricsfile: serializedLyricsfile.value,
      })

      const succeeded = results.filter(result => result.status.type === 'success')
      const failed = results.filter(result => result.status.type !== 'success')

      if (succeeded.length > 0 && failed.length === 0) {
        toast.success(
          succeeded.length === 1
            ? `Exported to ${succeeded[0].format} successfully.`
            : `Exported ${succeeded.length} lyrics targets successfully.`
        )
        return true
      }

      if (succeeded.length > 0) {
        toast.warning(`Export completed with partial failures.`)
        return true
      }

      toast.error(failed.map(result => result.status.message || 'Unknown error').join('; '))
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
