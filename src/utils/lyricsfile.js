import YAML from 'yaml'

export const LYRICSFILE_VERSION = '1.0'
export const INSTRUMENTAL_LRC = '[au: instrumental]'

const emptyParsedLyricsfile = () => ({
  plainLyrics: '',
  syncedLines: [],
  isInstrumental: false,
  document: null,
})

const normalizeNonEmpty = value => {
  if (typeof value !== 'string') {
    return null
  }

  return value.trim().length > 0 ? value : null
}

const normalizeLrcText = value => {
  if (typeof value !== 'string') {
    return ''
  }

  return value.replace(/\r\n/g, '\n').trimEnd()
}

const stripTimestamp = lyrics => {
  if (typeof lyrics !== 'string') {
    return ''
  }

  return lyrics.replace(/^\[[^\]]+\] */gm, '')
}

const parseLrcTimestamp = token => {
  const match = token.match(/^(\d{2}):(\d{2})[.:](\d{2,3})$/)
  if (!match) {
    return null
  }

  const minutes = Number.parseInt(match[1], 10)
  const seconds = Number.parseInt(match[2], 10)
  const fractionRaw = match[3]
  const fractionMs =
    fractionRaw.length === 2
      ? Number.parseInt(fractionRaw, 10) * 10
      : Number.parseInt(fractionRaw, 10)

  return (minutes * 60 + seconds) * 1000 + fractionMs
}

const parseLrcLines = syncedLyrics => {
  const normalized = normalizeLrcText(syncedLyrics)
  if (!normalized) {
    return []
  }

  const parsed = normalized
    .split('\n')
    .map(line => {
      const match = line.match(/^\[([^\]]+)\](.*)$/)
      if (!match) {
        return null
      }

      const startMs = parseLrcTimestamp(match[1])
      if (startMs === null) {
        return null
      }

      return {
        text: match[2] || '',
        start_ms: startMs,
        words: [],
      }
    })
    .filter(line => line)

  return parsed.map((line, index) => ({
    ...line,
    end_ms: parsed[index + 1]?.start_ms,
  }))
}

const cloneWords = words => {
  if (!Array.isArray(words)) {
    return []
  }

  return words.map(word => ({
    text: typeof word?.text === 'string' ? word.text : '',
    start_ms: Number.isFinite(word?.start_ms) ? Math.max(0, Math.round(word.start_ms)) : undefined,
    end_ms: Number.isFinite(word?.end_ms) ? Math.max(0, Math.round(word.end_ms)) : undefined,
  }))
}

export const normalizeSyncedLine = (line, fallbackText = '') => {
  const words = cloneWords(line?.words)
  const textFromWords = words.map(word => word.text || '').join('')
  const text = typeof line?.text === 'string' ? line.text : textFromWords || fallbackText

  const normalizedLine = {
    text,
    words,
  }

  if (Number.isFinite(line?.start_ms)) {
    normalizedLine.start_ms = Math.max(0, Math.round(line.start_ms))
  }

  if (Number.isFinite(line?.end_ms)) {
    normalizedLine.end_ms = Math.max(0, Math.round(line.end_ms))
  }

  return normalizedLine
}

export const formatTimestampMs = timestampMs => {
  if (!Number.isFinite(timestampMs)) {
    return '--:--.---'
  }

  const safeMs = Math.max(0, Math.floor(timestampMs))
  const minutes = Math.floor(safeMs / 60000)
  const seconds = Math.floor((safeMs % 60000) / 1000)
  const milliseconds = safeMs % 1000

  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}.${String(milliseconds).padStart(3, '0')}`
}

export const createSyncedLinesFromPlain = (plainLyrics, existingLines = []) => {
  const plainLines =
    typeof plainLyrics === 'string' ? plainLyrics.replace(/\r\n/g, '\n').split('\n') : []

  return plainLines.map((text, index) => {
    const existingLine = existingLines[index]
    return normalizeSyncedLine(existingLine, text)
  })
}

const cloneSyncedLines = lines => {
  if (!Array.isArray(lines)) {
    return []
  }

  return lines.map(line => normalizeSyncedLine(line))
}

const formatLrcTimestamp = timestampMs => {
  const safeMs = Math.max(0, Math.floor(timestampMs || 0))
  const minutes = Math.floor(safeMs / 60000)
  const seconds = Math.floor((safeMs % 60000) / 1000)
  const milliseconds = safeMs % 1000

  return `[${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}.${String(milliseconds).padStart(3, '0')}]`
}

const linesToLrc = lines => {
  if (!Array.isArray(lines) || lines.length === 0) {
    return ''
  }

  const output = lines
    .map(line => {
      const text =
        Array.isArray(line.words) && line.words.length > 0
          ? line.words.map(word => word.text || '').join('')
          : line.text || ''

      return `${formatLrcTimestamp(line.start_ms)}${text}`
    })
    .join('\n')

  return normalizeNonEmpty(output) || ''
}

const isInstrumentalLyrics = lyrics => {
  const lowered = (lyrics || '').toLowerCase()
  return lowered.includes('[au:') && lowered.includes('instrumental')
}

const durationToMs = duration => {
  if (typeof duration === 'number' && duration > 0) {
    return Math.round(duration * 1000)
  }

  return undefined
}

export const parseLyricsfile = lyricsfileContent => {
  const normalizedLyricsfile = normalizeNonEmpty(lyricsfileContent)
  if (!normalizedLyricsfile) {
    return emptyParsedLyricsfile()
  }

  let document = {}

  try {
    document = YAML.parse(normalizedLyricsfile) || {}
  } catch {
    return emptyParsedLyricsfile()
  }

  const metadata = document.metadata || {}
  const lines = Array.isArray(document.lines) ? document.lines : []
  const isInstrumental = Boolean(metadata.instrumental)
  const syncedLyrics = isInstrumental ? INSTRUMENTAL_LRC : linesToLrc(lines)
  const plainLyrics =
    normalizeNonEmpty(document.plain) || normalizeNonEmpty(stripTimestamp(syncedLyrics)) || ''

  return {
    plainLyrics,
    syncedLyrics,
    syncedLines: cloneSyncedLines(lines),
    isInstrumental,
    document,
  }
}

export const normalizeLrclibLyrics = item => {
  const plainLyrics = normalizeNonEmpty(item?.plainLyrics)
  const syncedLyrics = normalizeNonEmpty(item?.syncedLyrics)
  const instrumental = Boolean(item?.instrumental)
  const lyricsfile = normalizeNonEmpty(item?.lyricsfile)

  if (!lyricsfile) {
    return {
      plainLyrics: plainLyrics || '',
      syncedLyrics: syncedLyrics || '',
      instrumental,
      hasLyricsfile: false,
    }
  }

  const parsed = parseLyricsfile(lyricsfile)
  const fallbackPlainLyrics = normalizeNonEmpty(parsed.plainLyrics) || ''
  const fallbackSyncedLyrics = normalizeNonEmpty(parsed.syncedLyrics) || ''

  return {
    plainLyrics: plainLyrics || fallbackPlainLyrics,
    syncedLyrics: syncedLyrics || fallbackSyncedLyrics,
    instrumental: instrumental || Boolean(parsed.isInstrumental),
    hasLyricsfile: true,
  }
}

export const serializeLyricsfile = ({
  track,
  plainLyrics,
  syncedLines,
  syncedLyrics,
  baseDocument,
  isInstrumental: forceInstrumental,
}) => {
  const normalizedPlain = normalizeNonEmpty(plainLyrics)
  const normalizedSynced = normalizeNonEmpty(syncedLyrics)
  const normalizedSyncedLines = cloneSyncedLines(syncedLines)

  if (
    !normalizedPlain &&
    normalizedSyncedLines.length === 0 &&
    !normalizedSynced &&
    !forceInstrumental
  ) {
    return null
  }

  const isInstrumental =
    forceInstrumental || (normalizedSynced ? isInstrumentalLyrics(normalizedSynced) : false)
  const baseMetadata = baseDocument?.metadata || {}
  const baseLines = Array.isArray(baseDocument?.lines) ? baseDocument.lines : []
  const baseLrc = linesToLrc(baseLines)

  let lines = normalizedSyncedLines

  if (!isInstrumental) {
    if (
      lines.length === 0 &&
      normalizedSynced &&
      normalizeLrcText(baseLrc) === normalizeLrcText(normalizedSynced)
    ) {
      lines = cloneSyncedLines(baseLines)
    } else if (lines.length === 0 && normalizedSynced) {
      lines = parseLrcLines(normalizedSynced)
    }
  }

  const plain = isInstrumental
    ? undefined
    : normalizedPlain || normalizeNonEmpty(stripTimestamp(normalizedSynced || '')) || undefined

  const metadata = {
    title: track?.title || baseMetadata.title || '',
    artist: track?.artist_name || baseMetadata.artist || '',
    instrumental: isInstrumental,
  }

  const album = normalizeNonEmpty(track?.album_name) || normalizeNonEmpty(baseMetadata.album)
  if (album) {
    metadata.album = album
  }

  const durationMs = durationToMs(track?.duration) ?? baseMetadata.duration_ms
  if (durationMs) {
    metadata.duration_ms = durationMs
  }

  if (typeof baseMetadata.offset_ms === 'number') {
    metadata.offset_ms = baseMetadata.offset_ms
  }

  if (normalizeNonEmpty(baseMetadata.language)) {
    metadata.language = baseMetadata.language
  }

  return YAML.stringify({
    version: LYRICSFILE_VERSION,
    metadata,
    lines,
    plain,
  })
}
