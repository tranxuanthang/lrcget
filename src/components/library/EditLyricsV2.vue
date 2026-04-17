<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[95vh] max-w-screen-lg"
    :title="modalTitle"
    @close="emit('close')"
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
        :playing-line-index="currentPlayingSyncedLineIndex"
        :progress-ms="progressMs"
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
        @update:words="updateLineWords"
        @word-timing-edited="handleWordTimingEdited"
        @update-line-text="handleUpdateLineText"
        @mark-as-instrumental="setInstrumental(true)"
      />
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useToast } from 'vue-toastification'
import { useModal } from 'vue-final-modal'
import BaseModal from '@/components/common/BaseModal.vue'
import EditLyricsV2DebugModal from '@/components/library/edit-lyrics-v2/EditLyricsV2DebugModal.vue'
import EditLyricsV2HeaderActions from '@/components/library/edit-lyrics-v2/EditLyricsV2HeaderActions.vue'
import EditLyricsV2PlayerBar from '@/components/library/edit-lyrics-v2/EditLyricsV2PlayerBar.vue'
import PlainLyricsCodeEditor from '@/components/library/edit-lyrics-v2/PlainLyricsCodeEditor.vue'
import SyncedLyricsEditor from '@/components/library/edit-lyrics-v2/SyncedLyricsEditor.vue'
import { useEditLyricsV2 } from '@/composables/edit-lyrics-v2.js'
import { useEditLyricsV2Document } from '@/composables/edit-lyrics-v2/useEditLyricsV2Document.js'
import { useEditLyricsV2Hotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2Hotkeys.js'
import { useEditLyricsV2Publish } from '@/composables/edit-lyrics-v2/useEditLyricsV2Publish.js'
import { useEditLyricsV2Playback } from '@/composables/edit-lyrics-v2/useEditLyricsV2Playback.js'
import { useEditLyricsV2Export } from '@/composables/edit-lyrics-v2/useEditLyricsV2Export.js'
import { useEditLyricsV2SyncedHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2SyncedHotkeys.js'
import { useGlobalState } from '@/composables/global-state.js'
import { usePlayer } from '@/composables/player.js'

const props = defineProps({
  track: {
    type: Object,
    default: null,
  },
  lyricsfileId: {
    type: Number,
    default: null,
  },
  initialLyricsfile: {
    type: String,
    default: null,
  },
})

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const { editingTrack, setEditingTrack } = useEditLyricsV2()
const toast = useToast()

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
  currentPlayingSyncedLineIndex,
  isInstrumental,
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
  saveLyrics,
  ensureSelectedSyncedLine,
  updateLineText,
  setInstrumental,
} = useEditLyricsV2Document({ editingTrack, progress, toast, lyricsfileId: props.lyricsfileId })

const codemirrorStyle = ref({
  fontSize: 1.0,
})

const { saveAndPublish, serializedLyricsfile } = useEditLyricsV2Publish({
  editingTrack,
  plainLyrics,
  syncedLines,
  lyricsfileDocument,
  isInstrumental,
  saveLyrics,
})

const { exportLyrics, isExporting } = useEditLyricsV2Export({
  editingTrack,
  saveLyrics,
  serializedLyricsfile,
  toast,
})

const { playLine, resumeOrPlay } = useEditLyricsV2Playback({
  editingTrack,
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

const handleWordTimingEdited = async ({ startMs }) => {
  // Auto-replay from the beginning of the edited line for instant verification
  const seekTo = Number.isFinite(startMs) ? startMs / 1000 : progress.value

  // Ensure we're playing the editing track
  const isLibraryTrack = !!editingTrack.value?.id
  const isPlayingCorrectTrack = isLibraryTrack
    ? playingTrack.value?.id === editingTrack.value?.id
    : playingTrack.value?.file_path === editingTrack.value?.file_path

  if (!playingTrack.value || !isPlayingCorrectTrack) {
    await playTrack(editingTrack.value)
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
  if (!editingTrack.value) return ''
  return serializedLyricsfile.value
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

const modalTitle = computed(() => {
  if (!editingTrack.value) {
    return 'Edit lyrics (v2)'
  }

  return `${editingTrack.value.title} - ${editingTrack.value.artist_name}`
})

const { bindHotkeys, unbindHotkeys } = useEditLyricsV2Hotkeys({
  activeTab,
  saveLyrics,
  changeFontSizeBy: changeCodemirrorFontSizeBy,
  resetFontSize: resetCodemirrorFontSize,
})

onMounted(() => {
  disableHotkey()

  // Set the editing track from props
  if (props.track) {
    setEditingTrack(props.track)
  }

  if (!editingTrack.value) {
    return
  }

  // Initialize lyrics - use initialLyricsfile from props if available
  initializeLyrics(props.initialLyricsfile)

  // Handle playback
  const isLibraryTrack = !!editingTrack.value.id
  if (isLibraryTrack) {
    // Library track: check by ID
    if (!playingTrack.value || playingTrack.value.id !== editingTrack.value.id) {
      playTrack(editingTrack.value)
      pause()
    }
  } else if (editingTrack.value.file_path) {
    // File-based track: always play since it's a new file
    playTrack(editingTrack.value)
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
