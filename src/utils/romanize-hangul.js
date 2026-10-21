import { romanize } from 'es-hangul'
import hanja from 'hanja'

const KOREAN_PATTERN = /[ㄱ-ㅎㅏ-ㅣ가-힣\u4E00-\u9FFF]/

const hanjaTranslate = hanja?.translate || hanja?.default?.translate

export const hasKorean = text => typeof text === 'string' && KOREAN_PATTERN.test(text)

// Same path as the synced-lyrics romanizer: hanja is read as Hangul, then romanized.
export const romanizeHangul = line => {
  if (typeof line !== 'string' || line.length === 0 || !hasKorean(line)) {
    return typeof line === 'string' ? line : ''
  }

  const hangul = typeof hanjaTranslate === 'function' ? hanjaTranslate(line, 'SUBSTITUTION') : line
  return romanize(hangul)
}

export const romanizedHangulLabel = text => {
  if (typeof text !== 'string') {
    return ''
  }

  const trimmed = text.trim()
  if (!trimmed || !hasKorean(trimmed)) {
    return ''
  }

  const romanized = romanizeHangul(trimmed).trim()
  if (!romanized || romanized === trimmed) {
    return ''
  }

  return romanized
}
