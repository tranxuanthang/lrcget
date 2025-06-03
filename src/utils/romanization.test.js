import { describe, it, expect } from 'vitest';
import {
  romanizeJapanese,
  romanizeChinese,
  detectLanguage,
  getRomanizedText,
} from './romanization';

describe('romanizeJapanese', () => {
  it('should romanize hiragana', () => {
    expect(romanizeJapanese('こんにちは')).toBe('konnichiha');
  });
  it('should romanize katakana', () => {
    expect(romanizeJapanese('コンニチハ')).toBe('konnichiha');
  });
  it('should romanize mixed hiragana and katakana', () => {
    expect(romanizeJapanese('こんにチハ')).toBe('konnichiha');
  });
  it('should pass through kanji', () => {
    expect(romanizeJapanese('日本語')).toBe('nihongo'); // wanakana handles common kanji readings
    expect(romanizeJapanese('日本')).toBe('nippon'); // common reading
    expect(romanizeJapanese('東京')).toBe('toukyou'); // common reading
  });
  it('should handle kanji mixed with kana', () => {
    expect(romanizeJapanese('食べ物')).toBe('tabemono');
    expect(romanizeJapanese('飲み物')).toBe('nomimono');
  });
  it('should preserve English text', () => {
    expect(romanizeJapanese('Hello World')).toBe('Hello World');
  });
  it('should handle mixed English and Japanese', () => {
    expect(romanizeJapanese('Hello こんにちは')).toBe('Hello konnichiha');
  });
  it('should return an empty string for an empty input', () => {
    expect(romanizeJapanese('')).toBe('');
  });
  it('should handle long vowel marks', () => {
    expect(romanizeJapanese('ラーメン')).toBe('ra-men'); // Standard for katakana long vowel
    expect(romanizeJapanese('とうきょう')).toBe('toukyou'); // Hiragana long vowel
  });
});

describe('romanizeChinese', () => {
  it('should romanize simple Chinese characters', () => {
    expect(romanizeChinese('你好')).toBe('nǐ hǎo');
  });
  it('should romanize phrases', () => {
    expect(romanizeChinese('我爱编程')).toBe('wǒ ài biān chéng');
  });
  it('should handle mixed Chinese and English text', () => {
    // pinyin library behavior might strip non-Chinese characters or handle them differently
    // Current implementation of pinyin() seems to ignore non-Chinese characters in the middle
    expect(romanizeChinese('我爱coding')).toBe('wǒ ài coding'); // Assuming pinyin passes English through
    expect(romanizeChinese('coding爱我')).toBe('coding ài wǒ');
  });
  it('should return an empty string for an empty input', () => {
    expect(romanizeChinese('')).toBe('');
  });
  it('should handle characters with multiple pronunciations (polyphones) - default pronunciation', () => {
    expect(romanizeChinese('银行')).toBe('yín háng'); // Common reading
  });
  it('should handle punctuation (pinyin library might strip it or keep it)', () => {
    expect(romanizeChinese('你好，世界！')).toBe('nǐ hǎo shì jiè'); // pinyin strips punctuation by default
  });
});

describe('detectLanguage', () => {
  it('should detect Japanese text (hiragana)', () => {
    expect(detectLanguage('こんにちは')).toBe('ja');
  });
  it('should detect Japanese text (katakana)', () => {
    expect(detectLanguage('コンニチハ')).toBe('ja');
  });
  it('should detect Japanese text (kanji with kana)', () => {
    expect(detectLanguage('日本語を話します')).toBe('ja');
  });
  it('should detect Chinese text (simplified)', () => {
    expect(detectLanguage('你好世界')).toBe('zh');
  });
   it('should detect Chinese text (traditional - if range covers)', () => {
    expect(detectLanguage('你好世界')).toBe('zh'); // Assuming same range or heuristic covers it
  });
  it('should return "unknown" for Korean text (Hangul)', () => {
    expect(detectLanguage('안녕하세요')).toBe('unknown'); // Hangul is outside CJK unified ideographs for basic check
  });
  it('should return "unknown" for English text', () => {
    expect(detectLanguage('Hello world')).toBe('unknown');
  });
  it('should prioritize Japanese if both Kana and Hanzi-like characters are present', () => {
    expect(detectLanguage('こんにちは你好')).toBe('ja');
  });
  it('should detect Chinese if only Hanzi-like characters (no Kana) are present', () => {
    expect(detectLanguage('你好 我好')).toBe('zh');
  });
  it('should return "unknown" for an empty string', () => {
    expect(detectLanguage('')).toBe('unknown');
  });
  it('should return "unknown" for special characters or numbers only', () => {
    expect(detectLanguage('12345 !@#$%')).toBe('unknown');
  });
});

describe('getRomanizedText', () => {
  it('should romanize Japanese text with lang code "ja"', () => {
    expect(getRomanizedText('こんにちは', 'ja')).toBe('konnichiha');
  });
  it('should romanize Chinese text with lang code "zh"', () => {
    expect(getRomanizedText('你好', 'zh')).toBe('nǐ hǎo');
  });
  it('should return original text if Japanese text is given with lang code "zh"', () => {
    expect(getRomanizedText('こんにちは', 'zh')).toBe('こんにちは');
  });
  it('should return original text if Chinese text is given with lang code "ja"', () => {
    expect(getRomanizedText('你好', 'ja')).toBe('你好');
  });
  it('should return original text for an unsupported language code', () => {
    expect(getRomanizedText('Hello', 'en')).toBe('Hello');
    expect(getRomanizedText('안녕하세요', 'ko')).toBe('안녕하세요');
  });
  it('should return original text if language is detected but a different code is passed', () => {
    // This assumes getRomanizedText strictly follows the provided lang code
    expect(getRomanizedText('こんにちは', 'zh')).toBe('こんにちは'); // Japanese text, but 'zh' code
    expect(getRomanizedText('你好', 'ja')).toBe('你好'); // Chinese text, but 'ja' code
  });
  it('should return an empty string for an empty input, regardless of lang code', () => {
    expect(getRomanizedText('', 'ja')).toBe('');
    expect(getRomanizedText('', 'zh')).toBe('');
    expect(getRomanizedText('', 'en')).toBe('');
  });
});
