/**
 * Word tokenization utilities for lyrics text.
 * Handles both Latin scripts (space-delimited) and CJK (character-based).
 * Also handles Japanese phonetic modifiers:
 *   - Yōon (small kana: ゃゅょ etc.) — merges backward with preceding character
 *   - Sokuon (small tsu: っッ) — merges forward with following character
 *   - Chōonpu (long vowel: ー) — merges backward with preceding character
 */

// CJK base character Unicode ranges (ideographs, syllables, kana)
const CJK_BASE_RANGES = [
  [0x4e00, 0x9fff], // CJK Unified Ideographs
  [0x3040, 0x309f], // Hiragana
  [0x30a0, 0x30ff], // Katakana
  [0xac00, 0xd7af], // Hangul Syllables
  [0x3400, 0x4dbf], // CJK Extension A
  [0xf900, 0xfaff], // CJK Compatibility Ideographs
  [0xff66, 0xff9f], // Halfwidth Katakana
  [0x31f0, 0x31ff], // Katakana Phonetic Extensions
]

function isInRanges(code, ranges) {
  return ranges.some(([start, end]) => code >= start && code <= end)
}

/**
 * Check if a character is a CJK base character (not punctuation)
 * @param {string} char - Single character
 * @returns {boolean}
 */
function isCJKBaseChar(char) {
  if (!char || char.length !== 1) return false
  if (isYoon(char) || isSokuon(char) || isChoonpu(char)) return false
  return isInRanges(char.charCodeAt(0), CJK_BASE_RANGES)
}

// Opening punctuation that merges with the following CJK token
const CJK_OPEN_PUNCT = new Set([
  '\u0028',
  '\u3008', '\u300a', '\u300c', '\u300e', '\u3010', '\u3014', '\u3016',
  '\uff08', '\uff3b', '\uff5b', '\uff5f', '\uff62',
])

// Closing punctuation that merges with the preceding CJK token
const CJK_CLOSE_PUNCT = new Set([
  '\u0029',
  '\u3001', '\u3002', '\u3009', '\u300b', '\u300d', '\u300f', '\u3011',
  '\u3015', '\u3017',
  '\uff09', '\uff0c', '\uff0e', '\uff1a', '\uff1b', '\uff1f', '\uff01',
  '\uff3d', '\uff5d', '\uff60', '\uff63', '\uff64',
])

/**
 * Check if a character is CJK opening punctuation
 * @param {string} char - Single character
 * @returns {boolean}
 */
function isCJKOpenPunct(char) {
  return CJK_OPEN_PUNCT.has(char)
}

/**
 * Check if a character is CJK closing punctuation
 * @param {string} char - Single character
 * @returns {boolean}
 */
function isCJKClosePunct(char) {
  return CJK_CLOSE_PUNCT.has(char)
}

// ---------------------------------------------------------------------------
// Japanese phonetic modifier characters
// ---------------------------------------------------------------------------

// Yōon (small kana) — merge backward with preceding character
const YOON_CHARS = new Set([
  // Hiragana small kana
  '\u3083', '\u3085', '\u3087', // ゃ ゅ ょ
  '\u3041', '\u3043', '\u3045', '\u3047', '\u3049', // ぁ ぃ ぅ ぇ ぉ
  // Katakana small kana
  '\u30e3', '\u30e5', '\u30e7', // ャ ュ ョ
  '\u30a1', '\u30a3', '\u30a5', '\u30a7', '\u30a9', // ァ ィ ゥ ェ ォ
  // Halfwidth Katakana small kana
  '\uff6c', '\uff6d', '\uff6e', // ｬ ｭ ｮ
  '\uff67', '\uff68', '\uff69', '\uff6a', '\uff6b', // ｧ ｨ ｩ ｪ ｫ
])

// Sokuon (small tsu) — merge forward with following character
const SOKUON_CHARS = new Set([
  '\u3063', // っ (hiragana)
  '\u30c3', // ッ (katakana)
  '\uff6f', // ｯ (halfwidth katakana)
])

// Chōonpu (long vowel mark) — merge backward with preceding character
const CHOONPU_CHARS = new Set([
  '\u30fc', // ー (fullwidth)
  '\uff70', // ｰ (halfwidth)
])

function isYoon(char) {
  return YOON_CHARS.has(char)
}

function isSokuon(char) {
  return SOKUON_CHARS.has(char)
}

function isChoonpu(char) {
  return CHOONPU_CHARS.has(char)
}

function absorbsSokuon(char) {
  return isCJKBaseChar(char) || isYoon(char) || isChoonpu(char)
}

// ---------------------------------------------------------------------------
// Tokenization
// ---------------------------------------------------------------------------

/**
 * Tokenize text into words.
 * - Latin scripts: space-delimited, preserve trailing spaces in tokens
 * - CJK: character-based, with punctuation grouped and spaces preserved as trailing
 * - Japanese: sokuon merges forward, yōon and chōonpu merge backward
 * @param {string} text - Input text to tokenize
 * @returns {Array<{text: string, isCJK: boolean}>} Array of word tokens
 */
export function tokenizeText(text) {
  if (!text || typeof text !== 'string') {
    return []
  }

  const tokens = []
  let buffer = ''
  let isCJK = false
  let hasBuffer = false
  let pendingOpen = ''
  let pendingSokuon = ''

  function flush() {
    if (hasBuffer && buffer) {
      tokens.push({ text: buffer, isCJK })
    }
    hasBuffer = false
    buffer = ''
  }

  function start(text, type) {
    flush()
    buffer = text
    isCJK = type
    hasBuffer = true
  }

  function append(text) {
    if (!hasBuffer) {
      hasBuffer = true
      isCJK = false
    }
    buffer += text
  }

  function appendToLast(text) {
    const last = tokens.at(-1)
    if (last?.isCJK) {
      last.text += text
    } else {
      start(text, true)
    }
  }

  function flushSokuon() {
    if (pendingSokuon) {
      start(pendingSokuon, true)
      pendingSokuon = ''
    }
  }

  for (const char of text) {
    // Sokuon accumulates — it merges with the following absorbable character
    if (isSokuon(char)) {
      pendingSokuon += char
      continue
    }

    // If pending sokuon and the next char cannot absorb it, flush it first
    if (pendingSokuon && !absorbsSokuon(char)) {
      flushSokuon()
    }

    const sokuonPrefix = pendingSokuon
    pendingSokuon = ''

    if (char === ' ') {
      append(' ')
      continue
    }

    const isBase = isCJKBaseChar(char)
    const isOpen = isCJKOpenPunct(char)
    const isClose = isCJKClosePunct(char)
    const isYoonChar = isYoon(char)
    const isChoonChar = isChoonpu(char)

    if (isOpen) {
      // Opening punctuation is accumulated and prepended to the next CJK base char.
      // Flush any pending non-CJK token so the open bracket doesn't get swallowed.
      if (hasBuffer && !isCJK) flush()
      pendingOpen += sokuonPrefix + char
      continue
    }

    if (isClose) {
      // Closing punctuation merges with the current buffer (CJK or non-CJK),
      // or with the previous CJK token if there is no current buffer.
      const text = sokuonPrefix + char
      if (hasBuffer) {
        buffer += text
      } else {
        appendToLast(text)
      }
      continue
    }

    if (isBase) {
      // Each CJK base character starts a new CJK token.
      // Prepend any accumulated opening punctuation and sokuon.
      start(pendingOpen + sokuonPrefix + char, true)
      pendingOpen = ''
      continue
    }

    if (isYoonChar || isChoonChar) {
      // Yōon and chōonpu merge backward with the current or previous CJK token
      const text = sokuonPrefix + char
      if (hasBuffer && isCJK) {
        buffer += text
      } else if (hasBuffer && !isCJK) {
        start(text, true)
      } else {
        appendToLast(text)
      }
      continue
    }

    // Non-CJK character (Latin, numbers, ASCII symbols, etc.)
    const full = pendingOpen ? pendingOpen + sokuonPrefix + char : sokuonPrefix + char
    pendingOpen = ''
    if (hasBuffer && !isCJK) {
      buffer += full
    } else {
      start(full, false)
    }
  }

  flush()

  if (pendingOpen) {
    appendToLast(pendingOpen)
    flush()
  }

  if (pendingSokuon) {
    appendToLast(pendingSokuon)
    flush()
  }

  // For non-CJK tokens, split by spaces while preserving trailing spaces
  const finalTokens = []
  for (const token of tokens) {
    if (token.isCJK) {
      finalTokens.push(token)
    } else {
      for (const lt of splitLatinText(token.text)) {
        finalTokens.push({ text: lt, isCJK: false })
      }
    }
  }

  return finalTokens
}

/**
 * Split Latin text by spaces, preserving spaces in tokens
 * @param {string} text - Latin text
 * @returns {string[]} Array of word tokens with trailing spaces
 */
function splitLatinText(text) {
  const tokens = []
  let currentWord = ''

  for (let i = 0; i < text.length; i++) {
    const char = text[i]

    if (char === ' ') {
      currentWord += char
    } else {
      if (currentWord && currentWord.endsWith(' ')) {
        tokens.push(currentWord)
        currentWord = char
      } else {
        currentWord += char
      }
    }
  }

  if (currentWord) {
    tokens.push(currentWord)
  }

  return tokens
}

/**
 * Check if line has valid word tokens
 * @param {Object} line - Line object with words array
 * @returns {boolean}
 */
export function hasValidWords(line) {
  if (!line?.words || !Array.isArray(line.words)) {
    return false
  }

  if (line.words.length === 0) {
    return false
  }

  // Check if words array matches the line text
  const reconstructedText = line.words.map(w => w.text || '').join('')
  if (reconstructedText !== line.text) {
    return false
  }

  return true
}

/**
 * Auto-generate word tokens from line text
 * @param {Object} line - Line object with text
 * @returns {Array<{text: string, start_ms?: number}>} Array of word tokens
 */
export function generateWordsFromLine(line) {
  if (!line?.text) {
    return []
  }

  const tokens = tokenizeText(line.text)
  return tokens.map(token => ({
    text: token.text,
  }))
}

/**
 * Distribute word timings evenly across a line's time window
 * @param {Array} words - Array of word objects with text
 * @param {number} startMs - Line start time in ms
 * @param {number} endMs - Line end time in ms
 * @returns {Array<{text: string, start_ms: number}>} Words with distributed timings
 */
export function distributeWordTimings(words, startMs, endMs) {
  if (!Array.isArray(words) || words.length === 0) {
    return []
  }

  const start = Number.isFinite(startMs) ? startMs : 0
  const end = Number.isFinite(endMs) ? endMs : start + 2000
  const duration = Math.max(0, end - start)

  if (duration === 0) {
    return words.map(word => ({
      ...word,
      start_ms: start,
    }))
  }

  const step = duration / words.length

  return words.map((word, index) => ({
    ...word,
    start_ms: Math.round(start + index * step),
  }))
}

/**
 * Get line end time (from line's own end_ms or fallback)
 * @param {Array} lines - All synced lines
 * @param {number} lineIndex - Current line index
 * @returns {number} End time in ms
 */
export function getLineEndTime(lines, lineIndex) {
  if (!Array.isArray(lines) || lineIndex < 0 || lineIndex >= lines.length) {
    return undefined
  }

  const line = lines[lineIndex]

  // Prefer the line's own end_ms
  if (Number.isFinite(line?.end_ms)) {
    return line.end_ms
  }

  // Fallback: line start + 2000ms
  if (Number.isFinite(line?.start_ms)) {
    return line.start_ms + 2000
  }

  return undefined
}

/**
 * Ensure line has valid words with timings
 * @param {Object} line - Line object
 * @param {Array} lines - All lines for context
 * @param {number} lineIndex - Current line index
 * @returns {Object} Line with validated words
 */
export function ensureLineWords(line, lines, lineIndex) {
  if (!line) return line

  // Check if existing words are valid
  if (hasValidWords(line)) {
    // Ensure all words have start_ms
    const endTime = getLineEndTime(lines, lineIndex)
    const startTime = line.start_ms || 0

    const hasTimings = line.words.every(w => Number.isFinite(w.start_ms))

    if (!hasTimings) {
      return {
        ...line,
        words: distributeWordTimings(line.words, startTime, endTime),
      }
    }

    return line
  }

  // Generate new words from text
  const words = generateWordsFromLine(line)
  const endTime = getLineEndTime(lines, lineIndex)
  const startTime = line.start_ms || 0

  return {
    ...line,
    words: distributeWordTimings(words, startTime, endTime),
  }
}
