/**
 * Timing helper for splitting a word into two in the synced word-timing lane.
 *
 * The split boundary must land where the user saw the highlighted divider on
 * the lane, not on the word's end bound. Every non-final Latin token carries a
 * trailing space (see word-tokenizer's splitLatinText); a trailing space
 * renders with ~zero advance width, so the legacy text-width ratio collapses to
 * ~1.0 for the last grapheme boundary and pins the new segment onto the word's
 * end (a zero-length segment). Preferring the highlighted divider's actual lane
 * time avoids that entirely.
 */

/**
 * Resolve the boundary time (ms) at which to split a word in two.
 *
 * Prefers `highlightTimeMs` (the lane time under the highlighted divider) and
 * falls back to `ratioTimeMs` (derived from text width) when it is absent. The
 * result is clamped to the word's strict interior so neither resulting segment
 * is zero-length.
 *
 * @param {Object} params
 * @param {number} params.wordStartMs - Start time of the word being split.
 * @param {number} params.wordEndMs - End time of the word being split.
 * @param {number|null} [params.highlightTimeMs] - Lane time under the highlight.
 * @param {number|null} [params.ratioTimeMs] - Fallback time from the width ratio.
 * @returns {number} Split time, clamped to (wordStartMs, wordEndMs).
 */
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
