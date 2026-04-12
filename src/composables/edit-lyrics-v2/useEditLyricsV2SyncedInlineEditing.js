import { nextTick, ref } from 'vue'

export function useEditLyricsV2SyncedInlineEditing({
  modelValue,
  emit,
  selectLine,
  updateLineText,
}) {
  const editingLineIndex = ref(null)
  const editingText = ref('')
  const lineInput = ref(null)

  const setLineInputRef = element => {
    lineInput.value = element
  }

  const finishEditing = emitStateChange => {
    const wasEditing = editingLineIndex.value !== null
    editingLineIndex.value = null
    editingText.value = ''

    if (emitStateChange && wasEditing) {
      emit('editing-state-change', false)
    }
  }

  const startEditingLine = lineIndex => {
    if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= modelValue.value.length) {
      return
    }

    selectLine(lineIndex)
    editingLineIndex.value = lineIndex
    editingText.value = modelValue.value[lineIndex]?.text || ''
    emit('editing-state-change', true)

    nextTick(() => {
      lineInput.value?.focus()
      lineInput.value?.select()
    })
  }

  const saveEditingLine = () => {
    if (editingLineIndex.value === null) {
      return
    }

    // Use updateLineText if provided (handles word timing erasure)
    if (updateLineText) {
      updateLineText(editingLineIndex.value, editingText.value)
    } else {
      // Fallback to direct update
      const nextLines = modelValue.value.map((line, index) => {
        if (index !== editingLineIndex.value) {
          return line
        }

        return {
          ...line,
          text: editingText.value,
        }
      })

      emit('update:modelValue', nextLines)
    }

    finishEditing(true)
  }

  const cancelEditingLine = () => {
    finishEditing(true)
  }

  const handleLineCountChange = lineCount => {
    if (editingLineIndex.value !== null && editingLineIndex.value >= lineCount) {
      finishEditing(true)
    }
  }

  return {
    editingLineIndex,
    editingText,
    setLineInputRef,
    startEditingLine,
    saveEditingLine,
    cancelEditingLine,
    handleLineCountChange,
  }
}
