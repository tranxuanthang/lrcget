import { computed, ref } from 'vue'

const DRAG_THRESHOLD = 3
const DRAG_REPLAY_SEPARATORS_BACK = 3
const DRAG_REPLAY_MAX_LOOKBACK_MS = 3000

export function useEditLyricsV2WordBoundaryDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  lineEndMs,
  timelineStartMs,
  timelineEndMs,
  selectedLineIndex,
  hasPersistedWords,
  onUpdateWords,
  onWordTimingEdited,
}) {
  const dragState = ref(null)
  const selectedBoundaryIndex = ref(0)
  const selectedBoundaryIndices = ref([0])
  const isDraggingBoundary = ref(false)
  const dragStartPos = ref(null)
  const boundaryDragEndedAt = ref(0)

  const getLineEndBoundaryIndex = () => (words.value.length > 0 ? words.value.length : -1)

  const isLineEndBoundaryIndex = index => index === getLineEndBoundaryIndex()

  const getLineEndBoundaryTimeMs = () => {
    const lastWord = words.value[words.value.length - 1]
    if (Number.isFinite(lastWord?.end_ms)) {
      return lastWord.end_ms
    }

    if (Number.isFinite(lineEndMs.value)) {
      return lineEndMs.value
    }

    if (Number.isFinite(timelineEndMs.value)) {
      return timelineEndMs.value
    }

    return null
  }

  const displayedWords = computed(() => {
    const currentWords = words.value

    if (!dragState.value) {
      return currentWords
    }

    if (dragState.value.rightWordIndex === currentWords.length && currentWords.length > 0) {
      return currentWords.map((word, index) => {
        if (index !== currentWords.length - 1) {
          return word
        }

        return {
          ...word,
          end_ms: dragState.value.currentStartMs,
        }
      })
    }

    return currentWords.map((word, index) => {
      if (index !== dragState.value.rightWordIndex) {
        return word
      }

      return {
        ...word,
        start_ms: dragState.value.currentStartMs,
      }
    })
  })

  const boundaryIndexes = computed(() => {
    if (displayedWords.value.length === 0) {
      return []
    }

    return Array.from({ length: displayedWords.value.length + 1 }, (_, index) => index)
  })

  const getBoundaryTimeMs = index => {
    const currentWords = displayedWords.value
    if (index < 0 || currentWords.length === 0) {
      return null
    }

    if (index < currentWords.length) {
      return currentWords[index]?.start_ms ?? null
    }

    if (index !== currentWords.length) {
      return null
    }

    if (dragState.value?.rightWordIndex === index) {
      return dragState.value.currentStartMs
    }

    return getLineEndBoundaryTimeMs()
  }

  const buildUpdatedWords = (rightWordIndex, startMs) => {
    return words.value.map((word, index) => {
      if (index === rightWordIndex) {
        return {
          ...word,
          start_ms: startMs,
        }
      }

      // Chain rule: boundary shared with previous word. Current word's own
      // end_ms is intentionally not updated; overshoot surfaces a warning.
      if (index === rightWordIndex - 1) {
        return {
          ...word,
          end_ms: startMs,
        }
      }

      return word
    })
  }

  const getBoundaryConstraint = rightWordIndex => {
    const currentWords = words.value

    if (isLineEndBoundaryIndex(rightWordIndex)) {
      const lastWord = currentWords[currentWords.length - 1]
      const minStartMs = Number.isFinite(lastWord?.start_ms) ? lastWord.start_ms + 1 : 0
      const paddedEndMs = Number.isFinite(timelineEndMs.value) ? timelineEndMs.value : Number.MAX_SAFE_INTEGER

      return {
        minStartMs,
        maxStartMs: paddedEndMs,
      }
    }

    const nextStartMs = currentWords[rightWordIndex + 1]?.start_ms
    const lineEndBoundaryMs = getLineEndBoundaryTimeMs()
    const maxAtLineEnd = Number.isFinite(lineEndBoundaryMs) ? lineEndBoundaryMs - 1 : timelineEndMs.value - 1

    if (rightWordIndex === 0) {
      const minStartMs = Number.isFinite(timelineStartMs.value) ? timelineStartMs.value : 0
      const maxStartMs = Number.isFinite(nextStartMs) ? nextStartMs - 1 : maxAtLineEnd

      return {
        minStartMs,
        maxStartMs: Math.max(minStartMs, maxStartMs),
      }
    }

    const previousStartMs = currentWords[rightWordIndex - 1]?.start_ms ?? timelineStartMs.value
    const minStartMs = previousStartMs + 1
    const maxStartMs = Number.isFinite(nextStartMs) ? nextStartMs - 1 : maxAtLineEnd

    return {
      minStartMs,
      maxStartMs: Math.max(minStartMs, maxStartMs),
    }
  }

  const stopBoundaryDrag = () => {
    document.removeEventListener('pointermove', handlePointerMove)
    document.removeEventListener('pointerup', handlePointerUp)
    document.removeEventListener('pointercancel', handlePointerUp)
    document.removeEventListener('pointermove', handlePotentialDragStart)
    document.removeEventListener('pointerup', handlePotentialDragEnd)
    document.removeEventListener('pointercancel', handlePotentialDragEnd)
  }

  const updateDragPosition = (clientX, clientXToTime) => {
    if (!dragState.value) {
      return
    }

    const { minStartMs, maxStartMs } = getBoundaryConstraint(dragState.value.rightWordIndex)
    const nextStartMs = clientXToTime(clientX)
    dragState.value = {
      ...dragState.value,
      currentStartMs: Math.max(minStartMs, Math.min(maxStartMs, nextStartMs)),
    }
  }

  const getStoredBoundaryTimeMs = index => {
    if (index <= 0) {
      const firstStartMs = words.value[0]?.start_ms
      return Number.isFinite(firstStartMs) ? firstStartMs : lineStartMs.value
    }

    if (index >= words.value.length) {
      return getLineEndBoundaryTimeMs()
    }

    return words.value[index]?.start_ms
  }

  // Only the last separator replays: three separators back, and never more
  // than 3 seconds before the dragged time.
  const getDragReplayStartMs = (rightWordIndex, draggedMs) => {
    const previousIndex = Math.max(0, rightWordIndex - DRAG_REPLAY_SEPARATORS_BACK)
    const previousMs =
      previousIndex === rightWordIndex ? draggedMs : getStoredBoundaryTimeMs(previousIndex)
    const boundedPreviousMs = Number.isFinite(previousMs) ? previousMs : draggedMs
    const earliestMs = draggedMs - DRAG_REPLAY_MAX_LOOKBACK_MS

    return Math.max(0, Math.min(draggedMs, Math.max(boundedPreviousMs, earliestMs)))
  }

  const getSeparatorPlayStartMs = index => {
    const boundaryMs = getBoundaryTimeMs(index)
    if (!isLineEndBoundaryIndex(index) || !Number.isFinite(boundaryMs)) {
      return boundaryMs
    }

    return getDragReplayStartMs(index, boundaryMs)
  }

  const commitLineEndBoundary = ({ startMs, shouldReplay, lineBoundsOnly = false }) => {
    // Read the lookback before the line end changes, so preview words are not
    // redistributed across the new window before the replay point is chosen.
    const replayStartMs = shouldReplay ? getDragReplayStartMs(words.value.length, startMs) : null

    if (lineBoundsOnly) {
      onUpdateWords({
        lineIndex: selectedLineIndex.value,
        preserveWords: true,
        lineEndMs: startMs,
      })
    } else {
      const lastIndex = words.value.length - 1
      const updatedWords = words.value.map((word, index) => {
        if (index !== lastIndex) {
          return word
        }

        return {
          ...word,
          end_ms: startMs,
        }
      })

      onUpdateWords({
        lineIndex: selectedLineIndex.value,
        words: updatedWords,
        lineEndMs: startMs,
      })
    }

    if (shouldReplay) {
      onWordTimingEdited({
        lineIndex: selectedLineIndex.value,
        startMs: replayStartMs,
      })
    }

    return true
  }

  const commitBoundary = ({
    rightWordIndex,
    startMs,
    initialStartMs,
    shouldReplay,
    lineBoundsOnly = false,
  }) => {
    if (startMs === initialStartMs) {
      return false
    }

    if (isLineEndBoundaryIndex(rightWordIndex)) {
      return commitLineEndBoundary({ startMs, shouldReplay, lineBoundsOnly })
    }

    if (lineBoundsOnly && rightWordIndex === 0) {
      onUpdateWords({
        lineIndex: selectedLineIndex.value,
        preserveWords: true,
        lineStartMs: startMs,
      })

      if (shouldReplay) {
        onWordTimingEdited({
          lineIndex: selectedLineIndex.value,
          startMs,
        })
      }

      return true
    }

    const updatedWords = buildUpdatedWords(rightWordIndex, startMs)

    onUpdateWords({
      lineIndex: selectedLineIndex.value,
      words: updatedWords,
      lineStartMs: rightWordIndex === 0 ? startMs : undefined,
    })

    if (shouldReplay) {
      onWordTimingEdited({
        lineIndex: selectedLineIndex.value,
        startMs,
      })
    }

    return true
  }

  const shiftSelectedBoundaryBy = offsetMs => {
    if (!isWordSyncAvailable.value || !Number.isFinite(offsetMs) || words.value.length === 0) {
      return null
    }

    const rightWordIndex = selectedBoundaryIndex.value
    const lineEndBoundaryIndex = getLineEndBoundaryIndex()
    if (rightWordIndex < 0 || rightWordIndex > lineEndBoundaryIndex) {
      return null
    }

    if (isLineEndBoundaryIndex(rightWordIndex)) {
      const lastWord = words.value[words.value.length - 1]
      const currentEndMs = getLineEndBoundaryTimeMs()
      if (!Number.isFinite(currentEndMs)) {
        return null
      }

      const minEndMs = Number.isFinite(lastWord?.start_ms) ? lastWord.start_ms + 1 : 0
      const nextEndMs = Math.max(minEndMs, Math.round(currentEndMs + offsetMs))

      commitBoundary({
        rightWordIndex,
        startMs: nextEndMs,
        initialStartMs: currentEndMs,
        shouldReplay: false,
      })

      return nextEndMs
    }

    const currentStartMs = words.value[rightWordIndex]?.start_ms
    if (!Number.isFinite(currentStartMs)) {
      return null
    }

    let minStartMs
    let maxStartMs

    if (rightWordIndex === 0) {
      const nextStartMs = words.value[1]?.start_ms
      const lineEndBoundaryMs = getLineEndBoundaryTimeMs()
      minStartMs = 0
      maxStartMs = Number.isFinite(nextStartMs)
        ? nextStartMs - 1
        : Number.isFinite(lineEndBoundaryMs)
          ? lineEndBoundaryMs - 1
          : timelineEndMs.value - 1
      maxStartMs = Math.max(minStartMs, maxStartMs)
    } else {
      const constraint = getBoundaryConstraint(rightWordIndex)
      minStartMs = constraint.minStartMs
      maxStartMs = constraint.maxStartMs
    }

    const nextStartMs = Math.max(minStartMs, Math.min(maxStartMs, Math.round(currentStartMs + offsetMs)))

    commitBoundary({
      rightWordIndex,
      startMs: nextStartMs,
      initialStartMs: currentStartMs,
      shouldReplay: false,
    })

    return nextStartMs
  }

  const normalizeSelection = indices => {
    const maxIndex = words.value.length
    const uniqueSorted = [...new Set(indices.filter(Number.isInteger))]
      .filter(index => index >= 0 && index <= maxIndex)
      .sort((a, b) => a - b)

    if (uniqueSorted.length === 0 && words.value.length > 0) {
      return [Math.min(Math.max(selectedBoundaryIndex.value, 0), maxIndex)]
    }

    return uniqueSorted
  }

  const applySelection = (indices, preferredIndex) => {
    const normalized = normalizeSelection(indices)
    selectedBoundaryIndices.value = normalized

    if (normalized.length === 0) {
      selectedBoundaryIndex.value = 0
      return
    }

    if (Number.isInteger(preferredIndex) && normalized.includes(preferredIndex)) {
      selectedBoundaryIndex.value = preferredIndex
      return
    }

    if (normalized.includes(selectedBoundaryIndex.value)) {
      return
    }

    selectedBoundaryIndex.value = normalized[0]
  }

  const handlePointerMove = event => {
    isDraggingBoundary.value = true
    updateDragPosition(event.clientX, dragStartPos.value.clientXToTime)
  }

  const consumeRecentBoundaryDrag = () => {
    if (!boundaryDragEndedAt.value) {
      return false
    }

    const recent = Date.now() - boundaryDragEndedAt.value < 500
    boundaryDragEndedAt.value = 0
    return recent
  }

  const handlePointerUp = () => {
    boundaryDragEndedAt.value = Date.now()

    if (dragState.value) {
      const rightWordIndex = dragState.value.rightWordIndex
      const isLineEdge = rightWordIndex === 0 || isLineEndBoundaryIndex(rightWordIndex)

      commitBoundary({
        rightWordIndex,
        startMs: dragState.value.currentStartMs,
        initialStartMs: dragState.value.initialStartMs,
        shouldReplay: true,
        // Preview separators are only a line boundary until word timings are saved.
        lineBoundsOnly: isLineEdge && hasPersistedWords?.value === false,
      })
    }

    setTimeout(() => {
      isDraggingBoundary.value = false
    }, 0)
    dragState.value = null
    dragStartPos.value = null
    stopBoundaryDrag()
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

    isDraggingBoundary.value = true
    dragState.value = {
      rightWordIndex: dragStartPos.value.rightWordIndex,
      initialStartMs: dragStartPos.value.initialStartMs,
      currentStartMs: dragStartPos.value.initialStartMs,
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

  const startBoundaryDrag = (rightWordIndex, event, clientXToTime) => {
    event.preventDefault()
    event.stopPropagation()

    if (!isWordSyncAvailable.value || words.value.length === 0) {
      return
    }

    const lineEndBoundaryIndex = getLineEndBoundaryIndex()
    if (rightWordIndex < 0 || rightWordIndex > lineEndBoundaryIndex) {
      return
    }

    const initialStartMs = isLineEndBoundaryIndex(rightWordIndex)
      ? getLineEndBoundaryTimeMs()
      : words.value[rightWordIndex]?.start_ms

    if (!Number.isFinite(initialStartMs)) {
      return
    }

    isDraggingBoundary.value = false
    dragStartPos.value = {
      x: event.clientX,
      y: event.clientY,
      rightWordIndex,
      initialStartMs,
      clientXToTime,
    }

    document.addEventListener('pointermove', handlePotentialDragStart)
    document.addEventListener('pointerup', handlePotentialDragEnd)
    document.addEventListener('pointercancel', handlePotentialDragEnd)
  }

  const selectBoundary = (index, event) => {
    if (!isWordSyncAvailable.value || words.value.length === 0) {
      return
    }

    if (index < 0 || index > getLineEndBoundaryIndex()) {
      return
    }

    if (isDraggingBoundary.value) {
      return
    }

    const currentSelection = selectedBoundaryIndices.value

    if (event?.shiftKey && currentSelection.length > 0) {
      const anchor = selectedBoundaryIndex.value
      const min = Math.min(anchor, index)
      const max = Math.max(anchor, index)
      const range = Array.from({ length: max - min + 1 }, (_, offset) => min + offset)
      applySelection(range, index)
      return
    }

    if (event?.ctrlKey || event?.metaKey) {
      const nextSelection = currentSelection.includes(index)
        ? currentSelection.filter(value => value !== index)
        : [...currentSelection, index]
      applySelection(nextSelection, index)
      return
    }

    applySelection([index], index)
  }

  const isBoundarySelected = index => selectedBoundaryIndices.value.includes(index)

  const selectPreviousBoundary = () => {
    if (!isWordSyncAvailable.value || words.value.length === 0) {
      return false
    }

    const nextIndex = Math.max(0, selectedBoundaryIndex.value - 1)
    applySelection([nextIndex], nextIndex)
    return true
  }

  const selectNextBoundary = () => {
    if (!isWordSyncAvailable.value || words.value.length === 0) {
      return false
    }

    const nextIndex = Math.min(getLineEndBoundaryIndex(), selectedBoundaryIndex.value + 1)
    applySelection([nextIndex], nextIndex)
    return true
  }

  const syncSelectedBoundary = (progressMs, options = {}) => {
    if (!isWordSyncAvailable.value) {
      return false
    }

    const advance = options.advance !== false

    const rightWordIndex = selectedBoundaryIndex.value
    const lineEndBoundaryIndex = getLineEndBoundaryIndex()
    if (rightWordIndex < 0 || rightWordIndex > lineEndBoundaryIndex) {
      return false
    }

    const currentWords = words.value
    const timelineMinStartMs = Number.isFinite(timelineStartMs.value) ? timelineStartMs.value : 0
    const lineEndBoundaryMs = getLineEndBoundaryTimeMs()
    const timelineMaxStartMs = Math.max(
      timelineMinStartMs,
      (Number.isFinite(lineEndBoundaryMs) ? lineEndBoundaryMs : timelineEndMs.value) - 1
    )

    if (isLineEndBoundaryIndex(rightWordIndex)) {
      const lastWord = currentWords[currentWords.length - 1]
      const minEndMs = Number.isFinite(lastWord?.start_ms) ? lastWord.start_ms + 1 : timelineMinStartMs
      const nextEndMs = Math.max(minEndMs, Math.round(progressMs))
      const currentEndMs = getLineEndBoundaryTimeMs()

      if (nextEndMs !== currentEndMs) {
        commitLineEndBoundary({ startMs: nextEndMs, shouldReplay: false })
      }

      return true
    }

    const boundaryCount = currentWords.length

    const absoluteMinForSelected = timelineMinStartMs + rightWordIndex
    const absoluteMaxForSelected = timelineMaxStartMs - (boundaryCount - 1 - rightWordIndex)
    const nextStartMs = Math.max(
      absoluteMinForSelected,
      Math.min(absoluteMaxForSelected, progressMs)
    )

    const nextStartValues = currentWords.map(word => word.start_ms)
    nextStartValues[rightWordIndex] = nextStartMs

    for (let index = rightWordIndex - 1; index >= 0; index--) {
      const minStartMs = timelineMinStartMs + index
      const maxStartMs = nextStartValues[index + 1] - 1
      nextStartValues[index] = Math.max(minStartMs, Math.min(nextStartValues[index], maxStartMs))
    }

    for (let index = rightWordIndex + 1; index < boundaryCount; index++) {
      const minStartMs = nextStartValues[index - 1] + 1
      const maxStartMs = timelineMaxStartMs - (boundaryCount - 1 - index)
      nextStartValues[index] = Math.min(maxStartMs, Math.max(nextStartValues[index], minStartMs))
    }

    const hasChanges = nextStartValues.some((startMs, index) => startMs !== currentWords[index].start_ms)

    if (hasChanges) {
      const updatedWords = currentWords.map((word, index) => {
        const updated = { ...word, start_ms: nextStartValues[index] }
        const nextIndex = index + 1
        if (
          nextIndex < boundaryCount &&
          nextStartValues[nextIndex] !== currentWords[nextIndex].start_ms
        ) {
          updated.end_ms = nextStartValues[nextIndex]
        }
        return updated
      })

      onUpdateWords({
        lineIndex: selectedLineIndex.value,
        words: updatedWords,
        lineStartMs: updatedWords[0].start_ms,
      })
    }

    if (advance) {
      const nextIndex = rightWordIndex + 1
      if (nextIndex <= getLineEndBoundaryIndex()) {
        applySelection([nextIndex], nextIndex)
      }
    }

    return true
  }

  const deleteSelectedBoundaries = () => {
    if (!isWordSyncAvailable.value || words.value.length <= 1) {
      return false
    }

    // Boundary 0 defines the line start and cannot be deleted.
    // The extra last-word-end boundary (index === words.length) also cannot.
    const deletableBoundaries = selectedBoundaryIndices.value
      .filter(index => index > 0 && index < words.value.length)
      .sort((a, b) => a - b)

    if (deletableBoundaries.length === 0) {
      return false
    }

    const deletedSet = new Set(deletableBoundaries)
    const mergedWords = []

    for (let index = 0; index < words.value.length; index++) {
      const word = words.value[index]

      if (index > 0 && deletedSet.has(index) && mergedWords.length > 0) {
        mergedWords[mergedWords.length - 1] = {
          ...mergedWords[mergedWords.length - 1],
          text: `${mergedWords[mergedWords.length - 1].text || ''}${word.text || ''}`,
          ...(Number.isFinite(word.end_ms) ? { end_ms: word.end_ms } : {}),
        }
        continue
      }

      mergedWords.push({ ...word })
    }

    onUpdateWords({
      lineIndex: selectedLineIndex.value,
      words: mergedWords,
    })

    const nextPrimary = Math.max(0, Math.min(mergedWords.length - 1, deletableBoundaries[0] - 1))
    applySelection([nextPrimary], nextPrimary)
    return true
  }

  const resetBoundarySelection = () => {
    const defaultBoundaryIndex = Math.min(1, Math.max(0, words.value.length - 1))
    applySelection([defaultBoundaryIndex], defaultBoundaryIndex)
  }

  const cancelBoundaryInteraction = () => {
    dragState.value = null
    dragStartPos.value = null
    isDraggingBoundary.value = false
    boundaryDragEndedAt.value = 0
    stopBoundaryDrag()
  }

  return {
    dragState,
    displayedWords,
    boundaryIndexes,
    getBoundaryTimeMs,
    getSeparatorPlayStartMs,
    consumeRecentBoundaryDrag,
    selectedBoundaryIndex,
    selectedBoundaryIndices,
    isDraggingBoundary,
    startBoundaryDrag,
    selectBoundary,
    isBoundarySelected,
    selectPreviousBoundary,
    selectNextBoundary,
    syncSelectedBoundary,
    shiftSelectedBoundaryBy,
    deleteSelectedBoundaries,
    stopBoundaryDrag,
    resetBoundarySelection,
    cancelBoundaryInteraction,
  }
}
