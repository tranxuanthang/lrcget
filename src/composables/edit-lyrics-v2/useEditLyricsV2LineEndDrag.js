import { ref } from 'vue'

const DRAG_THRESHOLD = 3

// Drag handler for the last word's right edge (the line end). Mirrors the
// pointer/threshold plumbing of useEditLyricsV2WordBoundaryDrag, but instead of
// moving a word-start boundary it trims the line window from the right: the
// commit moves the line end (and, via setLineEndMs upstream, the last word's
// explicit end) together. A short press without crossing DRAG_THRESHOLD is left
// for the caller's click handler (which nudges/expands instead of trimming).
// Committing deliberately leaves playback alone, unlike the word-boundary
// drag's replay: this edits the line end, so seeking back to the line start
// would be incoherent, and the row-level end nudge is playback-neutral too.
export function useEditLyricsV2LineEndDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  lineEndMs,
  selectedLineIndex,
  onCommitLineEnd,
}) {
  const endDragState = ref(null)
  const isDraggingEnd = ref(false)
  const dragStartPos = ref(null)

  // The end can be trimmed inward from the current line end down to just past
  // the last word's start. Expanding past the current end is the click's job,
  // so the max stays pinned at the pre-drag line end.
  const getEndConstraint = () => {
    const currentWords = words.value
    const lastWord = currentWords.length > 0 ? currentWords[currentWords.length - 1] : null
    const lastStartMs = Number.isFinite(lastWord?.start_ms) ? lastWord.start_ms : lineStartMs.value
    const minEndMs = (Number.isFinite(lastStartMs) ? lastStartMs : 0) + 1
    const maxEndMs = Number.isFinite(lineEndMs.value) ? lineEndMs.value : minEndMs

    return {
      minEndMs,
      maxEndMs: Math.max(minEndMs, maxEndMs),
    }
  }

  const stopEndDrag = () => {
    document.removeEventListener('pointermove', handlePointerMove)
    document.removeEventListener('pointerup', handlePointerUp)
    document.removeEventListener('pointercancel', handlePointerUp)
    document.removeEventListener('pointermove', handlePotentialDragStart)
    document.removeEventListener('pointerup', handlePotentialDragEnd)
    document.removeEventListener('pointercancel', handlePotentialDragEnd)
  }

  const updateDragPosition = (clientX, clientXToTime) => {
    if (!endDragState.value) {
      return
    }

    const { minEndMs, maxEndMs } = getEndConstraint()
    const nextEndMs = clientXToTime(clientX)
    endDragState.value = {
      ...endDragState.value,
      currentEndMs: Math.max(minEndMs, Math.min(maxEndMs, nextEndMs)),
    }
  }

  const commitEnd = ({ endMs, initialEndMs }) => {
    if (endMs === initialEndMs) {
      return false
    }

    onCommitLineEnd({
      lineIndex: selectedLineIndex.value,
      endMs,
    })

    return true
  }

  const handlePointerMove = event => {
    isDraggingEnd.value = true
    updateDragPosition(event.clientX, dragStartPos.value.clientXToTime)
  }

  const handlePointerUp = () => {
    if (endDragState.value) {
      commitEnd({
        endMs: endDragState.value.currentEndMs,
        initialEndMs: endDragState.value.initialEndMs,
      })
    }

    setTimeout(() => {
      isDraggingEnd.value = false
    }, 0)
    endDragState.value = null
    dragStartPos.value = null
    stopEndDrag()
  }

  const handlePotentialDragStart = event => {
    if (!dragStartPos.value) {
      return
    }

    const dx = Math.abs(event.clientX - dragStartPos.value.x)
    const dy = Math.abs(event.clientY - dragStartPos.value.y)

    if (dx <= DRAG_THRESHOLD && dy <= DRAG_THRESHOLD) {
      return
    }

    isDraggingEnd.value = true
    endDragState.value = {
      initialEndMs: dragStartPos.value.initialEndMs,
      currentEndMs: dragStartPos.value.initialEndMs,
    }

    const clientXToTime = dragStartPos.value.clientXToTime
    dragStartPos.value = {
      clientXToTime,
    }

    document.removeEventListener('pointermove', handlePotentialDragStart)
    document.removeEventListener('pointerup', handlePotentialDragEnd)
    document.removeEventListener('pointercancel', handlePotentialDragEnd)
    document.addEventListener('pointermove', handlePointerMove)
    document.addEventListener('pointerup', handlePointerUp)
    document.addEventListener('pointercancel', handlePointerUp)

    updateDragPosition(event.clientX, clientXToTime)
  }

  const handlePotentialDragEnd = () => {
    dragStartPos.value = null
    document.removeEventListener('pointermove', handlePotentialDragStart)
    document.removeEventListener('pointerup', handlePotentialDragEnd)
    document.removeEventListener('pointercancel', handlePotentialDragEnd)
  }

  const startEndDrag = (event, clientXToTime) => {
    event.preventDefault()
    event.stopPropagation()

    if (!isWordSyncAvailable.value) {
      return
    }

    const initialEndMs = Number.isFinite(lineEndMs.value) ? lineEndMs.value : 0

    isDraggingEnd.value = false
    dragStartPos.value = {
      x: event.clientX,
      y: event.clientY,
      initialEndMs,
      clientXToTime,
    }

    document.addEventListener('pointermove', handlePotentialDragStart)
    document.addEventListener('pointerup', handlePotentialDragEnd)
    document.addEventListener('pointercancel', handlePotentialDragEnd)
  }

  const cancelEndDrag = () => {
    endDragState.value = null
    dragStartPos.value = null
    isDraggingEnd.value = false
    stopEndDrag()
  }

  return {
    endDragState,
    isDraggingEnd,
    startEndDrag,
    cancelEndDrag,
    stopEndDrag,
  }
}
