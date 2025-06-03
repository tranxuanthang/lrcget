import * as wanakana from 'wanakana';
import pinyin from 'pinyin';

// Basic language detection
function detectLanguage(text) {
  // Check for Japanese characters (Hiragana, Katakana, Kanji)
  // Hiragana: \u3040-\u309F
  // Katakana: \u30A0-\u30FF
  // Kanji (common range, not exhaustive): \u4E00-\u9FAF
  if (/[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FAF]/.test(text)) {
    // Further distinguish between Japanese and Chinese if needed,
    // For now, prioritizing Japanese if Kanji is present alongside Kana.
    // A more sophisticated approach might be needed for mixed scripts.
    if (/[\u3040-\u309F\u30A0-\u30FF]/.test(text)) {
        return 'ja';
    }
    // If only Kanji, it's harder to distinguish without more context or a library.
    // For this basic version, let's assume Kanji without Kana is more likely Chinese.
    // However, many Kanji are shared. This is a simplification.
    if (/[\u4E00-\u9FAF]/.test(text) && !/[\u3040-\u309F\u30A0-\u30FF]/.test(text)) {
        return 'zh';
    }
    return 'ja'; // Default to Japanese if Kana or mixed Kanji/Kana
  }

  // Simplified check for Chinese characters (common range)
  // This range overlaps significantly with Kanji, so the Japanese check should ideally run first.
  // This is a fallback if no Kana characters were found.
  if (/[\u4E00-\u9FAF]/.test(text)) {
    return 'zh';
  }
  return 'unknown';
}

function romanizeJapanese(text) {
  return wanakana.toRomaji(text);
}

function romanizeChinese(text) {
  const pinyinArray = pinyin(text, {
    style: pinyin.STYLE_NORMAL, // Get Pinyin without tone marks
  });
  // pinyin() returns an array of arrays, e.g., [['ni'], ['hao']]
  // We need to join them into a single string: "ni hao"
  return pinyinArray.map(arr => arr.join('')).join(' ');
}

function getRomanizedText(text, language) {
  if (language === 'ja') {
    return romanizeJapanese(text);
  } else if (language === 'zh') {
    return romanizeChinese(text);
  } else {
    return text;
  }
}

export { detectLanguage, romanizeJapanese, romanizeChinese, getRomanizedText };
