/**
 * Word tokenization utilities for lyrics text.
 * Handles both Latin scripts (space-delimited) and CJK (character-based).
 */

// CJK Unicode ranges
const CJK_RANGES = [
  [0x4e00, 0x9fff], // CJK Unified Ideographs
  [0x3040, 0x309f], // Hiragana
  [0x30a0, 0x30ff], // Katakana
  [0xac00, 0xd7af], // Hangul Syllables
  [0x3400, 0x4dbf], // CJK Extension A
  [0xf900, 0xfaff], // CJK Compatibility Ideographs
  [0xff66, 0xff9f], // Halfwidth Katakana
  [0x31f0, 0x31ff], // Katakana Phonetic Extensions
  [0x3000, 0x303f], // CJK Symbols and Punctuation
]

/**
 * Check if a character is a CJK character
 * @param {string} char - Single character
 * @returns {boolean}
 */
function isCJKChar(char) {
  if (!char || char.length !== 1) return false
  const code = char.charCodeAt(0)
  return CJK_RANGES.some(([start, end]) => code >= start && code <= end)
}

/**
 * Tokenize text into words.
 * - Latin scripts: space-delimited, preserve trailing spaces in tokens
 * - CJK: each character is a separate token
 * @param {string} text - Input text to tokenize
 * @returns {Array<{text: string, isCJK: boolean}>} Array of word tokens
 */
export function tokenizeText(text) {
  if (!text || typeof text !== 'string') {
    return []
  }

  const tokens = []
  let currentToken = ''
  let currentIsCJK = null

  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    const charIsCJK = isCJKChar(char)

    // Start of a new token
    if (currentIsCJK === null) {
      currentToken = char
      currentIsCJK = charIsCJK
    } else if (charIsCJK === currentIsCJK) {
      // Continue current token
      if (charIsCJK) {
        // CJK: each character is its own token
        tokens.push({ text: currentToken, isCJK: true })
        currentToken = char
      } else {
        // Latin: accumulate characters
        currentToken += char
      }
    } else {
      // Switch between CJK and Latin
      if (currentToken) {
        tokens.push({ text: currentToken, isCJK: currentIsCJK })
      }
      currentToken = char
      currentIsCJK = charIsCJK
    }
  }

  // Push the last token
  if (currentToken) {
    tokens.push({ text: currentToken, isCJK: currentIsCJK })
  }

  // For Latin tokens, split by spaces while preserving trailing spaces
  const finalTokens = []
  for (const token of tokens) {
    if (token.isCJK) {
      finalTokens.push(token)
    } else {
      // Split Latin text by spaces
      const latinTokens = splitLatinText(token.text)
      for (const lt of latinTokens) {
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
