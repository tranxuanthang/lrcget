<template>
    <BaseModal
      :click-to-close="false"
      :esc-to-close="false"
      content-class="w-full h-[80vh] max-w-screen-lg"
      @close="emit('close')"
      :title="`${editingTrack.title} - ${editingTrack.artist_name}`"
    >
      <template #titleLeft>
        <div class="flex-none flex gap-1 items-center">
          <VTooltip theme="lrcget-tooltip">
            <button
              class="button text-sm px-5 py-1.5 h-8 w-24 rounded-full"
              :class="{ 'button-primary': isDirty, 'button-disabled': !isDirty }"
              :disabled="!isDirty"
              @click="saveLyrics"
            >
              Save
            </button>

            <template #popper>
              <div class="text-xs font-bold">
                Save lyrics
                <span class="text-[0.65rem] text-brave-30 bg-brave-95 px-1 rounded-full">Ctrl+S</span>
              </div>
            </template>
          </VTooltip>

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
        <div class="toolbar flex flex-col bg-brave-95 dark:bg-brave-10 rounded-lg">
          <div class="px-4 py-2 flex justify-between items-stretch gap-1">
            <div class="flex gap-1">
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Sync line & move next (Alt+Enter)" @click="syncLine"><EqualEnter /> <span class="text-xs">Sync Line & Move Next</span></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Sync line (Alt+X)" @click="syncLine(false)"><Equal /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Rewind line 100ms (Alt+LeftArrow)" @click="rewind100"><Minus /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Forward line 100ms (Alt+RightArrow)" @click="fastForward100"><Plus /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Replay line (Alt+Z)" @click="repeatLine"><MotionPlay /></button>
            </div>

            <div>
              <button class="button button-warning px-3 py-1 text-lg rounded-full" title="Mark track as instrumental" @click="markAsInstrumental"><Music /> <span class="text-xs">Mark Instrumental</span></button>
            </div>
          </div>
          <div class="w-full border-b border-brave-90 dark:border-brave-20"></div>
          <div class="flex gap-4 items-center px-4 py-2">
            <button v-if="status !== 'playing'" @click.prevent="resumeOrPlay" class="button button-normal p-2 rounded-full text-xl"><Play /></button>
            <button v-else @click.prevent="pause" class="button button-normal p-2 rounded-full text-xl"><Pause /></button>
            <div class="flex-none w-12 text-xs">{{ humanDuration(progress) }}</div>
            <Seek class="grow" :duration="duration" :progress="progress" @seek="seek" />
            <div class="flex-none w-12 text-xs">{{ humanDuration(duration) }}</div>
          </div>
        </div>
        <button @click="romanize">
          Romanizar
        </button>
        <!-- NOTE: AsyncCodemirror component does not have @wheel event handler, so it has to be handled here (in the container) -->
        <div class="relative h-full w-full" id="cm-container" ref="cmContainer">
          <div class="flex flex-row">
            <div class="overflow-hidden  w-full" :style="{ height: `${cmHeight}px` }" @wheel="handleWheel">
              <AsyncCodemirror
                v-if="shouldLoadCodeMirror"
                v-model="unifiedLyrics"
                placeholder="Lyrics is currently empty"
                class="codemirror-custom h-full outline-none"
                :style="{ fontSize: `${codemirrorStyle.fontSize}em` }"
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
            <div class="overflow-hidden  w-full" :style="{ height: `${cmHeight}px` }" @wheel="handleWheel">
              <AsyncCodemirror
                v-if="shouldLoadCodeMirror"
                v-model="romanizedLyrics"
                placeholder="Lyrics is currently empty"
                class="codemirror-custom h-full outline-none"
                :style="{ fontSize: `${codemirrorStyle.fontSize}em` }"
                :indent-with-tab="true"
                :tab-size="2"
                :config="{ height: 'auto' }"
                @ready="handleReady"

              />

              <div v-else class="flex flex-col h-full items-center justify-center text-sm text-brave-40">
                <div class="animate-spin text-xl text-brave-30"><Loading /></div>
                <div>Loading editor...</div>
              </div>


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

<script setup lang="ts">
import {invoke} from '@tauri-apps/api/core'
import {defineAsyncComponent, onMounted, onUnmounted, ref, watch} from 'vue'
import {
  AlertCircle,
  AlertCircleOutline,
  Check,
  Equal,
  Loading,
  MagnifyMinus,
  MagnifyPlus,
  Minus,
  MotionPlay,
  Music,
  Pause,
  Play,
  Plus
} from 'mdue'
import EqualEnter from '@/components/icons/EqualEnter.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import {useToast} from 'vue-toastification'
import {Lrc, parseLine, Runner} from 'lrc-kit'
import {detectStandard, isSynchronizedLyrics, timestampToString} from '@/utils/lyrics.js'
import {useEditLyrics} from '@/composables/edit-lyrics.js'
import {usePlayer} from '@/composables/player.js'
import {useGlobalState} from '@/composables/global-state.js'
import Seek from '@/components/now-playing/Seek.vue'
import PublishLyrics from './edit-lyrics/PublishLyrics.vue'
import PublishPlainText from './edit-lyrics/PublishPlainText.vue'
import {Decoration, EditorView} from '@codemirror/view'
import {StateEffect, StateField} from '@codemirror/state'
import {executeLint as executeLyricsLint} from '@/utils/lyrics-lint.js'
import {executeLint as executePlainTextLint} from '@/utils/plain-text-lint.js'
import {useModal} from 'vue-final-modal'

import {canonicalize, romanizeJapanese} from "@/ts-utils/utils";

const romanizedLyrics = ref('')
const romanize = async ()=>{
    romanizedLyrics.value='kiwi'
  // await romanizeJapaneseOrHangul(lines[i])
  romanizedLyrics.value=await romanizeJapanese('辞書内での単語ID')
}

const AsyncCodemirror = defineAsyncComponent(async () => {
  const { Codemirror } = await import('vue-codemirror')
  return Codemirror
})

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const toast = useToast()
const { editingTrack } = useEditLyrics()

const unifiedLyrics = ref('')
const shouldLoadCodeMirror = ref(false)
const isDirty = ref(false)
const lyricsLintResult = ref([])
const plainTextLintResult = ref([])
const cmContainer = ref(null)
const cmHeight = ref(null)
const currentIndex = ref(null)

let view = null
let runner = null
let keydownEvent = null

const hotkeyConfig = [
  { keys: 'Alt+Enter', handler: () => syncLine() },
  { keys: 'Alt+X', handler: () => syncLine(false) },
  { keys: 'Alt+Z', handler: () => repeatLine() },
  { keys: 'Ctrl+S', handler: () => saveLyrics() },
  { keys: 'Alt+ArrowLeft', handler: () => rewind100() },
  { keys: 'Alt+ArrowRight', handler: () => fastForward100() },
  { keys: 'Ctrl+Plus', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+=', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+-', handler: () => changeCodemirrorFontSizeBy(-1) },
  { keys: 'Ctrl+_', handler: () => changeCodemirrorFontSizeBy(-1) }
]

const createHotkeyHandler = (config) => (event) => {
  for (const { keys, handler } of config) {
    const [modifier, key] = keys.split('+')
    const expectedModifier = modifier.toLowerCase()
    const expectedKey = key.toLowerCase()

    const modifierMatch = {
      alt: event.altKey,
      ctrl: event.ctrlKey || event.metaKey
    }[expectedModifier]

    if (modifierMatch && event.key.toLowerCase() === expectedKey) {
      event.preventDefault()
      handler()
      break
    }
  }
}

const { open: openPublishLyricsModal, close: closePublishLyricsModal, patchOptions: patchPublishLyricsModalOptions } = useModal({
  component: PublishLyrics,
  attrs: {
    track: editingTrack.value,
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
    track: editingTrack.value,
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

const humanDuration = (seconds) => {
  if (!seconds) {
    seconds = 0
  }

  return new Date(seconds * 1000).toISOString().slice(11, 19)
}

const resumeOrPlay = () => {
  if (status.value === 'paused') {
    resume()
  } else {
    playTrack(editingTrack.value)
  }
}

const handleWheel = (payload) => {
  if (!payload.ctrlKey) return;

  changeCodemirrorFontSizeBy(payload.deltaY > 0 ? -1 : +1)
}

const changeCodemirrorFontSizeBy = (offset) => {
  if (!shouldLoadCodeMirror) return;


  let newFontSize = codemirrorStyle.value.fontSize + offset * 0.1;
  if (newFontSize < 0.4) newFontSize = 0.4;

  codemirrorStyle.value.fontSize = +(newFontSize.toFixed(2));
}

const handleReady = (payload) => {
  view = payload.view

  view.dispatch({
    effects: EditorView.scrollIntoView(0)
  })
}

const resetCodemirrorFontSize = () => {
  if (!shouldLoadCodeMirror) return;

  codemirrorStyle.value.fontSize = 1.0
}

const lyricsUpdated =  async (newLyrics) => {
  debugger
  const parsed = Lrc.parse(newLyrics)
  runner = new Runner(parsed)
  isDirty.value = true
  lyricsLintResult.value = executeLyricsLint(newLyrics)


  let romanized: string = ''
  for (const lyric of parsed.lyrics) {

    let ro = canonicalize(lyric.content);
    ro = await romanizeJapanese(ro)
    romanized += canonicalize(ro) + '\n';
  }
  romanizedLyrics.value = romanized

  // romanizedLyrics.value = await romanizeJapanese(parsed.lyrics[0].content)
  plainTextLintResult.value = executePlainTextLint(newLyrics)
}

const syncLine = (moveNext = true) => {
  const currentLine = view.state.doc.lineAt(view.state.selection.main.head)
  const currentLineText = currentLine.text

  const standard = detectStandard(unifiedLyrics.value)

  const stringifiedTime = timestampToString(progress.value, standard.msPrecision)
  const replacedText = currentLineText.replace(
    /^(\s*\[(.*)\]\s*)*/g,
    `[${stringifiedTime}]${standard.space ? ' ': ''}`
  )

  view.dispatch({
    changes: {
      from: currentLine.from,
      to: currentLine.to,
      insert: replacedText
    }
  })

  if (moveNext) {
    const newLine = view.state.doc.lineAt(view.state.selection.main.head)
    let targetLineNumber = newLine.number

    // Keep checking subsequent lines until we find a non-empty one or reach the end
    while (targetLineNumber + 1 <= view.state.doc.lines) {
      const nextLine = view.state.doc.line(targetLineNumber + 1)
      if (nextLine.text.trim() !== '') {
        break
      }
      targetLineNumber++
    }

    // Move to target line if it exists
    if (targetLineNumber + 1 <= view.state.doc.lines) {
      const targetLine = view.state.doc.line(targetLineNumber + 1)
      view.dispatch({
        selection: {
          anchor: targetLine.from
        }
      })
      view.dispatch({
        effects: EditorView.scrollIntoView(targetLine.from, { y: 'center', behavior: 'smooth' })
      })
    }
  }

  view.focus()

  lyricsUpdated(view.state.doc.toString())
}

const repeatLine = () => {
  const currentLine = view.state.doc.lineAt(view.state.selection.main.head)
  const currentLineText = currentLine.text
  const parsed = parseLine(currentLineText)
  if (parsed.type === 'TIME') {
    seek(parsed.timestamps[0])
  }
}

const rewind100 = () => {
  const selection = view.state.selection
  const changes = []

  for (let range of selection.ranges) {
    const startLine = view.state.doc.lineAt(range.from)
    const endLine = view.state.doc.lineAt(range.to)

    for (let lineNo = startLine.number; lineNo <= endLine.number; lineNo++) {
      const currentLine = view.state.doc.line(lineNo)
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
    view.dispatch({ changes })
    view.focus()
    lyricsUpdated(view.state.doc.toString())

    // Seek to the timestamp of the first changed line
    const firstChangedLine = view.state.doc.lineAt(changes[0].from)
    const firstParsed = parseLine(firstChangedLine.text)
    if (firstParsed.type === 'TIME') {
      seek(firstParsed.timestamps[0])
    }
  }
}

const fastForward100 = () => {
  const selection = view.state.selection
  const changes = []

  for (let range of selection.ranges) {
    const startLine = view.state.doc.lineAt(range.from)
    const endLine = view.state.doc.lineAt(range.to)

    for (let lineNo = startLine.number; lineNo <= endLine.number; lineNo++) {
      const currentLine = view.state.doc.line(lineNo)
      const currentLineText = currentLine.text
      const parsed = parseLine(currentLineText)

      if (parsed.type === 'TIME') {
        const timestamp = parsed.timestamps[0]
        const newTimestamp = Math.min(duration.value, timestamp + 0.1)
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
    view.dispatch({ changes })
    view.focus()
    lyricsUpdated(view.state.doc.toString())

    // Seek to the timestamp of the first changed line
    const firstChangedLine = view.state.doc.lineAt(changes[0].from)
    const firstParsed = parseLine(firstChangedLine.text)
    if (firstParsed.type === 'TIME') {
      seek(firstParsed.timestamps[0])
    }
  }
}

const markAsInstrumental = () => {
  if (!view) return

  const newContent = '[au: instrumental]'

  view.dispatch({
    changes: {
      from: 0,
      to: view.state.doc.length,
      insert: newContent
    }
  })

  view.focus()

  lyricsUpdated(view.state.doc.toString())
}

const handleResize = () => {
  cmHeight.value = cmContainer.value.offsetHeight
}

const handlePublish = () => {
  if (isSynchronizedLyrics(unifiedLyrics.value)) {
    patchPublishLyricsModalOptions({
      attrs: {
        title: editingTrack.value.title,
        albumName: editingTrack.value.album_name,
        artistName: editingTrack.value.artist_name,
        duration: editingTrack.value.duration,
        lyrics: unifiedLyrics.value,
        lintResult: lyricsLintResult.value,
      }
    })
    openPublishLyricsModal()
  } else {
    patchPublishPlainTextModalOptions({
      attrs: {
        title: editingTrack.value.title,
        albumName: editingTrack.value.album_name,
        artistName: editingTrack.value.artist_name,
        duration: editingTrack.value.duration,
        lyrics: unifiedLyrics.value,
        lintResult: plainTextLintResult.value,
      }
    })
    openPublishPlainTextModal()
  }
}

const saveLyrics = async () => {
  try {
    const isLyricsSynced = /^\[.*\]/m.test(unifiedLyrics.value);
    await invoke('save_lyrics', {
      trackId: editingTrack.value.id,
      plainLyrics: unifiedLyrics.value.replace(/^\[(.*)\] */mg, ''),
      syncedLyrics: isLyricsSynced ? unifiedLyrics.value : ''
    })
    isDirty.value = false
  } catch (error) {
    console.error(error)
    toast.error(error)
  }
}

onUnmounted(async () => {
  runner = null

  enableHotkey()
  if (keydownEvent) {
    document.removeEventListener('keydown', keydownEvent)
  }
  window.removeEventListener('resize', handleResize)
})

onMounted(() => {
  disableHotkey()

  if (!editingTrack.value) {
    return
  }

  if (editingTrack.value.lrc_lyrics) {
    unifiedLyrics.value = editingTrack.value.lrc_lyrics
  } else if (editingTrack.value.txt_lyrics) {
    unifiedLyrics.value = editingTrack.value.txt_lyrics
  } else {
    unifiedLyrics.value = ''
  }

  if (!playingTrack.value || playingTrack.value.id !== editingTrack.value.id) {
    playTrack(editingTrack.value)
    pause()
  }

  const parsed = Lrc.parse(unifiedLyrics.value)

  runner = new Runner(parsed)

  keydownEvent = createHotkeyHandler(hotkeyConfig)
  document.addEventListener('keydown', keydownEvent)

  lyricsLintResult.value = executeLyricsLint(unifiedLyrics.value)
  plainTextLintResult.value = executePlainTextLint(unifiedLyrics.value)
})

watch(progress, (newProgress) => {
  if (!editingTrack.value || !view) {
    return
  }

  if (!runner || !unifiedLyrics.value) {
    return
  }

  runner.timeUpdate(newProgress)
  let resultCurrentIndex = runner.curIndex()

  if (resultCurrentIndex === null) {
    resultCurrentIndex = -1
  }

  currentIndex.value = resultCurrentIndex

  if (currentIndex.value >= 0) {
    const line = view.state.doc.line(currentIndex.value + 1)
    view.dispatch({ effects: addLineHighlight.of(line.from) })
  } else {
    view.dispatch({ effects: addLineHighlight.of(null) })
  }
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
</script>
