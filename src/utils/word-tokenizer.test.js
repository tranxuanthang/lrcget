import { describe, it, expect } from 'vitest'
import {
  tokenizeText,
  generateWordsFromLine,
  distributeWordTimings,
  hasValidWords,
  ensureLineWords,
  getLineEndTime,
} from './word-tokenizer.js'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Assert tokenization produces the expected text tokens. */
function expectTokens(input, expectedTexts) {
  const result = tokenizeText(input).map(t => t.text)
  expect(result).toEqual(expectedTexts)
}

/** Assert joining tokens back together reconstructs the exact input. */
function expectReconstruction(input) {
  const result = tokenizeText(input)
    .map(t => t.text)
    .join('')
  expect(result).toBe(input)
}

// ---------------------------------------------------------------------------
// tokenizeText
// ---------------------------------------------------------------------------

describe('tokenizeText', () => {
  describe('edge cases', () => {
    it('returns empty array for empty string', () => {
      expectTokens('', [])
    })

    it('returns empty array for null', () => {
      expect(tokenizeText(null)).toEqual([])
    })

    it('returns empty array for undefined', () => {
      expect(tokenizeText(undefined)).toEqual([])
    })

    it('returns empty array for non-string', () => {
      expect(tokenizeText(123)).toEqual([])
    })

    it('handles single space', () => {
      expectTokens(' ', [' '])
    })

    it('handles multiple spaces', () => {
      expectTokens('   ', ['   '])
    })
  })

  describe('Latin text', () => {
    it('splits words by spaces', () => {
      expectTokens('hello world', ['hello ', 'world'])
    })

    it('preserves trailing space on last word', () => {
      expectTokens('hello world ', ['hello ', 'world '])
    })

    it('preserves leading space', () => {
      expectTokens(' hello', [' ', 'hello'])
    })

    it('handles multiple spaces between words', () => {
      expectTokens('hello  world', ['hello  ', 'world'])
    })

    it('handles single Latin word', () => {
      expectTokens('hello', ['hello'])
    })

    it('handles Latin with numbers', () => {
      expectTokens('track 1 of 10', ['track ', '1 ', 'of ', '10'])
    })

    it('handles Latin with punctuation', () => {
      expectTokens("don't stop", ["don't ", 'stop'])
    })
  })

  describe('CJK text — no spaces', () => {
    it('splits each character', () => {
      expectTokens('你好世界', ['你', '好', '世', '界'])
    })

    it('handles single CJK character', () => {
      expectTokens('你', ['你'])
    })

    it('handles Hiragana', () => {
      expectTokens('あいう', ['あ', 'い', 'う'])
    })

    it('handles Katakana', () => {
      expectTokens('アイウ', ['ア', 'イ', 'ウ'])
    })

    it('handles Hangul', () => {
      expectTokens('안녕하세요', ['안', '녕', '하', '세', '요'])
    })
  })

  describe('CJK text — spaces', () => {
    it('attaches space to preceding CJK token', () => {
      expectTokens('光 破', ['光 ', '破'])
    })

    it('handles multiple spaces between CJK', () => {
      expectTokens('光  破', ['光  ', '破'])
    })

    it('handles space at end of CJK text', () => {
      expectTokens('你好 ', ['你', '好 '])
    })

    it('handles real-world Chinese lyric line', () => {
      expectTokens('那起舞的光 破碎的浪', [
        '那', '起', '舞', '的', '光 ',
        '破', '碎', '的', '浪',
      ])
    })
  })

  describe('CJK punctuation — brackets', () => {
    it('groups fullwidth parentheses with content', () => {
      expectTokens('（啊）', ['（啊）'])
    })

    it('groups opening bracket with following CJK', () => {
      expectTokens('天之涯（啊）', ['天', '之', '涯', '（啊）'])
    })

    it('handles multiple bracket groups', () => {
      expectTokens('风起了（啊）雨又下（啊）', [
        '风', '起', '了', '（啊）',
        '雨', '又', '下', '（啊）',
      ])
    })

    it('handles nested bracket text', () => {
      expectTokens('天之涯（啊）心上花（啊）', [
        '天', '之', '涯', '（啊）',
        '心', '上', '花', '（啊）',
      ])
    })

    it('handles CJK angle brackets', () => {
      expectTokens('「你好」', ['「你', '好」'])
    })

    it('handles CJK square brackets', () => {
      expectTokens('【重要】', ['【重', '要】'])
    })
  })

  describe('CJK punctuation — sentence', () => {
    it('attaches comma to preceding CJK', () => {
      expectTokens('你好，', ['你', '好，'])
    })

    it('attaches period to preceding CJK', () => {
      expectTokens('世界。', ['世', '界。'])
    })

    it('handles comma and period in sentence', () => {
      expectTokens('你好，世界。', ['你', '好，', '世', '界。'])
    })

    it('handles enumeration comma', () => {
      expectTokens('天、地、人', ['天、', '地、', '人'])
    })

    it('handles exclamation and question', () => {
      expectTokens('什么！真的？', ['什', '么！', '真', '的？'])
    })

    it('handles colon and semicolon', () => {
      expectTokens('注：重要；', ['注：', '重', '要；'])
    })
  })

  describe('mixed CJK and Latin', () => {
    it('splits at script boundary (Latin then CJK)', () => {
      expectTokens('hello世界', ['hello', '世', '界'])
    })

    it('splits at script boundary (CJK then Latin)', () => {
      expectTokens('你好world', ['你', '好', 'world'])
    })

    it('handles Latin surrounded by CJK', () => {
      expectTokens('歌ABC曲', ['歌', 'ABC', '曲'])
    })

    it('handles space between CJK and Latin', () => {
      expectTokens('hello 世界', ['hello ', '世', '界'])
    })

    it('handles real-world mixed', () => {
      expectTokens('Verse 1: 你好', ['Verse ', '1: ', '你', '好'])
    })
  })

  describe('reconstruction invariant', () => {
    it('reconstructs simple Latin', () => {
      expectReconstruction('hello world')
    })

    it('reconstructs CJK with spaces', () => {
      expectReconstruction('那起舞的光 破碎的浪')
    })

    it('reconstructs CJK with brackets', () => {
      expectReconstruction('天之涯（啊）')
    })

    it('reconstructs CJK with sentence punctuation', () => {
      expectReconstruction('你好，世界。')
    })

    it('reconstructs mixed', () => {
      expectReconstruction('hello世界')
    })

    it('reconstructs complex real-world line', () => {
      expectReconstruction(
        '风起了（啊）雨又下（啊）你看着我不说话'
      )
    })

    it('reconstructs multiple spaces', () => {
      expectReconstruction('光  破')
    })

    it('reconstructs leading and trailing spaces', () => {
      expectReconstruction('  hello  ')
    })
  })

  describe('Japanese phonetic modifiers', () => {
    describe('Sokuon (small tsu) — merges forward', () => {
      it('merges sokuon with following katakana', () => {
        expectTokens('ポッキー', ['ポ', 'ッキー'])
      })

      it('merges sokuon with following hiragana', () => {
        expectTokens('って', ['って'])
      })

      it('merges sokuon in context', () => {
        expectTokens('人間って', ['人', '間', 'って'])
      })

      it('handles consecutive sokuon', () => {
        expectTokens('ッッキ', ['ッッキ'])
      })

      it('handles sokuon at end of text', () => {
        expectTokens('あっ', ['あっ'])
      })

      it('handles sokuon at start of text', () => {
        expectTokens('っあ', ['っあ'])
      })

      it('flushes sokuon before space', () => {
        // Space attaches to preceding CJK token, consistent with existing behavior
        expectTokens('っ ', ['っ '])
      })

      it('flushes sokuon before Latin', () => {
        expectTokens('っa', ['っ', 'a'])
      })

      it('flushes sokuon before CJK punctuation', () => {
        expectTokens('っ、', ['っ、'])
      })

      it('handles sokuon with opening bracket', () => {
        expectTokens('っ（あ）', ['っ', '（あ）'])
      })

      it('handles sokuon inside brackets', () => {
        expectTokens('（っあ）', ['（っあ）'])
      })
    })

    describe('Yōon (small kana) — merges backward', () => {
      it('merges yōon with preceding hiragana', () => {
        expectTokens('きゃ', ['きゃ'])
      })

      it('merges yōon with preceding katakana', () => {
        expectTokens('キャ', ['キャ'])
      })

      it('handles multiple yōon in sequence', () => {
        expectTokens('きょきゃ', ['きょ', 'きゃ'])
      })

      it('handles yōon at start of text', () => {
        expectTokens('ゃ', ['ゃ'])
      })

      it('does not merge yōon across Latin', () => {
        expectTokens('aゃ', ['a', 'ゃ'])
      })

      it('handles halfwidth yōon', () => {
        expectTokens('ｷｬ', ['ｷｬ'])
      })
    })

    describe('Chōonpu (long vowel) — merges backward', () => {
      it('merges chōonpu with preceding katakana', () => {
        expectTokens('ポー', ['ポー'])
      })

      it('handles chōonpu in context', () => {
        expectTokens('あのー', ['あ', 'のー'])
      })

      it('handles multiple chōonpu', () => {
        expectTokens('ポーー', ['ポーー'])
      })

      it('handles chōonpu at start of text', () => {
        expectTokens('ー', ['ー'])
      })

      it('handles halfwidth chōonpu', () => {
        expectTokens('ポｰ', ['ポｰ'])
      })
    })

    describe('Combined modifiers', () => {
      it('handles sokuon + base + chōonpu', () => {
        expectTokens('ポッキー', ['ポ', 'ッキー'])
      })

      it('handles sokuon + yōon', () => {
        expectTokens('キャッシャ', ['キャ', 'ッシャ'])
      })

      it('handles full line from user issue', () => {
        expectTokens('あのー、人間ってどう思いますか？', [
          'あ', 'のー、', '人', '間', 'って', 'ど', 'う', '思', 'い', 'ま', 'す', 'か？',
        ])
      })

      it('handles another full line from user issue', () => {
        expectTokens('東京特許きょきゃきょきゃ共感性羞恥', [
          '東', '京', '特', '許', 'きょ', 'きゃ', 'きょ', 'きゃ', '共', '感', '性', '羞', '恥',
        ])
      })

      it('handles sokuon with trailing punctuation', () => {
        expectTokens('ポッ、', ['ポ', 'ッ、'])
      })
    })

    describe('reconstruction invariant — Japanese', () => {
      it('reconstructs sokuon text', () => {
        expectReconstruction('ポッキー')
      })

      it('reconstructs yōon text', () => {
        expectReconstruction('きょきゃ')
      })

      it('reconstructs chōonpu text', () => {
        expectReconstruction('あのー')
      })

      it('reconstructs mixed modifiers', () => {
        expectReconstruction('キャッシャ')
      })

      it('reconstructs full Japanese sentence', () => {
        expectReconstruction('あのー、人間ってどう思いますか？')
      })

      it('reconstructs Japanese with spaces', () => {
        expectReconstruction('ポッキー 食べた')
      })
    })
  })
})

// ---------------------------------------------------------------------------
// generateWordsFromLine
// ---------------------------------------------------------------------------

describe('generateWordsFromLine', () => {
  it('returns empty array for null line', () => {
    expect(generateWordsFromLine(null)).toEqual([])
  })

  it('returns empty array for line without text', () => {
    expect(generateWordsFromLine({})).toEqual([])
  })

  it('generates words from CJK text', () => {
    const result = generateWordsFromLine({ text: '你好' })
    expect(result).toEqual([{ text: '你' }, { text: '好' }])
  })

  it('generates words from Latin text', () => {
    const result = generateWordsFromLine({ text: 'hello world' })
    expect(result).toEqual([{ text: 'hello ' }, { text: 'world' }])
  })
})

// ---------------------------------------------------------------------------
// distributeWordTimings
// ---------------------------------------------------------------------------

describe('distributeWordTimings', () => {
  it('returns empty array for empty words', () => {
    expect(distributeWordTimings([], 0, 1000)).toEqual([])
  })

  it('returns empty array for non-array', () => {
    expect(distributeWordTimings(null, 0, 1000)).toEqual([])
  })

  it('distributes evenly across duration', () => {
    const words = [{ text: 'a' }, { text: 'b' }]
    const result = distributeWordTimings(words, 0, 1000)
    expect(result).toEqual([
      { text: 'a', start_ms: 0 },
      { text: 'b', start_ms: 500 },
    ])
  })

  it('distributes three words', () => {
    const words = [{ text: 'a' }, { text: 'b' }, { text: 'c' }]
    const result = distributeWordTimings(words, 0, 900)
    expect(result).toEqual([
      { text: 'a', start_ms: 0 },
      { text: 'b', start_ms: 300 },
      { text: 'c', start_ms: 600 },
    ])
  })

  it('uses fallback end time when endMs is not finite', () => {
    const words = [{ text: 'a' }]
    const result = distributeWordTimings(words, 1000, NaN)
    expect(result).toEqual([{ text: 'a', start_ms: 1000 }])
  })

  it('uses start + 2000 fallback when both are missing', () => {
    const words = [{ text: 'a' }]
    const result = distributeWordTimings(words, NaN, NaN)
    expect(result).toEqual([{ text: 'a', start_ms: 0 }])
  })

  it('handles zero duration', () => {
    const words = [{ text: 'a' }, { text: 'b' }]
    const result = distributeWordTimings(words, 1000, 1000)
    expect(result).toEqual([
      { text: 'a', start_ms: 1000 },
      { text: 'b', start_ms: 1000 },
    ])
  })
})

// ---------------------------------------------------------------------------
// hasValidWords
// ---------------------------------------------------------------------------

describe('hasValidWords', () => {
  it('returns false for null line', () => {
    expect(hasValidWords(null)).toBe(false)
  })

  it('returns false for missing words array', () => {
    expect(hasValidWords({ text: 'hello' })).toBe(false)
  })

  it('returns false for empty words array', () => {
    expect(hasValidWords({ text: 'hello', words: [] })).toBe(false)
  })

  it('returns false when words do not reconstruct text', () => {
    expect(
      hasValidWords({ text: 'hello', words: [{ text: 'hi' }] })
    ).toBe(false)
  })

  it('returns true when words reconstruct text exactly', () => {
    expect(
      hasValidWords({
        text: 'hello world',
        words: [{ text: 'hello ' }, { text: 'world' }],
      })
    ).toBe(true)
  })

  it('returns true for CJK words', () => {
    expect(
      hasValidWords({
        text: '你好',
        words: [{ text: '你' }, { text: '好' }],
      })
    ).toBe(true)
  })
})

// ---------------------------------------------------------------------------
// getLineEndTime
// ---------------------------------------------------------------------------

describe('getLineEndTime', () => {
  const lines = [
    { start_ms: 1000, end_ms: 3000 },
    { start_ms: 3000 },
  ]

  it('returns line end_ms when available', () => {
    expect(getLineEndTime(lines, 0)).toBe(3000)
  })

  it('falls back to start_ms + 2000 when end_ms missing', () => {
    expect(getLineEndTime(lines, 1)).toBe(5000)
  })

  it('returns undefined for out of bounds index', () => {
    expect(getLineEndTime(lines, 99)).toBeUndefined()
  })

  it('returns undefined for negative index', () => {
    expect(getLineEndTime(lines, -1)).toBeUndefined()
  })

  it('returns undefined for non-array', () => {
    expect(getLineEndTime(null, 0)).toBeUndefined()
  })
})

// ---------------------------------------------------------------------------
// ensureLineWords
// ---------------------------------------------------------------------------

describe('ensureLineWords', () => {
  it('returns null for null line', () => {
    expect(ensureLineWords(null, [], 0)).toBeNull()
  })

  it('generates words when missing', () => {
    const line = { text: 'ab', start_ms: 0, end_ms: 1000 }
    const result = ensureLineWords(line, [line], 0)
    expect(result.words).toHaveLength(1)
    expect(result.words[0].text).toBe('ab')
    expect(result.words[0].start_ms).toBe(0)
  })

  it('distributes timings when words exist but lack start_ms', () => {
    const line = {
      text: 'a b',
      start_ms: 0,
      end_ms: 1000,
      words: [{ text: 'a ' }, { text: 'b' }],
    }
    const result = ensureLineWords(line, [line], 0)
    expect(result.words[0].start_ms).toBe(0)
    expect(result.words[1].start_ms).toBe(500)
  })

  it('keeps existing timings when all words have start_ms', () => {
    const line = {
      text: 'a b',
      start_ms: 0,
      end_ms: 1000,
      words: [
        { text: 'a ', start_ms: 100 },
        { text: 'b', start_ms: 500 },
      ],
    }
    const result = ensureLineWords(line, [line], 0)
    expect(result.words[0].start_ms).toBe(100)
    expect(result.words[1].start_ms).toBe(500)
  })

  it('regenerates words when existing words do not match text', () => {
    const line = {
      text: 'hello',
      start_ms: 0,
      end_ms: 1000,
      words: [{ text: 'old' }],
    }
    const result = ensureLineWords(line, [line], 0)
    expect(result.words).toHaveLength(1)
    expect(result.words[0].text).toBe('hello')
  })
})
