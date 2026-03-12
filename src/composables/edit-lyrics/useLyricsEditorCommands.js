import { EditorView } from '@codemirror/view'
import { parseLine } from 'lrc-kit'
import { detectStandard, timestampToString } from '@/utils/lyrics.js'
import { shiftSelectedTimestamps } from '@/composables/edit-lyrics/shiftSelectedTimestamps.js'

export function useLyricsEditorCommands({ view, unifiedLyrics, progress, duration, seek, onLyricsChange }) {
  const syncLine = (moveNext = true) => {
    if (!view.value) {
      return
    }

    const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
    const standard = detectStandard(unifiedLyrics.value)
    const replacedText = currentLine.text.replace(
      /^(\s*\[(.*)\]\s*)*/g,
      `[${timestampToString(progress.value, standard.msPrecision)}]${standard.space ? ' ' : ''}`
    )

    view.value.dispatch({
      changes: {
        from: currentLine.from,
        to: currentLine.to,
        insert: replacedText
      }
    })

    if (moveNext) {
      const selectedLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
      let targetLineNumber = selectedLine.number

      while (targetLineNumber + 1 <= view.value.state.doc.lines) {
        const nextLine = view.value.state.doc.line(targetLineNumber + 1)

        if (nextLine.text.trim() !== '') {
          break
        }

        targetLineNumber++
      }

      if (targetLineNumber + 1 <= view.value.state.doc.lines) {
        const targetLine = view.value.state.doc.line(targetLineNumber + 1)

        view.value.dispatch({
          selection: {
            anchor: targetLine.from
          }
        })

        view.value.dispatch({
          effects: EditorView.scrollIntoView(targetLine.from, { y: 'center', behavior: 'smooth' })
        })
      }
    }

    view.value.focus()
    onLyricsChange(view.value.state.doc.toString())
  }

  const repeatLine = () => {
    if (!view.value) {
      return
    }

    const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
    const parsed = parseLine(currentLine.text)

    if (parsed.type === 'TIME') {
      seek(parsed.timestamps[0])
    }
  }

  const shiftSelectionBy = (delta) => {
    if (!view.value) {
      return
    }

    const { changes, firstTimestamp } = shiftSelectedTimestamps({
      view: view.value,
      delta,
      maxTimestamp: delta > 0 ? duration.value : Infinity
    })

    if (changes.length === 0) {
      return
    }

    view.value.dispatch({ changes })
    view.value.focus()
    onLyricsChange(view.value.state.doc.toString())

    if (firstTimestamp !== null) {
      seek(firstTimestamp)
    }
  }

  const rewind100 = () => shiftSelectionBy(-0.1)
  const fastForward100 = () => shiftSelectionBy(+0.1)

  const markAsInstrumental = () => {
    if (!view.value) {
      return
    }

    view.value.dispatch({
      changes: {
        from: 0,
        to: view.value.state.doc.length,
        insert: '[au: instrumental]'
      }
    })

    view.value.focus()
    onLyricsChange(view.value.state.doc.toString())
  }

  return {
    syncLine,
    repeatLine,
    rewind100,
    fastForward100,
    markAsInstrumental
  }
}
