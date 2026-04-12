import { nextTick, ref } from 'vue'

const INSERT_HOVER_MAX_DISTANCE_PX = 26

const resolveRowElement = value => {
  if (value instanceof HTMLElement) {
    return value
  }

  if (value?.rowElement instanceof HTMLElement) {
    return value.rowElement
  }

  if (value?.rowElement?.value instanceof HTMLElement) {
    return value.rowElement.value
  }

  if (value?.$el instanceof HTMLElement) {
    return value.$el
  }

  return null
}

export function useEditLyricsV2SyncedInsertHover({ modelValue }) {
  const lineRowElements = ref([])
  const hoveredInsertIndex = ref(-1)
  const hoveredInsertOpacity = ref(0)
  const lastMouseClientY = ref(null)

  const setLineRowRef = (value, index) => {
    lineRowElements.value[index] = resolveRowElement(value)
  }

  const getInsertCenterY = insertIndex => {
    const lineCount = modelValue.value.length
    if (lineCount === 0) {
      return null
    }

    if (insertIndex === 0) {
      return lineRowElements.value[0]?.getBoundingClientRect().top ?? null
    }

    if (insertIndex === lineCount) {
      return lineRowElements.value[lineCount - 1]?.getBoundingClientRect().bottom ?? null
    }

    const previousRow = lineRowElements.value[insertIndex - 1]
    const nextRow = lineRowElements.value[insertIndex]
    if (!previousRow || !nextRow) {
      return null
    }

    const previousRect = previousRow.getBoundingClientRect()
    const nextRect = nextRow.getBoundingClientRect()
    return (previousRect.bottom + nextRect.top) / 2
  }

  const resetHoveredInsert = () => {
    hoveredInsertIndex.value = -1
    hoveredInsertOpacity.value = 0
  }

  const updateHoveredInsertByMouseY = mouseClientY => {
    if (!Number.isFinite(mouseClientY) || modelValue.value.length < 1) {
      resetHoveredInsert()
      return
    }

    let nearestInsertIndex = -1
    let nearestDistance = Number.POSITIVE_INFINITY

    for (let insertIndex = 0; insertIndex <= modelValue.value.length; insertIndex += 1) {
      const centerY = getInsertCenterY(insertIndex)
      if (!Number.isFinite(centerY)) {
        continue
      }

      const distance = Math.abs(mouseClientY - centerY)
      if (distance < nearestDistance) {
        nearestDistance = distance
        nearestInsertIndex = insertIndex
      }
    }

    if (nearestDistance > INSERT_HOVER_MAX_DISTANCE_PX || nearestInsertIndex === -1) {
      resetHoveredInsert()
      return
    }

    hoveredInsertIndex.value = nearestInsertIndex
    hoveredInsertOpacity.value = Math.max(0, 1 - nearestDistance / INSERT_HOVER_MAX_DISTANCE_PX)
  }

  const insertControlOpacity = insertIndex => {
    if (hoveredInsertIndex.value !== insertIndex) {
      return 0
    }

    return hoveredInsertOpacity.value
  }

  const handleLinesMouseMove = event => {
    lastMouseClientY.value = event.clientY
    updateHoveredInsertByMouseY(event.clientY)
  }

  const handleLinesMouseLeave = () => {
    lastMouseClientY.value = null
    resetHoveredInsert()
  }

  const handleLinesScroll = () => {
    if (!Number.isFinite(lastMouseClientY.value)) {
      return
    }

    updateHoveredInsertByMouseY(lastMouseClientY.value)
  }

  const handleLineCountChange = lineCount => {
    lineRowElements.value = lineRowElements.value.slice(0, lineCount)

    if (lineCount < 1) {
      resetHoveredInsert()
      return
    }

    if (Number.isFinite(lastMouseClientY.value)) {
      nextTick(() => {
        updateHoveredInsertByMouseY(lastMouseClientY.value)
      })
    }
  }

  return {
    lineRowElements,
    setLineRowRef,
    insertControlOpacity,
    handleLinesMouseMove,
    handleLinesMouseLeave,
    handleLinesScroll,
    handleLineCountChange,
  }
}
