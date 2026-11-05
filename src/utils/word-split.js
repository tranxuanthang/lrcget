// Split time: the highlighted divider's lane time, else the text-width ratio time,
// clamped inside the word so neither part is empty.
export function resolveWordSplitTimeMs({
  wordStartMs,
  wordEndMs,
  highlightTimeMs = null,
  ratioTimeMs = null,
}) {
  let candidate
  if (Number.isFinite(highlightTimeMs)) {
    candidate = highlightTimeMs
  } else if (Number.isFinite(ratioTimeMs)) {
    candidate = ratioTimeMs
  } else {
    candidate = Math.round((wordStartMs + wordEndMs) / 2)
  }

  return Math.max(wordStartMs + 1, Math.min(wordEndMs - 1, candidate))
}
