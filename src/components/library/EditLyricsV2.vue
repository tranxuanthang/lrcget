<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[80vh] max-w-screen-lg"
    :title="modalTitle"
    @close="emit('close')"
  >
    <template #titleLeft>
      <EditLyricsV2HeaderActions :is-dirty="isDirty" @save="saveLyrics" />
    </template>

    <div class="grow overflow-hidden flex flex-col gap-2 h-full">
      <div class="toolbar bg-brave-95 dark:bg-brave-10 rounded-lg">
        <div class="px-2 pt-2">
          <div class="inline-flex gap-1 p-1 rounded-full bg-brave-90 dark:bg-brave-20">
            <button
              class="button text-sm px-3 py-1 h-8 rounded-full"
              :class="activeTab === 'plain' ? 'button-primary' : 'button-normal'"
              @click="activeTab = 'plain'"
            >
              Plain lyrics
            </button>
            <button
              class="button text-sm px-3 py-1 h-8 rounded-full"
              :class="activeTab === 'synced' ? 'button-primary' : 'button-normal'"
              @click="activeTab = 'synced'"
            >
              Synced lyrics
            </button>
          </div>
        </div>

        <EditLyricsV2PlayerBar
          :status="status"
          :duration="duration"
          :progress="progress"
          @play-toggle="resumeOrPlay"
          @pause="pause"
          @seek="seek"
        />
      </div>

      <PlainLyricsCodeEditor
        v-if="activeTab === 'plain'"
        :model-value="plainLyrics"
        :font-size="codemirrorStyle.fontSize"
        @update:model-value="updatePlainLyrics"
        @change-font-size="changeCodemirrorFontSizeBy"
        @reset-font-size="resetCodemirrorFontSize"
      />

      <SyncedLyricsEditor
        v-else
        :model-value="syncedLines"
        :can-import-from-plain="hasPlainLyrics"
        :selected-line-index="selectedSyncedLineIndex"
        :playing-line-index="currentPlayingSyncedLineIndex"
        @update:model-value="updateSyncedLines"
        @update:selected-line-index="selectSyncedLine"
        @editing-state-change="setSyncedLineEditingState"
        @play-line="playLine"
        @sync-line="syncLineToCurrentProgress"
        @rewind-line="rewindLineBy100"
        @forward-line="forwardLineBy100"
        @delete-line="deleteSyncedLine"
        @add-line-at="addSyncedLineAt"
        @import-lines-from-plain="importSyncedLinesFromPlain"
      />
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'vue-toastification'
import BaseModal from '@/components/common/BaseModal.vue'
import EditLyricsV2HeaderActions from '@/components/library/edit-lyrics-v2/EditLyricsV2HeaderActions.vue'
import EditLyricsV2PlayerBar from '@/components/library/edit-lyrics-v2/EditLyricsV2PlayerBar.vue'
import PlainLyricsCodeEditor from '@/components/library/edit-lyrics-v2/PlainLyricsCodeEditor.vue'
import SyncedLyricsEditor from '@/components/library/edit-lyrics-v2/SyncedLyricsEditor.vue'
import { useEditLyricsV2 } from '@/composables/edit-lyrics-v2.js'
import { useGlobalState } from '@/composables/global-state.js'
import { usePlayer } from '@/composables/player.js'
import { useLyricsEditorHotkeys } from '@/composables/edit-lyrics/useLyricsEditorHotkeys.js'
import { createSyncedLinesFromPlain, parseLyricsfile, serializeLyricsfile } from '@/utils/lyricsfile.js'

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const { editingTrack } = useEditLyricsV2()
const toast = useToast()

const activeTab = ref('plain')
const plainLyrics = ref('')
const syncedLines = ref([])
const lyricsfileDocument = ref(null)
const isDirty = ref(false)
const selectedSyncedLineIndex = ref(-1)
const isSyncedLineEditing = ref(false)
const codemirrorStyle = ref({
  fontSize: 1.0
})

const ensureSelectedSyncedLine = () => {
  if (syncedLines.value.length === 0) {
    selectedSyncedLineIndex.value = -1
    return
  }

  if (
    !Number.isInteger(selectedSyncedLineIndex.value)
    || selectedSyncedLineIndex.value < 0
    || selectedSyncedLineIndex.value >= syncedLines.value.length
  ) {
    selectedSyncedLineIndex.value = 0
  }
}

const selectSyncedLine = (lineIndex) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
    return
  }

  selectedSyncedLineIndex.value = lineIndex
}

const setSyncedLineEditingState = (value) => {
  isSyncedLineEditing.value = value
}

const initializeLyrics = () => {
  const track = editingTrack.value
  if (!track) {
    plainLyrics.value = ''
    syncedLines.value = []
    lyricsfileDocument.value = null
    isDirty.value = false
    return
  }

  const parsed = parseLyricsfile(track.lyricsfile)

  plainLyrics.value = parsed.plainLyrics
  syncedLines.value = createSyncedLinesFromPlain(parsed.plainLyrics, parsed.syncedLines)
  lyricsfileDocument.value = parsed.document
  isDirty.value = false
  isSyncedLineEditing.value = false
  ensureSelectedSyncedLine()
}

const updatePlainLyrics = (lyrics) => {
  plainLyrics.value = lyrics
  syncedLines.value = createSyncedLinesFromPlain(lyrics, syncedLines.value)
  isDirty.value = true
  ensureSelectedSyncedLine()
}

const updateSyncedLines = (lines) => {
  syncedLines.value = lines
  isDirty.value = true
  ensureSelectedSyncedLine()
}

const createEmptySyncedLine = () => ({
  text: '',
  words: []
})

const addSyncedLineAt = (lineIndex) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex > syncedLines.value.length) {
    return
  }

  const nextLines = [...syncedLines.value]
  nextLines.splice(lineIndex, 0, createEmptySyncedLine())

  syncedLines.value = nextLines
  selectedSyncedLineIndex.value = lineIndex
  isDirty.value = true
}

const deleteSyncedLine = (lineIndex) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
    return
  }

  syncedLines.value = syncedLines.value.filter((_, index) => index !== lineIndex)
  isDirty.value = true

  if (syncedLines.value.length === 0) {
    selectedSyncedLineIndex.value = -1
    return
  }

  selectedSyncedLineIndex.value = Math.min(lineIndex, syncedLines.value.length - 1)
}

const importSyncedLinesFromPlain = () => {
  if (!hasPlainLyrics.value) {
    return
  }

  syncedLines.value = createSyncedLinesFromPlain(plainLyrics.value, [])
  isDirty.value = true
  ensureSelectedSyncedLine()
}

const withUpdatedLine = (lineIndex, updater) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
    return
  }

  const nextLines = syncedLines.value.map((line, index) => {
    if (index !== lineIndex) {
      return line
    }

    return updater(line)
  })

  syncedLines.value = nextLines
  isDirty.value = true
}

const playLine = async (lineIndex) => {
  if (!editingTrack.value) {
    return
  }

  const lineStartMs = syncedLines.value[lineIndex]?.start_ms
  const seekTo = Number.isFinite(lineStartMs) ? lineStartMs / 1000 : progress.value

  if (!playingTrack.value || playingTrack.value.id !== editingTrack.value.id) {
    await playTrack(editingTrack.value)
  } else if (status.value === 'paused') {
    resume()
  }

  seek(seekTo)
}

const syncLineToCurrentProgress = (lineIndex) => {
  withUpdatedLine(lineIndex, (line) => ({
    ...line,
    start_ms: Math.max(0, Math.round(progress.value * 1000))
  }))
}

const shiftLineTimestampBy = (lineIndex, offsetMs) => {
  withUpdatedLine(lineIndex, (line) => ({
    ...line,
    start_ms: Math.max(0, Math.round((line.start_ms || 0) + offsetMs))
  }))
}

const rewindLineBy100 = (lineIndex) => {
  shiftLineTimestampBy(lineIndex, -100)
  void playLine(lineIndex)
}

const forwardLineBy100 = (lineIndex) => {
  shiftLineTimestampBy(lineIndex, 100)
  void playLine(lineIndex)
}

const saveLyrics = async () => {
  if (!editingTrack.value || !isDirty.value) {
    return
  }

  try {
    const lyricsfile = serializeLyricsfile({
      track: editingTrack.value,
      plainLyrics: plainLyrics.value,
      syncedLines: syncedLines.value,
      baseDocument: lyricsfileDocument.value
    })

    await invoke('save_lyrics', {
      trackId: editingTrack.value.id,
      lyricsfile
    })

    const parsed = parseLyricsfile(lyricsfile)
    syncedLines.value = createSyncedLinesFromPlain(parsed.plainLyrics, parsed.syncedLines)
    lyricsfileDocument.value = parsed.document
    isDirty.value = false
  } catch (error) {
    console.error(error)
    toast.error(error)
  }
}

const hasPlainLyrics = computed(() => plainLyrics.value.trim().length > 0)

const selectedLineExists = computed(() => (
  Number.isInteger(selectedSyncedLineIndex.value)
  && selectedSyncedLineIndex.value >= 0
  && selectedSyncedLineIndex.value < syncedLines.value.length
))

const currentPlayingSyncedLineIndex = computed(() => {
  if (!Number.isFinite(progress.value) || syncedLines.value.length === 0) {
    return -1
  }

  const progressMs = Math.max(0, Math.round(progress.value * 1000))

  for (let index = syncedLines.value.length - 1; index >= 0; index -= 1) {
    const startMs = syncedLines.value[index]?.start_ms
    if (Number.isFinite(startMs) && startMs <= progressMs) {
      return index
    }
  }

  return -1
})

watch(syncedLines, () => {
  ensureSelectedSyncedLine()
}, { deep: true })

watch(activeTab, (value) => {
  if (value !== 'synced') {
    isSyncedLineEditing.value = false
    return
  }

  ensureSelectedSyncedLine()
})

const isKeyboardTargetEditable = (event) => {
  const element = event.target

  if (!(element instanceof HTMLElement)) {
    return false
  }

  const tag = element.tagName.toLowerCase()
  return element.isContentEditable || tag === 'input' || tag === 'textarea' || tag === 'select'
}

const handleSyncedEditorKeyboardShortcuts = (event) => {
  if (
    activeTab.value !== 'synced'
    || isSyncedLineEditing.value
    || !selectedLineExists.value
    || isKeyboardTargetEditable(event)
  ) {
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    selectSyncedLine(Math.max(0, selectedSyncedLineIndex.value - 1))
    return
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
    return
  }

  if (event.key === ' ') {
    event.preventDefault()
    syncLineToCurrentProgress(selectedSyncedLineIndex.value)
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    syncLineToCurrentProgress(selectedSyncedLineIndex.value)
    selectSyncedLine(Math.min(syncedLines.value.length - 1, selectedSyncedLineIndex.value + 1))
    return
  }

  if (event.key === 'ArrowLeft') {
    event.preventDefault()
    rewindLineBy100(selectedSyncedLineIndex.value)
    return
  }

  if (event.key === 'ArrowRight') {
    event.preventDefault()
    forwardLineBy100(selectedSyncedLineIndex.value)
  }
}

const changeCodemirrorFontSizeBy = (offset) => {
  const nextFontSize = Math.max(0.4, codemirrorStyle.value.fontSize + offset * 0.1)
  codemirrorStyle.value.fontSize = +nextFontSize.toFixed(2)
}

const resetCodemirrorFontSize = () => {
  codemirrorStyle.value.fontSize = 1.0
}

const resumeOrPlay = () => {
  if (status.value === 'paused') {
    resume()
    return
  }

  if (editingTrack.value) {
    playTrack(editingTrack.value)
  }
}

const modalTitle = computed(() => {
  if (!editingTrack.value) {
    return 'Edit lyrics (v2)'
  }

  return `${editingTrack.value.title} - ${editingTrack.value.artist_name}`
})

const hotkeyConfig = [
  { keys: 'Ctrl+S', handler: () => saveLyrics() },
  { keys: 'Ctrl+Plus', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+=', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+-', handler: () => changeCodemirrorFontSizeBy(-1) },
  { keys: 'Ctrl+_', handler: () => changeCodemirrorFontSizeBy(-1) }
]

const { bindHotkeys, unbindHotkeys } = useLyricsEditorHotkeys(hotkeyConfig)

onMounted(() => {
  disableHotkey()

  if (!editingTrack.value) {
    return
  }

  initializeLyrics()

  if (!playingTrack.value || playingTrack.value.id !== editingTrack.value.id) {
    playTrack(editingTrack.value)
    pause()
  }

  bindHotkeys()
  document.addEventListener('keydown', handleSyncedEditorKeyboardShortcuts)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleSyncedEditorKeyboardShortcuts)
  unbindHotkeys()
  enableHotkey()
})
</script>
