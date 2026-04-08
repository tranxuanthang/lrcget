import { computed, ref } from 'vue'

const DRAG_THRESHOLD = 3

export function useEditLyricsV2WordBoundaryDrag({
  isWordSyncAvailable,
  words,
  lineStartMs,
  lineEndMs,
  selectedLineIndex,
  onUpdateWords,
  onWordTimingEdited
}) {
  const dragState = ref(null)
  const selectedBoundaryIndex = ref(1)
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
        start_ms: dragState.value.currentStartMs
      }
    })
  })

  const boundaryIndexes = computed(() => {
    if (displayedWords.value.length < 2) {
      return []
    }

    return Array.from({ length: displayedWords.value.length - 1 }, (_, index) => index + 1)
  })

  const buildUpdatedWords = (rightWordIndex, startMs) => {
    return words.value.map((word, index) => {
      if (index !== rightWordIndex) {
        return word
      }

      return {
        ...word,
        start_ms: startMs
      }
    })
  }

  const getBoundaryConstraint = (rightWordIndex) => {
    const currentWords = words.value
    const previousStartMs = currentWords[rightWordIndex - 1]?.start_ms ?? lineStartMs.value
    const nextStartMs = currentWords[rightWordIndex + 1]?.start_ms
    const minStartMs = previousStartMs + 1
    const maxStartMs = Number.isFinite(nextStartMs)
      ? nextStartMs - 1
      : lineEndMs.value - 1

    return {
      minStartMs,
      maxStartMs: Math.max(minStartMs, maxStartMs)
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
      currentStartMs: Math.max(minStartMs, Math.min(maxStartMs, nextStartMs))
    }
  }

  const commitBoundary = ({ rightWordIndex, startMs, initialStartMs, shouldReplay }) => {
    if (startMs === initialStartMs) {
      return false
    }

    onUpdateWords({
      lineIndex: selectedLineIndex.value,
      words: buildUpdatedWords(rightWordIndex, startMs)
    })

    if (shouldReplay) {
      onWordTimingEdited({
        lineIndex: selectedLineIndex.value,
        startMs: lineStartMs.value
      })
    }

    return true
  }

  const handlePointerMove = (event) => {
    isDraggingBoundary.value = true
    updateDragPosition(event.clientX, dragStartPos.value.clientXToTime)
  }

  const handlePointerUp = () => {
    if (dragState.value) {
      commitBoundary({
        rightWordIndex: dragState.value.rightWordIndex,
        startMs: dragState.value.currentStartMs,
        initialStartMs: dragState.value.initialStartMs,
        shouldReplay: true
      })
    }

    setTimeout(() => {
      isDraggingBoundary.value = false
    }, 0)
    dragState.value = null
    dragStartPos.value = null
    stopBoundaryDrag()
  }

  const handlePotentialDragStart = (event) => {
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
      currentStartMs: dragStartPos.value.initialStartMs
    }

    const clientXToTime = dragStartPos.value.clientXToTime
    dragStartPos.value = {
      clientXToTime
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

    if (!isWordSyncAvailable.value || rightWordIndex <= 0 || rightWordIndex >= words.value.length) {
      return
    }

    isDraggingBoundary.value = false
    dragStartPos.value = {
      x: event.clientX,
      y: event.clientY,
      rightWordIndex,
      initialStartMs: words.value[rightWordIndex].start_ms,
      clientXToTime
    }

    document.addEventListener('pointermove', handlePotentialDragStart)
    document.addEventListener('pointerup', handlePotentialDragEnd)
    document.addEventListener('pointercancel', handlePotentialDragEnd)
  }

  const selectBoundary = (index) => {
    if (!isWordSyncAvailable.value || index <= 0 || index >= words.value.length) {
      return
    }

    if (isDraggingBoundary.value) {
      return
    }

    selectedBoundaryIndex.value = index
  }

  const syncSelectedBoundary = (progressMs) => {
    if (!isWordSyncAvailable.value) {
      return false
    }

    const rightWordIndex = selectedBoundaryIndex.value
    if (rightWordIndex <= 0 || rightWordIndex >= words.value.length) {
      return false
    }

    const { minStartMs, maxStartMs } = getBoundaryConstraint(rightWordIndex)
    const newStartMs = Math.max(minStartMs, Math.min(maxStartMs, progressMs))
    const oldStartMs = words.value[rightWordIndex].start_ms

    if (newStartMs !== oldStartMs) {
      commitBoundary({
        rightWordIndex,
        startMs: newStartMs,
        initialStartMs: oldStartMs,
        shouldReplay: false
      })
    }

    const nextIndex = rightWordIndex + 1
    if (nextIndex < words.value.length) {
      selectedBoundaryIndex.value = nextIndex
    }

    return true
  }

  const resetBoundarySelection = () => {
    selectedBoundaryIndex.value = 1
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
    isDraggingBoundary,
    startBoundaryDrag,
    selectBoundary,
    syncSelectedBoundary,
    stopBoundaryDrag,
    resetBoundarySelection,
    cancelBoundaryInteraction
  }
}
