import { parseLine } from 'lrc-kit'
import { timestampToString } from '@/utils/lyrics.js'

export function shiftSelectedTimestamps({ view, delta, maxTimestamp = Infinity }) {
  const changes = []
  let firstTimestamp = null
  const boundedMaxTimestamp = Number.isFinite(maxTimestamp) ? maxTimestamp : Infinity

  for (const range of view.state.selection.ranges) {
    const startLine = view.state.doc.lineAt(range.from)
    const endLine = view.state.doc.lineAt(range.to)

    for (let lineNo = startLine.number; lineNo <= endLine.number; lineNo++) {
      const currentLine = view.state.doc.line(lineNo)
      const parsed = parseLine(currentLine.text)

      if (parsed.type !== 'TIME') {
        continue
      }

      const nextTimestamp = Math.max(0, Math.min(boundedMaxTimestamp, parsed.timestamps[0] + delta))
      const replacedText = currentLine.text.replace(/^(\s*\[(.*)\]\s*)*/g, `[${timestampToString(nextTimestamp)}] `)

      if (firstTimestamp === null) {
        firstTimestamp = nextTimestamp
      }

      changes.push({
        from: currentLine.from,
        to: currentLine.to,
        insert: replacedText
      })
    }
  }

  return {
    changes,
    firstTimestamp
  }
}
