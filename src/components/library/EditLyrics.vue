<template>
  <BaseModal
    :click-to-close="false"
    :esc-to-close="false"
    content-class="w-full h-[80vh] max-w-screen-lg"
    :title="modalTitle"
    @close="emit('close')"
  >
    <template #titleLeft>
      <EditLyricsHeaderActions
        :is-dirty="isDirty"
        :publish-button-tooltip="publishButtonTooltip"
        :publish-status="publishStatus"
        :publish-status-tooltip="publishStatusTooltip"
        @save="saveLyrics"
        @publish="openPublishModal"
      />
    </template>

    <div class="grow overflow-hidden flex flex-col gap-2 h-full">
      <div class="toolbar flex flex-col bg-brave-95 dark:bg-brave-10 rounded-lg">
        <EditLyricsSyncToolbar
          @sync-line="syncLine"
          @sync-line-only="syncLine(false)"
          @rewind="rewind100"
          @forward="fastForward100"
          @repeat="repeatLine"
          @mark-instrumental="markAsInstrumental"
        />
        <div class="w-full border-b border-brave-90 dark:border-brave-20" />
        <EditLyricsPlayerBar
          :status="status"
          :duration="duration"
          :progress="progress"
          @play-toggle="resumeOrPlay"
          @pause="pause"
          @seek="seek"
        />
      </div>

      <LyricsCodeEditor
        :model-value="unifiedLyrics"
        :extensions="lineHighlightExtensions"
        :font-size="codemirrorStyle.fontSize"
        @update:model-value="updateLyrics"
        @ready="handleReady"
        @change-font-size="changeCodemirrorFontSizeBy"
        @reset-font-size="resetCodemirrorFontSize"
      />
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, shallowRef } from 'vue'
import { EditorView } from '@codemirror/view'
import BaseModal from '@/components/common/BaseModal.vue'
import EditLyricsHeaderActions from '@/components/library/edit-lyrics/EditLyricsHeaderActions.vue'
import EditLyricsSyncToolbar from '@/components/library/edit-lyrics/EditLyricsSyncToolbar.vue'
import EditLyricsPlayerBar from '@/components/library/edit-lyrics/EditLyricsPlayerBar.vue'
import LyricsCodeEditor from '@/components/library/edit-lyrics/LyricsCodeEditor.vue'
import {
  addLineHighlight,
  lineHighlightExtensions,
} from '@/components/library/edit-lyrics/codemirror-line-highlight.js'
import { useToast } from 'vue-toastification'
import { useEditLyrics } from '@/composables/edit-lyrics.js'
import { useGlobalState } from '@/composables/global-state.js'
import { usePlayer } from '@/composables/player.js'
import { useLyricsDocument } from '@/composables/edit-lyrics/useLyricsDocument.js'
import { useLyricsEditorCommands } from '@/composables/edit-lyrics/useLyricsEditorCommands.js'
import { useLyricsEditorHotkeys } from '@/composables/edit-lyrics/useLyricsEditorHotkeys.js'
import { useLyricsPlaybackSync } from '@/composables/edit-lyrics/useLyricsPlaybackSync.js'
import { useLyricsPublish } from '@/composables/edit-lyrics/useLyricsPublish.js'

const emit = defineEmits(['close'])

const { disableHotkey, enableHotkey } = useGlobalState()
const { status, duration, progress, playingTrack, playTrack, pause, resume, seek } = usePlayer()
const { editingTrack } = useEditLyrics()
const toast = useToast()

const view = shallowRef(null)
const codemirrorStyle = ref({
  fontSize: 1.0,
})

const {
  unifiedLyrics,
  isDirty,
  lyricsLintResult,
  plainTextLintResult,
  initializeLyrics,
  updateLyrics,
  saveLyrics,
} = useLyricsDocument({ editingTrack, toast })

const { publishStatus, publishButtonTooltip, publishStatusTooltip, openPublishModal } =
  useLyricsPublish({
    editingTrack,
    unifiedLyrics,
    lyricsLintResult,
    plainTextLintResult,
  })

const { resetPlaybackSync } = useLyricsPlaybackSync({
  progress,
  unifiedLyrics,
  view,
  addLineHighlight,
})

const { syncLine, repeatLine, rewind100, fastForward100, markAsInstrumental } =
  useLyricsEditorCommands({
    view,
    unifiedLyrics,
    progress,
    duration,
    seek,
    onLyricsChange: lyrics => updateLyrics(lyrics),
  })

const changeCodemirrorFontSizeBy = offset => {
  const nextFontSize = Math.max(0.4, codemirrorStyle.value.fontSize + offset * 0.1)
  codemirrorStyle.value.fontSize = +nextFontSize.toFixed(2)
}

const resetCodemirrorFontSize = () => {
  codemirrorStyle.value.fontSize = 1.0
}

const handleReady = ({ view: editorView }) => {
  view.value = editorView

  view.value.dispatch({
    effects: EditorView.scrollIntoView(0),
  })
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
    return 'Edit lyrics'
  }

  return `${editingTrack.value.title} - ${editingTrack.value.artist_name}`
})

const hotkeyConfig = [
  { keys: 'Alt+Enter', handler: () => syncLine() },
  { keys: 'Alt+X', handler: () => syncLine(false) },
  { keys: 'Alt+Z', handler: () => repeatLine() },
  { keys: 'Ctrl+S', handler: () => saveLyrics() },
  { keys: 'Alt+K', handler: () => rewind100() },
  { keys: 'Alt+J', handler: () => fastForward100() },
  { keys: 'Ctrl+Plus', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+=', handler: () => changeCodemirrorFontSizeBy(+1) },
  { keys: 'Ctrl+-', handler: () => changeCodemirrorFontSizeBy(-1) },
  { keys: 'Ctrl+_', handler: () => changeCodemirrorFontSizeBy(-1) },
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
})

onUnmounted(() => {
  resetPlaybackSync()
  unbindHotkeys()
  enableHotkey()
})
</script>
