<template>
  <div>
    <div class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-30 select-none" :class="{ 'hidden': !props.isShow }">
      <div class="w-full h-[80vh] max-w-screen-lg rounded-lg m-4 bg-white flex flex-col gap-2">
        <div class="flex-none flex justify-between items-center px-6 py-2">
          <div class="flex gap-1">
            <button v-if="isDirty" class="button button-primary text-sm px-5 py-1.5 h-8 w-24 rounded-full" @click="saveLyrics" title="Save lyrics (Ctrl+S)">Save</button>
            <button v-else class="button button-disabled text-sm px-5 py-1.5 h-8 w-24 rounded-full" title="Save lyrics (Ctrl+S)" disabled>Saved</button>
            <button v-if="!isDirty" class="button button-normal text-sm px-5 py-1.5 h-8 w-24 rounded-full" @click="publishLyrics" title="Publish to LRCLIB service">Publish</button>
            <button v-else class="button button-disabled text-sm px-5 py-1.5 h-8 w-24 rounded-full" title="Publish to LRCLIB service" disabled>Publish</button>
          </div>

          <div class="text-sm text-brave-30">
            {{ editingTrack.title }} - {{ editingTrack.artist_name }}
          </div>

          <button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="close"><Close /></button>
        </div>

        <div class="px-6 py-2 grow overflow-hidden flex flex-col gap-4">
          <div class="flex flex-col bg-brave-95 rounded-lg">
            <div class="toolbar px-4 py-2 flex items-stretch gap-1">
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Sync line & move next (Alt+Enter)" @click="syncLine"><EqualEnter /> <span class="text-xs">Sync Line & Move Next</span></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Sync line (Alt+X)" @click="syncLine(false)"><Equal /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Rewind line 100ms (Alt+LeftArrow)" @click="rewind100"><Minus /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Forward line 100ms (Alt+RightArrow)" @click="fastForward100"><Plus /></button>
              <button class="button button-normal px-3 py-1 text-lg rounded-full" title="Replay line (Alt+Z)" @click="repeatLine"><MotionPlay /></button>
            </div>
            <div class="w-full border-b border-brave-90"></div>
            <div class="flex gap-1 items-center px-4 py-2">
              <button v-if="status !== 'playing'" @click.prevent="resumeOrPlay" class="button button-normal p-2 rounded-full text-xl"><Play /></button>
              <button v-else @click.prevent="pause" class="button button-normal p-2 rounded-full text-xl"><Pause /></button>
              <div class="flex-none w-12 text-xs text-brave-30">{{ humanDuration(progress) }}</div>
              <Seek class="grow" :duration="duration" :progress="progress" @seek="seek" />
              <div class="flex-none w-12 text-xs text-brave-30">{{ humanDuration(duration) }}</div>
            </div>
          </div>

          <div class="h-full grow overflow-hidden">
            <AsyncCodemirror
              v-if="shouldLoadCodeMirror"
              v-model="unifiedLyrics"
              placeholder="Lyrics is currently empty"
              class="codemirror-custom h-full outline-none"
              :autofocus="true"
              :indent-with-tab="true"
              :tab-size="2"
              :extensions="extensions"
              @ready="handleReady"
              @change="lyricsUpdated"
            />

            <div v-else class="flex flex-col h-full items-center justify-center text-sm text-brave-40">
              <div class="animate-spin text-xl text-brave-30"><Loading /></div>
              <div>Loading editor...</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="fixed top-0 left-0 h-full w-full z-20 bg-black/30" :class="{ 'hidden': !props.isShow }" @click="close">
    </div>
  </div>

  <Publish :is-show="isPublishing" :lint-result="lintResult" :track="editingTrack" :lyrics="unifiedLyrics" @close="isPublishing = false" />
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted, onUnmounted, shallowRef, watch } from 'vue'
import { Close, Loading, Equal, Play, Pause, MotionPlay, Minus, Plus } from 'mdue'
import EqualEnter from '@/components/icons/EqualEnter.vue'
import { useToast } from 'vue-toastification'
import { Lrc, Runner, timestampToString, parseLine } from 'lrc-kit'
import { useEditLyrics } from '@/composables/edit-lyrics.js'
import { usePlayer } from '@/composables/player.js'
import Seek from '@/components/now-playing/Seek.vue'
import Publish from './edit-lyrics/Publish.vue'
import { Decoration, EditorView } from '@codemirror/view'
import { StateField, StateEffect } from '@codemirror/state'
import { defineAsyncComponent } from 'vue'
import { executeLint } from '@/utils/lrclint.js'

const AsyncCodemirror = defineAsyncComponent(async () => {
  const { Codemirror } = await import('vue-codemirror')
  return Codemirror
})

const { status, duration, progress, setTrack, playTrack, pause, resume, seek, unload } = usePlayer()
const toast = useToast()
const props = defineProps(['isShow'])
const { editingTrack } = useEditLyrics()
const unifiedLyrics = ref('')
const shouldLoadCodeMirror = ref(false)
const view = shallowRef()
const keydownEvent = ref(null)
const isDirty = ref(false)
const isPublishing = ref(false)
const lintResult = ref([])

const runner = ref(null)
const currentIndex = ref(null)

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

const handleReady = (payload) => {
  view.value = payload.view

  setTimeout(() => {
    view.value.scrollDOM.scrollTop = 0
  }, 100)
}

const lyricsUpdated = (newLyrics) => {
  const parsed = Lrc.parse(newLyrics)
  runner.value = new Runner(parsed)
  isDirty.value = true
  lintResult.value = executeLint(newLyrics)
}

const syncLine = (moveNext = true) => {
  const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
  const currentLineText = currentLine.text
  const stringifiedTime = timestampToString(progress.value)
  const replacedText = currentLineText.replace(/^(\s*\[(.*)\]\s*)*/g, `[${stringifiedTime}] `)

  view.value.dispatch({
    changes: {
      from: currentLine.from,
      to: currentLine.to,
      insert: replacedText
    }
  })

  if (moveNext) {
    const newLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)

    if (newLine.to + 1 < view.value.state.doc.length) {
      view.value.dispatch({
        selection: {
          anchor: newLine.to + 1
        }
      })
    }

    view.value.dispatch({
      effects: EditorView.scrollIntoView(newLine.from, { y: 'center', behavior: 'smooth' })
    })
  }

  view.value.focus()

  lyricsUpdated(view.value.state.doc.toString())
}

const repeatLine = () => {
  const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
  const currentLineText = currentLine.text
  const parsed = parseLine(currentLineText)
  if (parsed.type === 'TIME') {
    seek(parsed.timestamps[0])
  }
}

const rewind100 = () => {
  const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
  const currentLineText = currentLine.text
  const parsed = parseLine(currentLineText)
  if (parsed.type === 'TIME') {
    const timestamp = parsed.timestamps[0]
    const newTimestamp = timestamp - 0.1
    const stringifiedTime = timestampToString(newTimestamp)
    const replacedText = currentLineText.replace(/^(\s*\[(.*)\]\s*)*/g, `[${stringifiedTime}] `)

    view.value.dispatch({
      changes: {
        from: currentLine.from,
        to: currentLine.to,
        insert: replacedText
      }
    })

    view.value.focus()

    lyricsUpdated(view.value.state.doc.toString())

    seek(parsed.timestamps[0])
  }
}

const fastForward100 = () => {
  const currentLine = view.value.state.doc.lineAt(view.value.state.selection.main.head)
  const currentLineText = currentLine.text
  const parsed = parseLine(currentLineText)
  if (parsed.type === 'TIME') {
    const timestamp = parsed.timestamps[0]
    const newTimestamp = timestamp + 0.1
    const stringifiedTime = timestampToString(newTimestamp)
    const replacedText = currentLineText.replace(/^(\s*\[(.*)\]\s*)*/g, `[${stringifiedTime}] `)

    view.value.dispatch({
      changes: {
        from: currentLine.from,
        to: currentLine.to,
        insert: replacedText
      }
    })

    view.value.focus()

    lyricsUpdated(view.value.state.doc.toString())

    seek(parsed.timestamps[0])
  }
}

onUnmounted(async () => {
  if (keydownEvent.value) {
    document.removeEventListener(keydownEvent.value)
  }
})

onMounted(async () => {
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

  unload()
  setTrack(editingTrack.value, false)

  const parsed = Lrc.parse(unifiedLyrics.value)

  runner.value = new Runner(parsed)

  setTimeout(() => shouldLoadCodeMirror.value = true, 100)

  keydownEvent.value = document.addEventListener('keydown', (event) => {
    if (event.altKey === true && event.key === 'Enter') {
      event.preventDefault()
      syncLine()
    } else if (event.altKey === true && event.key === 'x') {
      event.preventDefault()
      syncLine(false)
    } else if (event.altKey === true && event.key === 'z') {
      event.preventDefault()
      repeatLine()
    } else if (event.ctrlKey === true && event.key === 's') {
      event.preventDefault()
      if (isDirty.value) {
        saveLyrics()
      }
    } else if (event.altKey === true && event.key === 'ArrowLeft') {
      event.preventDefault()
      rewind100()
    } else if (event.altKey === true && event.key === 'ArrowRight') {
      event.preventDefault()
      fastForward100()
    }
  })

  lintResult.value = executeLint(unifiedLyrics.value)
})

watch(progress, (newProgress) => {
  if (!editingTrack.value || !view.value) {
    return
  }

  if (!runner.value || !unifiedLyrics.value) {
    return
  }

  runner.value.timeUpdate(newProgress)
  let resultCurrentIndex = runner.value.curIndex()

  if (resultCurrentIndex === null) {
    resultCurrentIndex = -1
  }

  currentIndex.value = resultCurrentIndex

  if (currentIndex.value >= 0) {
    const line = view.value.state.doc.line(currentIndex.value + 1)
    view.value.dispatch({ effects: addLineHighlight.of(line.from) })
  } else {
    view.value.dispatch({ effects: addLineHighlight.of(null) })
  }
})

const close = () => {
  runner.value = null
  editingTrack.value = null
}

const saveLyrics = async () => {
  try {
    await invoke('save_lyrics', { trackId: editingTrack.value.id, plainLyrics: unifiedLyrics.value.replace(/^\[(.*)\] */mg, ''), syncedLyrics: unifiedLyrics.value })
    isDirty.value = false
  } catch (error) {
    console.error(error)
    toast.error(error)
  }
}

const publishLyrics = async () => {
  isPublishing.value = true
}
</script>

<style>
.codemirror-custom .cm-editor {
  @apply outline-none h-full;
}

.cm-scroller {
  @apply scroll-smooth;
}

.codemirror-custom .cm-current-lyrics {
  @apply font-bold;
}

.codemirror-custom .cm-line {
  @apply text-brave-10;
}

.codemirror-custom .cm-activeLine {
  @apply bg-brave-80/30;
}

.codemirror-custom .cm-activeLineGutter {
  @apply bg-brave-80/30;
}

.codemirror-custom .cm-gutters {
  @apply bg-brave-90/20 text-brave-40 border-r border-brave-90;
}
</style>