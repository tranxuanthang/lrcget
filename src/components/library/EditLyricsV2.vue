<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[95vh] max-w-screen-lg"
    :title="modalTitle"
    @close="emit('close')"
  >
    <template #titleLeft>
      <EditLyricsV2HeaderActions :is-dirty="isDirty" @save="saveLyrics" @debug="openDebugModal" />
    </template>

    <template #titleRight>
      <div class="inline-flex gap-0.5">
        <button
          class="button text-sm h-8 w-24 rounded-l-full rounded-r-none"
          :class="activeTab === 'plain' ? 'button-primary' : 'button-normal'"
          @click="activeTab = 'plain'"
        >
          Plain
        </button>
        <button
          class="button text-sm h-8 w-24 rounded-r-full rounded-l-none"
          :class="activeTab === 'synced' ? 'button-primary' : 'button-normal'"
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
import { useEditLyricsV2Playback } from '@/composables/edit-lyrics-v2/useEditLyricsV2Playback.js'
import { useEditLyricsV2SyncedHotkeys } from '@/composables/edit-lyrics-v2/useEditLyricsV2SyncedHotkeys.js'
import { useGlobalState } from '@/composables/global-state.js'
import { usePlayer } from '@/composables/player.js'
import { serializeLyricsfile } from '@/utils/lyricsfile.js'

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const { editingTrack } = useEditLyricsV2()
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
  updateLineText
} = useEditLyricsV2Document({ editingTrack, progress, toast })

const codemirrorStyle = ref({
  fontSize: 1.0
})

const { playLine, resumeOrPlay } = useEditLyricsV2Playback({
  editingTrack,
  syncedLines,
  progress,
  playingTrack,
  status,
  playTrack,
  resume,
  seek
})

const rewindLineBy100 = (lineIndex) => {
  rewindLineTimestampBy100(lineIndex)
  void playLine(lineIndex)
}

const forwardLineBy100 = (lineIndex) => {
  forwardLineTimestampBy100(lineIndex)
  void playLine(lineIndex)
}

const updateLineWords = ({ lineIndex, words, lineStartMs }) => {
  if (!Number.isInteger(lineIndex) || lineIndex < 0 || lineIndex >= syncedLines.value.length) {
    return
  }

  const nextLineStartMs = Number.isFinite(lineStartMs)
    ? Math.max(0, Math.round(lineStartMs))
    : null

  const newLines = syncedLines.value.map((line, index) => {
    if (index !== lineIndex) {
      return line
    }

    return {
      ...line,
      ...(nextLineStartMs === null ? {} : { start_ms: nextLineStartMs }),
      words
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
  if (!playingTrack.value || playingTrack.value.id !== editingTrack.value?.id) {
    await playTrack(editingTrack.value)
  } else if (status.value === 'paused') {
    resume()
  }

  seek(seekTo)
}

watch(activeTab, (value) => {
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
  forwardLineBy100: forwardLineTimestampBy100
})

const changeCodemirrorFontSizeBy = (offset) => {
  const nextFontSize = Math.max(0.4, codemirrorStyle.value.fontSize + offset * 0.1)
  codemirrorStyle.value.fontSize = +nextFontSize.toFixed(2)
}

const resetCodemirrorFontSize = () => {
  codemirrorStyle.value.fontSize = 1.0
}

const debugModalContent = computed(() => {
  if (!editingTrack.value) return ''
  return serializeLyricsfile({
    track: editingTrack.value,
    plainLyrics: plainLyrics.value,
    syncedLines: syncedLines.value,
    baseDocument: lyricsfileDocument.value
  }) || ''
})

const { open: openDebugModal, close: closeDebugModal } = useModal({
  component: EditLyricsV2DebugModal,
  attrs: {
    content: debugModalContent,
    onClose() {
      closeDebugModal()
    }
  }
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
  resetFontSize: resetCodemirrorFontSize
})

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
  bindSyncedHotkeys()
})

onUnmounted(() => {
  unbindSyncedHotkeys()
  unbindHotkeys()
  enableHotkey()
})
</script>
