export const isSynchronizedLyrics = (lyrics) => {
  return /^\[\d{2}:\d{2}[.:]\d{2,3}\]/.test(lyrics)
}
