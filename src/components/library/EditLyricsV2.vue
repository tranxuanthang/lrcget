<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[95vh] max-w-screen-lg"
    :title="modalTitle"
    @close="handleClose"
  >
    <template #titleLeft>
      <EditLyricsV2HeaderActions
        :is-dirty="isDirty"
        :is-exporting="isExporting"
        @save="saveLyrics"
        @save-and-publish="saveAndPublish"
        @export="exportLyrics"
        @debug="openDebugModal"
      />
    </template>

    <template #titleRight>
      <div class="inline-flex gap-0.5">
        <button
          class="button text-sm h-8 w-24 rounded-l-full rounded-r-none"
          :class="
            isInstrumental
              ? 'button-disabled'
              : activeTab === 'plain'
                ? 'button-primary'
                : 'button-normal'
          "
          :disabled="isInstrumental"
          @click="activeTab = 'plain'"
        >
          Plain
        </button>
        <button
          class="button text-sm h-8 w-24 rounded-r-full rounded-l-none"
          :class="
            isInstrumental
              ? 'button-disabled'
              : activeTab === 'synced'
                ? 'button-primary'
                : 'button-normal'
          "
          :disabled="isInstrumental"
          @click="activeTab = 'synced'"
        >
          Synced
        </button>
      </div>
    </template>

    <div class="grow overflow-hidden flex flex-col gap-2 h-full">
      <div class="toolbar bg-brave-95 dark:bg-brave-10 rounded-lg">
        <div class="px-2 py-2">
          <EditLyricsV2PlayerBar
            :status="status"
            :duration="duration"
            :progress="progress"
            @play-toggle="resumeOrPlay"
            @pause="pause"
            @seek="seek"
          />
        </div>
      </div>

      <!-- Instrumental State -->
      <div v-if="isInstrumental" class="absolute bottom-16 left-1/2 -translate-x-1/2 px-3 z-10">
        <div
          class="w-full max-w-lg rounded-lg border border-brave-90 dark:border-brave-20 bg-brave-98 dark:bg-brave-10 p-5 shadow-lg"
        >
          <h3 class="text-base font-semibold">Track is marked as instrumental</h3>
          <div class="mt-4 flex flex-wrap gap-2">
            <button
              class="button button-normal px-2 py-1 text-xs rounded-full"
              @click="setInstrumental(false)"
            >
              Unmark as instrumental
            </button>
          </div>
        </div>
      </div>

      <PlainLyricsCodeEditor
        v-else-if="activeTab === 'plain'"
        :model-value="plainLyrics"
        :font-size="codemirrorStyle.fontSize"
        :synced-lines="syncedLines"
        @update:model-value="updatePlainLyrics"
        @change-font-size="changeCodemirrorFontSizeBy"
        @reset-font-size="resetCodemirrorFontSize"
        @mark-as-instrumental="setInstrumental(true)"
      />

      <SyncedLyricsEditor
        v-else
        :model-value="syncedLines"
        :can-import-from-plain="hasPlainLyrics"
        :selected-line-index="selectedSyncedLineIndex"
        :progress-ms="progressMs"
        @update:model-value="updateSyncedLines"
        @update:selected-line-index="selectSyncedLine"
        @editing-state-change="setSyncedLineEditingState"
        @play-line="playLine"
        @sync-line="syncLineToCurrentProgress"
        @rewind-line="rewindLineBy100"
        @forward-line="forwardLineBy100"
        @sync-end="syncEndToCurrentProgress"
        @rewind-end="rewindEndBy100"
        @forward-end="forwardEndBy100"
        @delete-line="deleteSyncedLine"
        @add-line-at="addSyncedLineAt"
        @import-lines-from-plain="importSyncedLinesFromPlain"
        @import-lrc-file="handleImportLrcFile"
        @paste-lrc="handlePasteLrc"
        @update:words="updateLineWords"
        @word-timing-edited="handleWordTimingEdited"
        @update-line-text="handleUpdateLineText"
        @mark-as-instrumental="setInstrumental(true)"
      />
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, toRef, watch } from 'vue'
import { useToast } from 'vue-toastification'
import { useModal } from 'vue-final-modal'
import BaseModal from '@/components/common/BaseModal.vue'
import ConfirmModal from '@/components/common/ConfirmModal.vue'
import EditLyricsV2DebugModal from '@/components/library/edit-lyrics-v2/EditLyricsV2DebugModal.vue'
import EditLyricsV2HeaderActions from '@/components/library/edit-lyrics-v2/EditLyricsV2HeaderActions.vue'
import EditLyricsV2PlayerBar from '@/components/library/edit-lyrics-v2/EditLyricsV2PlayerBar.vue'
import PlainLyricsCodeEditor from '@/components/library/edit-lyrics-v2/PlainLyricsCodeEditor.vue'
import SyncedLyricsEditor from '@/components/library/edit-lyrics-v2/SyncedLyricsEditor.vue'
import { useEditLyricsV2Document } from '@/composables/edit-lyrics-v2/useEditLyricsV2Document.js'
import { useEditLyricsV2Hotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2Hotkeys.js'
import { useEditLyricsV2Publish } from '@/composables/edit-lyrics-v2/useEditLyricsV2Publish.js'
import { useEditLyricsV2Playback } from '@/composables/edit-lyrics-v2/useEditLyricsV2Playback.js'
import { useEditLyricsV2Export } from '@/composables/edit-lyrics-v2/useEditLyricsV2Export.js'
import { useEditLyricsV2SyncedHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2SyncedHotkeys.js'
import { useGlobalState } from '@/composables/global-state.js'
import { usePlayer } from '@/composables/player.js'
import { open } from '@tauri-apps/plugin-dialog'
import { readText } from '@tauri-apps/plugin-clipboard-manager'
import { invoke } from '@tauri-apps/api/core'
import { parseLrcLines } from '@/utils/lyricsfile.js'

const props = defineProps({
  // Audio source for playback (library track or file-based track)
  // Format: { type: 'library'|'file', id?, file_path?, duration?, title?, artist_name?, album_name?, ... }
  audioSource: {
    type: Object,
    required: true,
  },
  // Lyricsfile object for editing operations (save, debug, publish)
  // Format: { id?, content, metadata?: { title, artist, album, duration_ms } }
  // For library tracks, id is null and content comes from track.lyricsfile
  // For standalone lyricsfiles, id is the lyricsfiles table record ID
  lyricsfile: {
    type: Object,
    default: null,
  },
  // Track ID for save operations. Set for library tracks, null for temporary associations
  // This is separate from audioSource to handle the case where a library track is temporarily
  // associated with a standalone lyricsfile (e.g., LRCLIB Browser flow)
  trackId: {
    type: Number,
    default: null,
  },
})

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const toast = useToast()

// Convert props to refs for composables
const audioSourceRef = toRef(props, 'audioSource')
const lyricsfileRef = toRef(props, 'lyricsfile')
const trackIdRef = toRef(props, 'trackId')

const progressMs = computed(() => Math.max(0, Math.round(progress.value * 1000)))

const activeTab = ref('plain')
const {
  plainLyrics,
  syncedLines,
  lyricsfileDocument,
  isDirty,
  selectedSyncedLineIndex,
  isSyncedLineEditing,
  hasPlainLyrics,
  selectedLineExists,
  isInstrumental,
  serializedLyricsfile,
  initializeLyrics,
  updatePlainLyrics,
  updateSyncedLines,
  selectSyncedLine,
  setSyncedLineEditingState,
  addSyncedLineAt,
  deleteSyncedLine,
  importSyncedLinesFromPlain,
  syncLineToCurrentProgress,
  rewindLineBy100: rewindLineTimestampBy100,
  forwardLineBy100: forwardLineTimestampBy100,
  syncEndToCurrentProgress,
  rewindEndBy100,
  forwardEndBy100,
  saveLyrics,
  ensureSelectedSyncedLine,
  updateLineText,
  setInstrumental,
} = useEditLyricsV2Document({
  audioSource: audioSourceRef,
  lyricsfile: lyricsfileRef,
  trackId: trackIdRef,
  progress,
  toast,
})

const codemirrorStyle = ref({
  fontSize: 1.0,
})

const { saveAndPublish } = useEditLyricsV2Publish({
  audioSource: audioSourceRef,
  lyricsfileDocument: lyricsfileDocument,
  serializedLyricsfile,
  saveLyrics,
})

const { exportLyrics, isExporting } = useEditLyricsV2Export({
  audioSource: audioSourceRef,
  saveLyrics,
  serializedLyricsfile,
  toast,
})

const { playLine, resumeOrPlay } = useEditLyricsV2Playback({
  audioSource: audioSourceRef,
  syncedLines,
  progress,
  playingTrack,
  status,
  playTrack,
  resume,
  seek,
})

const rewindLineBy100 = lineIndex => {
  rewindLineTimestampBy100(lineIndex)
  void playLine(lineIndex)
}

const forwardLineBy100 = lineIndex => {
  forwardLineTimestampBy100(lineIndex)
  void playLine(lineIndex)
}

const updateLineWords = ({ lineIndex, words, lineStartMs }) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
    return
  }

  const nextLineStartMs = Number.isFinite(lineStartMs) ? Math.max(0, Math.round(lineStartMs)) : null

  const newLines = syncedLines.value.map((line, index) => {
    if (index !== lineIndex) {
      return line
    }

    return {
      ...line,
      ...(nextLineStartMs === null ? {} : { start_ms: nextLineStartMs }),
      words,
    }
  })

  updateSyncedLines(newLines)
}

const handleUpdateLineText = (lineIndex, newText) => {
  updateLineText(lineIndex, newText)
}

const handleImportLrcFile = async () => {
  try {
    const filePath = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: 'LRC Files', extensions: ['lrc'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    })

    if (!filePath) {
      return
    }

    const content = await invoke('read_text_file', { filePath })
    const parsedLines = parseLrcLines(content)

    if (parsedLines.length === 0) {
      toast.error('No valid synced lines found in the selected file')
      return
    }

    updateSyncedLines(parsedLines)
    toast.success(`Imported ${parsedLines.length} synced lines`)
  } catch (error) {
    console.error(error)
    toast.error(error?.toString?.() || 'Failed to import LRC file')
  }
}

const handlePasteLrc = async () => {
  try {
    const text = await readText()
    if (!text || !text.trim()) {
      toast.error('Clipboard is empty')
      return
    }

    const parsedLines = parseLrcLines(text)

    if (parsedLines.length === 0) {
      toast.error('No valid synced lines found in clipboard')
      return
    }

    updateSyncedLines(parsedLines)
    toast.success(`Imported ${parsedLines.length} synced lines`)
  } catch (error) {
    console.error(error)
    toast.error(error?.toString?.() || 'Failed to paste LRC from clipboard')
  }
}

const handleWordTimingEdited = async ({ startMs }) => {
  // Auto-replay from the beginning of the edited line for instant verification
  const seekTo = Number.isFinite(startMs) ? startMs / 1000 : progress.value

  // Ensure we're playing the correct audio source
  const isPlayingCorrectTrack =
    audioSourceRef.value.type === 'library'
      ? playingTrack.value?.id === audioSourceRef.value.id
      : playingTrack.value?.file_path === audioSourceRef.value.file_path

  if (!playingTrack.value || !isPlayingCorrectTrack) {
    await playTrack(audioSourceRef.value)
  } else if (status.value === 'paused') {
    resume()
  }

  seek(seekTo)
}

watch(activeTab, value => {
  if (value !== 'synced') {
    isSyncedLineEditing.value = false
    return
  }

  ensureSelectedSyncedLine()
})

const { bindSyncedHotkeys, unbindSyncedHotkeys } = useEditLyricsV2SyncedHotkeys({
  activeTab,
  isSyncedLineEditing,
  selectedLineExists,
  selectedSyncedLineIndex,
  syncedLines,
  selectSyncedLine,
  syncLineToCurrentProgress,
  rewindLineBy100: rewindLineTimestampBy100,
  forwardLineBy100: forwardLineTimestampBy100,
})

const changeCodemirrorFontSizeBy = offset => {
  const nextFontSize = Math.max(0.4, codemirrorStyle.value.fontSize + offset * 0.1)
  codemirrorStyle.value.fontSize = +nextFontSize.toFixed(2)
}

const resetCodemirrorFontSize = () => {
  codemirrorStyle.value.fontSize = 1.0
}

const debugModalContent = computed(() => {
  return serializedLyricsfile.value || ''
})

const { open: openDebugModal, close: closeDebugModal } = useModal({
  component: EditLyricsV2DebugModal,
  attrs: {
    content: debugModalContent,
    onClose() {
      closeDebugModal()
    },
  },
})

const { open: openConfirmModal, close: closeConfirmModal } = useModal({
  component: ConfirmModal,
  attrs: {
    title: 'Unsaved Changes',
    message: 'You have unsaved changes. Are you sure you want to close?',
    confirmText: 'Discard Changes',
    cancelText: 'Cancel',
    onConfirm() {
      closeConfirmModal()
      emit('close')
    },
    onCancel() {
      closeConfirmModal()
    },
  },
})

const handleClose = () => {
  if (isDirty.value) {
    openConfirmModal()
  } else {
    emit('close')
  }
}

const modalTitle = computed(() => {
  const title =
    audioSourceRef.value?.title || lyricsfileRef.value?.metadata?.title || 'Unknown Title'
  const artist =
    audioSourceRef.value?.artist_name || lyricsfileRef.value?.metadata?.artist || 'Unknown Artist'
  return `${title} - ${artist}`
})

const { bindHotkeys, unbindHotkeys } = useEditLyricsV2Hotkeys({
  activeTab,
  saveLyrics,
  changeFontSizeBy: changeCodemirrorFontSizeBy,
  resetFontSize: resetCodemirrorFontSize,
})

onMounted(() => {
  disableHotkey()

  // Initialize lyrics from props
  initializeLyrics()

  // Handle playback - ensure correct audio source is loaded
  const isPlayingCorrectTrack =
    audioSourceRef.value?.type === 'library'
      ? playingTrack.value?.id === audioSourceRef.value?.id
      : playingTrack.value?.file_path === audioSourceRef.value?.file_path

  if (!playingTrack.value || !isPlayingCorrectTrack) {
    playTrack(audioSourceRef.value)
    pause()
  }

  bindHotkeys()
  bindSyncedHotkeys()
})

onUnmounted(() => {
  unbindSyncedHotkeys()
  unbindHotkeys()
  enableHotkey()
})
</script>
