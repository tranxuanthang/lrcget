export const isSynchronizedLyrics = (lyrics) => {
  // Match either timestamp tags [00:00.00] or metadata tags [xx:value]
  return /^\[(?:\d{2}:\d{2}[.:]\d{2,3}|[a-z]+:.+?)\]/.test(lyrics)
}
