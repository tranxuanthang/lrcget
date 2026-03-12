import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { executeLint as executeLyricsLint } from '@/utils/lyrics-lint.js'
import { executeLint as executePlainTextLint } from '@/utils/plain-text-lint.js'

const getInitialLyrics = (track) => track?.lrc_lyrics || track?.txt_lyrics || ''

const buildLintState = (lyrics) => ({
  lyricsLintResult: executeLyricsLint(lyrics),
  plainTextLintResult: executePlainTextLint(lyrics)
})

export function useLyricsDocument({ editingTrack, toast }) {
  const unifiedLyrics = ref('')
  const isDirty = ref(false)
  const lyricsLintResult = ref([])
  const plainTextLintResult = ref([])

  const syncDerivedState = (lyrics) => {
    const nextState = buildLintState(lyrics)
    lyricsLintResult.value = nextState.lyricsLintResult
    plainTextLintResult.value = nextState.plainTextLintResult
  }

  const initializeLyrics = () => {
    const nextLyrics = getInitialLyrics(editingTrack.value)
    unifiedLyrics.value = nextLyrics
    isDirty.value = false
    syncDerivedState(nextLyrics)
  }

  const updateLyrics = (lyrics, { markDirty = true } = {}) => {
    unifiedLyrics.value = lyrics
    isDirty.value = markDirty ? true : isDirty.value
    syncDerivedState(lyrics)
  }

  const saveLyrics = async () => {
    if (!editingTrack.value) {
      return
    }

    try {
      const isLyricsSynced = /^\[.*\]/m.test(unifiedLyrics.value)

      await invoke('save_lyrics', {
        trackId: editingTrack.value.id,
        plainLyrics: unifiedLyrics.value.replace(/^\[(.*)\] */mg, ''),
        syncedLyrics: isLyricsSynced ? unifiedLyrics.value : ''
      })

      isDirty.value = false
    } catch (error) {
      console.error(error)
      toast.error(error)
    }
  }

  return {
    unifiedLyrics,
    isDirty,
    lyricsLintResult,
    plainTextLintResult,
    initializeLyrics,
    updateLyrics,
    saveLyrics
  }
}
