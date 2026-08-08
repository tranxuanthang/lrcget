import { computed, ref } from 'vue'

const DRAG_THRESHOLD = 3

export function useEditLyricsV2WordBoundaryDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  timelineStartMs,
  timelineEndMs,
  selectedLineIndex,
  onUpdateWords,
  onWordTimingEdited,
}) {
  const dragState = ref(null)
  const selectedBoundaryIndex = ref(0)
  const selectedBoundaryIndices = ref([0])
  const isDraggingBoundary = ref(false)
  const dragStartPos = ref(null)

  const displayedWords = computed(() => {
    const currentWords = words.value

    if (!dragState.value) {
      return currentWords
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

    return Array.from({ length: displayedWords.value.length }, (_, index) => index)
  })

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
    const nextStartMs = currentWords[rightWordIndex + 1]?.start_ms

    if (rightWordIndex === 0) {
      const minStartMs = Number.isFinite(timelineStartMs.value) ? timelineStartMs.value : 0
      const maxStartMs = Number.isFinite(nextStartMs) ? nextStartMs - 1 : timelineEndMs.value - 1

      return {
        minStartMs,
        maxStartMs: Math.max(minStartMs, maxStartMs),
      }
    }

    const previousStartMs = currentWords[rightWordIndex - 1]?.start_ms ?? timelineStartMs.value
    const minStartMs = previousStartMs + 1
    const maxStartMs = Number.isFinite(nextStartMs) ? nextStartMs - 1 : timelineEndMs.value - 1

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

  const commitBoundary = ({ rightWordIndex, startMs, initialStartMs, shouldReplay }) => {
    if (startMs === initialStartMs) {
      return false
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
        startMs: rightWordIndex === 0 ? startMs : lineStartMs.value,
      })
    }

    return true
  }

  const normalizeSelection = indices => {
    const maxIndex = Math.max(0, words.value.length - 1)
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

  const handlePointerUp = () => {
    if (dragState.value) {
      commitBoundary({
        rightWordIndex: dragState.value.rightWordIndex,
        startMs: dragState.value.currentStartMs,
        initialStartMs: dragState.value.initialStartMs,
        shouldReplay: true,
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

    if (!isWordSyncAvailable.value || rightWordIndex < 0 || rightWordIndex >= words.value.length) {
      return
    }

    isDraggingBoundary.value = false
    dragStartPos.value = {
      x: event.clientX,
      y: event.clientY,
      rightWordIndex,
      initialStartMs: words.value[rightWordIndex].start_ms,
      clientXToTime,
    }

    document.addEventListener('pointermove', handlePotentialDragStart)
    document.addEventListener('pointerup', handlePotentialDragEnd)
    document.addEventListener('pointercancel', handlePotentialDragEnd)
  }

  const selectBoundary = (index, event) => {
    if (!isWordSyncAvailable.value || index < 0 || index >= words.value.length) {
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

    const nextIndex = Math.min(words.value.length - 1, selectedBoundaryIndex.value + 1)
    applySelection([nextIndex], nextIndex)
    return true
  }

  const syncSelectedBoundary = (progressMs, options = {}) => {
    if (!isWordSyncAvailable.value) {
      return false
    }

    const advance = options.advance !== false

    const rightWordIndex = selectedBoundaryIndex.value
    if (rightWordIndex < 0 || rightWordIndex >= words.value.length) {
      return false
    }

    const currentWords = words.value
    const timelineMinStartMs = Number.isFinite(timelineStartMs.value) ? timelineStartMs.value : 0
    const timelineMaxStartMs = Math.max(timelineMinStartMs, timelineEndMs.value - 1)
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
      if (nextIndex < words.value.length) {
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
    stopBoundaryDrag()
  }

  return {
    dragState,
    displayedWords,
    boundaryIndexes,
    selectedBoundaryIndex,
    selectedBoundaryIndices,
    isDraggingBoundary,
    startBoundaryDrag,
    selectBoundary,
    isBoundarySelected,
    selectPreviousBoundary,
    selectNextBoundary,
    syncSelectedBoundary,
    deleteSelectedBoundaries,
    stopBoundaryDrag,
    resetBoundarySelection,
    cancelBoundaryInteraction,
  }
}
