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

const wordWindowEndMs = (words, index, lineEndMs) => {
  const word = words[index]
  if (Number.isFinite(word?.end_ms)) {
    return word.end_ms
  }

  const nextStartMs = words[index + 1]?.start_ms
  if (Number.isFinite(nextStartMs)) {
    return nextStartMs
  }

  return lineEndMs
}

const boundarySplitMs = (leftWord, rightWord) => {
  if (Number.isFinite(rightWord?.start_ms)) {
    return rightWord.start_ms
  }

  if (Number.isFinite(leftWord?.end_ms)) {
    return leftWord.end_ms
  }

  return null
}

// A caret in the trailing spaces of a word is the boundary before the next word.
const isTrailingSpaceCaret = (wordText, within) => {
  if (within <= 0 || within >= wordText.length) {
    return false
  }

  const leftFragment = wordText.slice(0, within)
  const rightFragment = wordText.slice(within)
  return leftFragment.trim().length > 0 && rightFragment.trim().length === 0
}

const findWordSplit = (words, caretIndex, lineEndMs) => {
  let offset = 0

  for (let index = 0; index < words.length; index++) {
    const wordText = words[index].text || ''
    const wordEnd = offset + wordText.length

    if (caretIndex < wordEnd) {
      const within = caretIndex - offset
      if (within <= 0 || isTrailingSpaceCaret(wordText, within)) {
        const splitAt = within <= 0 ? index : index + 1
        return {
          midWord: false,
          splitMs: boundarySplitMs(words[splitAt - 1], words[splitAt]),
        }
      }

      const wordStartMs = words[index].start_ms
      const wordEndMs = wordWindowEndMs(words, index, lineEndMs)
      let splitMs = null
      if (Number.isFinite(wordStartMs) && Number.isFinite(wordEndMs) && wordEndMs > wordStartMs) {
        splitMs = Math.round(wordStartMs + (wordEndMs - wordStartMs) * (within / wordText.length))
      }

      return { midWord: true, splitMs }
    }

    if (caretIndex === wordEnd) {
      return {
        midWord: false,
        splitMs: boundarySplitMs(words[index], words[index + 1]),
      }
    }

    offset = wordEnd
  }

  return null
}

const partitionWords = (words, caretIndex, splitMs, leftText, rightText, startMs, endMs, preserveWordTimes = false) => {
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

      const placed = preserveWordTimes
        ? { leftWords, rightWords }
        : stampSharedBoundary(leftWords, rightWords, splitMs)
      const alignedLeft = alignWordsToText(placed.leftWords, leftText)
      const alignedRight = alignWordsToText(placed.rightWords, rightText)
      if (preserveWordTimes) {
        return { leftWords: alignedLeft, rightWords: alignedRight }
      }

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

  const leftWords = words.slice(0, splitAt).map(cloneWord)
  const rightWords = words.slice(splitAt).map(cloneWord)
  const placed = preserveWordTimes
    ? { leftWords, rightWords }
    : stampSharedBoundary(leftWords, rightWords, splitMs)
  const alignedLeft = alignWordsToText(placed.leftWords, leftText)
  const alignedRight = alignWordsToText(placed.rightWords, rightText)
  if (preserveWordTimes) {
    return { leftWords: alignedLeft, rightWords: alignedRight }
  }

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
  const storedWords = Array.isArray(line?.words) ? line.words : []
  const storedText = storedWords.map(word => word.text || '').join('')
  const canUseWords = storedWords.length > 0 && storedText === source
  const wordSplit = canUseWords ? findWordSplit(storedWords, clamped, endMs) : null
  const splitFromWords = Number.isFinite(wordSplit?.splitMs)
  const splitMs = splitFromWords
    ? wordSplit.splitMs
    : splitTimeByWordCount(leftText, rightText, startMs, endMs)
  const partitioned = canUseWords
    ? partitionWords(
        storedWords,
        clamped,
        splitMs,
        leftText,
        rightText,
        startMs,
        endMs,
        splitFromWords && wordSplit.midWord === false
      )
    : { leftWords: [], rightWords: [] }

  return {
    leftText,
    rightText,
    splitMs,
    leftWords: partitioned.leftWords,
    rightWords: partitioned.rightWords,
  }
}
