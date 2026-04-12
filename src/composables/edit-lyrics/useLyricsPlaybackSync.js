import { ref, watch } from 'vue'
import { Lrc, Runner } from 'lrc-kit'

export function useLyricsPlaybackSync({ progress, unifiedLyrics, view, addLineHighlight }) {
  const currentIndex = ref(-1)
  let runner = null

  const clearHighlight = () => {
    if (!view.value) {
      return
    }

    view.value.dispatch({ effects: addLineHighlight.of(null) })
  }

  const refreshRunner = (lyrics = unifiedLyrics.value) => {
    runner = new Runner(Lrc.parse(lyrics || ''))
  }

  watch(
    unifiedLyrics,
    newLyrics => {
      refreshRunner(newLyrics)

      if (!newLyrics) {
        currentIndex.value = -1
        clearHighlight()
      }
    },
    { immediate: true }
  )

  watch(progress, newProgress => {
    if (!view.value || !runner || !unifiedLyrics.value) {
      return
    }

    runner.timeUpdate(newProgress)

    const nextIndex = runner.curIndex()
    currentIndex.value = nextIndex === null ? -1 : nextIndex

    if (currentIndex.value >= 0) {
      const line = view.value.state.doc.line(currentIndex.value + 1)
      view.value.dispatch({ effects: addLineHighlight.of(line.from) })
      return
    }

    clearHighlight()
  })

  const resetPlaybackSync = () => {
    runner = null
    currentIndex.value = -1
    clearHighlight()
  }

  return {
    currentIndex,
    refreshRunner,
    resetPlaybackSync,
  }
}
