import { tokenizeText } from '@/utils/word-tokenizer.js'

const wordWeight = text => {
  const count = tokenizeText(text).length
  return count > 0 ? count : text.length
}

const cloneWord = word => ({
  text: typeof word?.text === 'string' ? word.text : '',
  ...(Number.isFinite(word?.start_ms) ? { start_ms: word.start_ms } : {}),
  ...(Number.isFinite(word?.end_ms) ? { end_ms: word.end_ms } : {}),
})

const joinedWordText = words => words.map(word => word.text || '').join('')

const alignWordsToText = (words, text) => {
  if (!Array.isArray(words) || words.length === 0) {
    return []
  }

  const next = words.map(cloneWord)
  if (joinedWordText(next) === text) {
    return next
  }

  next[0] = { ...next[0], text: next[0].text.trimStart() }
  const lastIndex = next.length - 1
  next[lastIndex] = { ...next[lastIndex], text: next[lastIndex].text.trimEnd() }

  if (joinedWordText(next) !== text || next.some(word => word.text.length === 0)) {
    return []
  }

  return next
}

const wordsFitWindow = (words, startMs, endMs) => {
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs)) {
    return true
  }

  return words.every(word => {
    if (Number.isFinite(word.start_ms) && (word.start_ms < startMs || word.start_ms > endMs)) {
      return false
    }
    if (Number.isFinite(word.end_ms) && (word.end_ms < startMs || word.end_ms > endMs)) {
      return false
    }
    return true
  })
}

const stampSharedBoundary = (leftWords, rightWords, splitMs) => {
  if (!Number.isFinite(splitMs)) {
    return { leftWords, rightWords }
  }

  const nextLeft = leftWords.map(cloneWord)
  const nextRight = rightWords.map(cloneWord)

  if (nextLeft.length > 0) {
    const lastIndex = nextLeft.length - 1
    nextLeft[lastIndex] = { ...nextLeft[lastIndex], end_ms: splitMs }
  }

  if (nextRight.length > 0) {
    nextRight[0] = { ...nextRight[0], start_ms: splitMs }
  }

  return { leftWords: nextLeft, rightWords: nextRight }
}

const partitionWords = (words, caretIndex, splitMs, leftText, rightText, startMs, endMs) => {
  let offset = 0
  let splitAt = words.length

  for (let index = 0; index < words.length; index++) {
    const wordText = words[index].text || ''
    const wordEnd = offset + wordText.length
    if (caretIndex < wordEnd) {
      splitAt = index
      const within = Math.max(0, caretIndex - offset)
      const leftFragment = wordText.slice(0, within)
      const rightFragment = wordText.slice(within)
      const leftWords = words.slice(0, index).map(cloneWord)
      const rightWords = words.slice(index + 1).map(cloneWord)

      if (leftFragment.trim().length > 0) {
        leftWords.push({ ...cloneWord(words[index]), text: leftFragment })
      }
      if (rightFragment.trim().length > 0) {
        rightWords.unshift({ ...cloneWord(words[index]), text: rightFragment })
      }

      const stamped = stampSharedBoundary(leftWords, rightWords, splitMs)
      const alignedLeft = alignWordsToText(stamped.leftWords, leftText)
      const alignedRight = alignWordsToText(stamped.rightWords, rightText)
      const leftEnd = Number.isFinite(splitMs) ? splitMs : endMs
      const rightStart = Number.isFinite(splitMs) ? splitMs : startMs

      return {
        leftWords: wordsFitWindow(alignedLeft, startMs, leftEnd) ? alignedLeft : [],
        rightWords: wordsFitWindow(alignedRight, rightStart, endMs) ? alignedRight : [],
      }
    }

    offset = wordEnd
    if (caretIndex === wordEnd) {
      splitAt = index + 1
      break
    }
  }

  const stamped = stampSharedBoundary(
    words.slice(0, splitAt).map(cloneWord),
    words.slice(splitAt).map(cloneWord),
    splitMs
  )
  const alignedLeft = alignWordsToText(stamped.leftWords, leftText)
  const alignedRight = alignWordsToText(stamped.rightWords, rightText)
  const leftEnd = Number.isFinite(splitMs) ? splitMs : endMs
  const rightStart = Number.isFinite(splitMs) ? splitMs : startMs

  return {
    leftWords: wordsFitWindow(alignedLeft, startMs, leftEnd) ? alignedLeft : [],
    rightWords: wordsFitWindow(alignedRight, rightStart, endMs) ? alignedRight : [],
  }
}

export const splitTimeByWordCount = (leftText, rightText, startMs, endMs) => {
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
    return null
  }

  const leftWeight = wordWeight(leftText)
  const rightWeight = wordWeight(rightText)
  if (leftWeight + rightWeight <= 0) {
    return null
  }

  const ratio = leftWeight / (leftWeight + rightWeight)
  let splitMs = Math.round(startMs + (endMs - startMs) * ratio)

  if (endMs - startMs >= 2) {
    splitMs = Math.min(endMs - 1, Math.max(startMs + 1, splitMs))
  }

  return splitMs
}

export const splitSyncedLineAtCaret = (line, text, caretIndex) => {
  const source = typeof text === 'string' ? text : ''
  const caret = Number.isInteger(caretIndex) ? caretIndex : 0
  const clamped = Math.max(0, Math.min(source.length, caret))
  const leftText = source.slice(0, clamped).trimEnd()
  const rightText = source.slice(clamped).trimStart()

  if (!leftText || !rightText) {
    return null
  }

  const startMs = line?.start_ms
  const endMs = line?.end_ms
  const splitMs = splitTimeByWordCount(leftText, rightText, startMs, endMs)
  const storedWords = Array.isArray(line?.words) ? line.words : []
  const storedText = storedWords.map(word => word.text || '').join('')
  const partitioned =
    storedWords.length > 0 && storedText === source
      ? partitionWords(storedWords, clamped, splitMs, leftText, rightText, startMs, endMs)
      : { leftWords: [], rightWords: [] }

  return {
    leftText,
    rightText,
    splitMs,
    leftWords: partitioned.leftWords,
    rightWords: partitioned.rightWords,
  }
}
