<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[80vh] max-w-screen-lg"
    @close="emit('close')"
    :title="`${editingTrack.name} - ${editingTrack.artistName}`"
  >
    <template #titleLeft>
      <div class="flex-none flex gap-1 items-center">
        <VTooltip theme="lrcget-tooltip">
          <button
            class="button text-sm px-5 py-1.5 h-8 w-24 rounded-full"
            :class="{ 'button-primary': !isDirty, 'button-disabled': isDirty }"
            :disabled="isDirty"
            @click="handlePublish"
          >
            Publish
          </button>

          <template #popper>
            <div v-if="isSynchronizedLyrics(unifiedLyrics)" class="text-xs font-bold">Publish synchronized lyrics to LRCLIB service</div>
            <div v-else class="text-xs font-bold">Publish plain text lyrics to LRCLIB service</div>
          </template>
        </VTooltip>

        <VTooltip v-if="lyricsLintResult.length === 0" theme="lrcget-tooltip">
          <Check class="text-lime-500 text-2xl block" />

          <template #popper>
            <div class="text-xs font-bold">No errors detected, you can publish it now</div>
          </template>
        </VTooltip>

        <VTooltip v-else-if="plainTextLintResult.length === 0" theme="lrcget-tooltip">
          <AlertCircleOutline class="text-orange-500 text-2xl block" />

          <template #popper>
            <div class="text-xs font-bold">Lyrics not synchronized<br />You can still publish it, but consider synchronizing it to help others</div>
          </template>
        </VTooltip>

        <VTooltip v-else theme="lrcget-tooltip">
          <AlertCircle class="text-red-500 text-2xl block" />

          <template #popper>
            <div class="text-xs font-bold">Lyrics error detected<br />Press the publish button for details</div>
          </template>
        </VTooltip>
      </div>
    </template>

    <div class="grow overflow-hidden flex flex-col gap-2 h-full">
      <div class="flex flex-col bg-brave-95 dark:bg-brave-10 rounded-lg">
        <div class="toolbar px-4 py-2 flex justify-between items-stretch gap-1">
          <div class="flex gap-1">
            <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Rewind line 100ms (Alt+LeftArrow)" @click="rewind100"><Minus /></button>
            <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Forward line 100ms (Alt+RightArrow)" @click="fastForward100"><Plus /></button>
          </div>

          <div>
            <button class="button button-warning px-3 py-1 text-lg rounded-full" title="Mark track as instrumental" @click="markAsInstrumental"><Music /> <span class="text-xs">Mark Instrumental</span></button>
          </div>
        </div>
      </div>

      <!-- NOTE: AsyncCodemirror component does not have @wheel event handler, so it has to be handled here (in the container) -->
      <div class="relative h-full w-full" id="cm-container" ref="cmContainer">
        <div class="overflow-hidden absolute w-full" :style="{ height: `${cmHeight}px` }" @keydown="handleKeydown" @wheel="handleWheel">
          <AsyncCodemirror
            v-if="shouldLoadCodeMirror"
            v-model="unifiedLyrics"
            placeholder="Lyrics is currently empty"
            class="codemirror-custom h-full outline-none"
            :autofocus="true"
            :indent-with-tab="true"
            :tab-size="2"
            :extensions="extensions"
            :config="{ height: 'auto' }"
            @ready="handleReady"
            @change="lyricsUpdated"
          />

          <div v-else class="flex flex-col h-full items-center justify-center text-sm text-brave-40">
            <div class="animate-spin text-xl text-brave-30"><Loading /></div>
            <div>Loading editor...</div>
          </div>
        </div>
      </div>

      <div class="flex flex-col w-fit self-end bg-brave-95 dark:bg-brave-10 rounded-lg">
        <div class="toolbar px-2 py-1 flex items-stretch gap-1">
          <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full" title="Zoom out" @click="changeCodemirrorFontSizeBy(-1)"><MagnifyMinus /></button>
          <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full w-[4.5em]" title="Reset zoom level" @click="resetCodemirrorFontSize">{{ (codemirrorStyle.fontSize * 100).toFixed(0) }}%</button>
          <button class="button button-normal px-1.5 py-0.5 text-sm rounded-full" title="Zoom in" @click="changeCodemirrorFontSizeBy(+1)"><MagnifyPlus /></button>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, onMounted, onUnmounted, shallowRef, watch } from 'vue'
import { Close, Loading, Minus, Plus, Check, AlertCircleOutline, AlertCircle, MagnifyPlus, MagnifyMinus, Music } from 'mdue'
import { Lrc, Runner, timestampToString, parseLine } from 'lrc-kit'
import { useGlobalState } from '@/composables/global-state.js'
import { useToast } from 'vue-toastification'
import PublishLyrics from '@/components/library/edit-lyrics/PublishLyrics.vue'
import PublishPlainText from '@/components/library/edit-lyrics/PublishPlainText.vue'
import { Decoration, EditorView } from '@codemirror/view'
import { StateField, StateEffect } from '@codemirror/state'
import { defineAsyncComponent } from 'vue'
import { executeLint as executeLyricsLint } from '@/utils/lyrics-lint.js'
import { executeLint as executePlainTextLint } from '@/utils/plain-text-lint.js'
import { useModal } from 'vue-final-modal'
import { isSynchronizedLyrics } from '@/utils/lyrics.js'

const AsyncCodemirror = defineAsyncComponent(async () => {
  const { Codemirror } = await import('vue-codemirror')
  return Codemirror
})

const props = defineProps(['editingTrack'])
const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const editingTrack = ref(props.editingTrack)

const unifiedLyrics = ref('')
const shouldLoadCodeMirror = ref(false)
const view = shallowRef()
const keydownEvent = ref(null)
const isDirty = ref(false)
const lyricsLintResult = ref([])
const plainTextLintResult = ref([])
const cmContainer = ref(null)
const cmHeight = ref(null)

const runner = ref(null)

const { open: openPublishLyricsModal, close: closePublishLyricsModal, patchOptions: patchPublishLyricsModalOptions } = useModal({
  component: PublishLyrics,
  attrs: {
    track: props.editingTrack,
    lyrics: unifiedLyrics.value,
    lintResult: lyricsLintResult.value,
    onClose() {
      closePublishLyricsModal()
    }
  }
})

const { open: openPublishPlainTextModal, close: closePublishPlainTextModal, patchOptions: patchPublishPlainTextModalOptions } = useModal({
  component: PublishPlainText,
  attrs: {
    track: props.editingTrack,
    lyrics: unifiedLyrics.value,
    lintResult: plainTextLintResult.value,
    onClose() {
      closePublishPlainTextModal()
    }
  }
})

const codemirrorStyle = ref({
  fontSize: 1.0
})

const addLineHighlight = StateEffect.define()

const lineHighlightField = StateField.define({
  create() {
    return Decoration.none;
  },
  update(lines, tr) {
    lines = lines.map(tr.changes)
    for (let e of tr.effects) {
      if (e.is(addLineHighlight) && e.value === null) {
        lines = Decoration.none
      } else if (e.is(addLineHighlight)) {
        lines = Decoration.none
        lines = lines.update({add: [lineHighlightMark.range(e.value)]})
      }
    }
    return lines
  },
  provide: (f) => EditorView.decorations.from(f)
})

const lineHighlightMark = Decoration.line({
  attributes: {
    class: 'cm-current-lyrics'
  }
})

const extensions = [lineHighlightField]

const handleWheel = (payload) => {
  if (!payload.ctrlKey) return;

  changeCodemirrorFontSizeBy(payload.deltaY > 0 ? -1 : +1)
}

const handleKeydown = (payload) => {
  if (!payload.ctrlKey) return;

  switch (payload.key) {
    case '+':
    case '=':
      changeCodemirrorFontSizeBy(+1)
      break;
    case '-':
    case '_':
      changeCodemirrorFontSizeBy(-1)
      break;
    default:
      break;
  }
}

const changeCodemirrorFontSizeBy = (offset) => {
  if (!shouldLoadCodeMirror) return;

  let newFontSize = codemirrorStyle.value.fontSize + offset * 0.1;
  if (newFontSize < 0.4) newFontSize = 0.4;

  codemirrorStyle.value.fontSize = +(newFontSize.toFixed(2));
}

const handleReady = (payload) => {
  view.value = payload.view

  view.value.dispatch({
    effects: EditorView.scrollIntoView(0)
  })
}

const resetCodemirrorFontSize = () => {
  if (!shouldLoadCodeMirror) return;

  codemirrorStyle.value.fontSize = 1.0
}

const lyricsUpdated = (newLyrics) => {
  const parsed = Lrc.parse(newLyrics)
  runner.value = new Runner(parsed)
  isDirty.value = true
  lyricsLintResult.value = executeLyricsLint(newLyrics)
  plainTextLintResult.value = executePlainTextLint(newLyrics)
}

const rewind100 = () => {
  const selection = view.value.state.selection
  const changes = []

  for (let range of selection.ranges) {
    const startLine = view.value.state.doc.lineAt(range.from)
    const endLine = view.value.state.doc.lineAt(range.to)

    for (let lineNo = startLine.number; lineNo <= endLine.number; lineNo++) {
      const currentLine = view.value.state.doc.line(lineNo)
      const currentLineText = currentLine.text
      const parsed = parseLine(currentLineText)

      if (parsed.type === 'TIME') {
        const timestamp = parsed.timestamps[0]
        const newTimestamp = Math.max(0, timestamp - 0.1)
        const stringifiedTime = timestampToString(newTimestamp)
        const replacedText = currentLineText.replace(/^(\s*\[(.*)\]\s*)*/g, `[${stringifiedTime}] `)

        changes.push({
          from: currentLine.from,
          to: currentLine.to,
          insert: replacedText
        })
      }
    }
  }

  if (changes.length > 0) {
    view.value.dispatch({ changes })
    view.value.focus()
    lyricsUpdated(view.value.state.doc.toString())
  }
}

const fastForward100 = () => {
  const selection = view.value.state.selection
  const changes = []

  for (let range of selection.ranges) {
    const startLine = view.value.state.doc.lineAt(range.from)
    const endLine = view.value.state.doc.lineAt(range.to)

    for (let lineNo = startLine.number; lineNo <= endLine.number; lineNo++) {
      const currentLine = view.value.state.doc.line(lineNo)
      const currentLineText = currentLine.text
      const parsed = parseLine(currentLineText)

      if (parsed.type === 'TIME') {
        const timestamp = parsed.timestamps[0]
        const newTimestamp = timestamp + 0.1
        const stringifiedTime = timestampToString(newTimestamp)
        const replacedText = currentLineText.replace(/^(\s*\[(.*)\]\s*)*/g, `[${stringifiedTime}] `)

        changes.push({
          from: currentLine.from,
          to: currentLine.to,
          insert: replacedText
        })
      }
    }
  }

  if (changes.length > 0) {
    view.value.dispatch({ changes })
    view.value.focus()
    lyricsUpdated(view.value.state.doc.toString())
  }
}

const markAsInstrumental = () => {
  if (!view.value) return

  const newContent = '[au: instrumental]'

  view.value.dispatch({
    changes: {
      from: 0,
      to: view.value.state.doc.length,
      insert: newContent
    }
  })

  view.value.focus()

  lyricsUpdated(view.value.state.doc.toString())
}

const handleResize = () => {
  cmHeight.value = cmContainer.value.offsetHeight
}

const handlePublish = () => {
  if (isSynchronizedLyrics(unifiedLyrics.value)) {
    patchPublishLyricsModalOptions({
      attrs: {
        title: editingTrack.value.name,
        albumName: editingTrack.value.albumName,
        artistName: editingTrack.value.artistName,
        duration: editingTrack.value.duration,
        lyrics: unifiedLyrics.value,
        lintResult: lyricsLintResult.value,
      }
    })
    openPublishLyricsModal()
  } else {
    patchPublishPlainTextModalOptions({
      attrs: {
        title: editingTrack.value.name,
        albumName: editingTrack.value.albumName,
        artistName: editingTrack.value.artistName,
        duration: editingTrack.value.duration,
        lyrics: unifiedLyrics.value,
        lintResult: plainTextLintResult.value,
      }
    })
    openPublishPlainTextModal()
  }
}

onUnmounted(async () => {
  enableHotkey()
  if (keydownEvent.value) {
    document.removeEventListener(keydownEvent.value)
  }

  // stop monitoring the window size
  window.removeEventListener('resize', handleResize)
})

onMounted(async () => {
  disableHotkey()
  if (!editingTrack.value) {
    return
  }

  if (editingTrack.value.syncedLyrics) {
    unifiedLyrics.value = editingTrack.value.syncedLyrics
  } else if (editingTrack.value.plainLyrics) {
    unifiedLyrics.value = editingTrack.value.plainLyrics
  } else if (editingTrack.value.instrumental) {
    unifiedLyrics.value = '[au: instrumental]'
  } else {
    unifiedLyrics.value = ''
  }

  const parsed = Lrc.parse(unifiedLyrics.value)

  runner.value = new Runner(parsed)

  keydownEvent.value = document.addEventListener('keydown', (event) => {
    if (event.altKey === true && event.key === 'ArrowLeft') {
      event.preventDefault()
      rewind100()
    } else if (event.altKey === true && event.key === 'ArrowRight') {
      event.preventDefault()
      fastForward100()
    }
  })

  lyricsLintResult.value = executeLyricsLint(unifiedLyrics.value)
  plainTextLintResult.value = executePlainTextLint(unifiedLyrics.value)
})

watch(cmContainer, () => {
  if (cmContainer.value) {
    setTimeout(() => shouldLoadCodeMirror.value = true, 100)

    // Monitor the window size and dynamically adjust the height of the CodeMirror editor accordingly
    window.addEventListener('resize', handleResize)
    handleResize()
    return () => window.removeEventListener('resize', handleResize)
  }
})

watch(() => props.editingTrack, () => {
  if (props.editingTrack) {
    editingTrack.value = props.editingTrack

    if (editingTrack.value.syncedLyrics) {
      unifiedLyrics.value = editingTrack.value.syncedLyrics
    } else if (editingTrack.value.plainLyrics) {
      unifiedLyrics.value = editingTrack.value.plainLyrics
    } else if (editingTrack.value.instrumental) {
      unifiedLyrics.value = '[au: instrumental]'
    } else {
      unifiedLyrics.value = ''
    }
  }
})
</script>


<style scoped>
.codemirror-custom {
  font-size: calc(var(--codemirror-font-size) * 1em);
}
</style>
