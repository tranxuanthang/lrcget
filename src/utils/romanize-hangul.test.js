import { describe, expect, it } from 'vitest'
import { romanizeHangul, romanizedHangulLabel } from './romanize-hangul.js'

describe('romanizeHangul', () => {
  it('romanizes a spaced Hangul line and keeps punctuation', () => {
    expect(romanizeHangul('뭘 좋아하는지? 나는 어떤지?')).toBe('mwol joahaneunji? naneun eotteonji?')
  })

  it('leaves text without Hangul unchanged', () => {
    expect(romanizeHangul('She killed my ego')).toBe('She killed my ego')
  })

  it('trims a word label and hides text that is already Latin', () => {
    expect(romanizedHangulLabel('뭘 ')).toBe('mwol')
    expect(romanizedHangulLabel('hello')).toBe('')
  })
})