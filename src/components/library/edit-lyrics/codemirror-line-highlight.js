import { StateEffect, StateField } from '@codemirror/state'
import { Decoration, EditorView } from '@codemirror/view'

export const addLineHighlight = StateEffect.define()

const lineHighlightMark = Decoration.line({
  attributes: {
    class: 'cm-current-lyrics',
  },
})

const lineHighlightField = StateField.define({
  create() {
    return Decoration.none
  },
  update(lines, transaction) {
    let nextLines = lines.map(transaction.changes)

    for (const effect of transaction.effects) {
      if (!effect.is(addLineHighlight)) {
        continue
      }

      if (effect.value === null) {
        nextLines = Decoration.none
        continue
      }

      nextLines = Decoration.none.update({
        add: [lineHighlightMark.range(effect.value)],
      })
    }

    return nextLines
  },
  provide: field => EditorView.decorations.from(field),
})

export const lineHighlightExtensions = [lineHighlightField]
