import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function buildPlaybackSource(audioSource, audioMode, vocalsPath) {
  if (audioMode === 'vocals' && vocalsPath) {
    return {
      type: 'file',
      file_path: vocalsPath,
      title: audioSource?.title,
      album_name: audioSource?.album_name,
      artist_name: audioSource?.artist_name,
      album_artist_name: audioSource?.album_artist_name,
      duration: audioSource?.duration,
    }
  }

  return audioSource
}

export function useEditLyricsV2VocalStem({ audioSource, toast }) {
  const vocalsPath = ref(null)
  const isSeparating = ref(false)
  const audioMode = ref('song')

  const canSeparateVocals = computed(() => Boolean(audioSource.value?.file_path))
  const hasVocalStem = computed(() => Boolean(vocalsPath.value))
  const playbackSource = computed(() =>
    buildPlaybackSource(audioSource.value, audioMode.value, vocalsPath.value)
  )
  const spectrogramFilePath = computed(() => playbackSource.value?.file_path ?? null)

  const refreshVocalStem = async () => {
    const filePath = audioSource.value?.file_path
    if (!filePath) {
      vocalsPath.value = null
      if (audioMode.value === 'vocals') {
        audioMode.value = 'song'
      }
      return
    }

    try {
      const existingPath = await invoke('get_vocal_stem', { filePath })
      vocalsPath.value = existingPath || null
      if (!vocalsPath.value && audioMode.value === 'vocals') {
        audioMode.value = 'song'
      }
    } catch (error) {
      console.error(error)
      vocalsPath.value = null
      if (audioMode.value === 'vocals') {
        audioMode.value = 'song'
      }
    }
  }

  const separateVocals = async () => {
    const filePath = audioSource.value?.file_path
    if (!filePath || isSeparating.value) {
      return
    }

    if (vocalsPath.value) {
      return
    }

    isSeparating.value = true
    const toastId = toast.info('Separating vocals… this can take several minutes.', {
      timeout: false,
    })

    try {
      const path = await invoke('separate_vocals', { filePath })
      vocalsPath.value = path || null
      if (vocalsPath.value) {
        toast.success('Vocal stem ready')
      } else {
        toast.error('Demucs finished but vocals.wav was not found')
      }
    } catch (error) {
      console.error(error)
      toast.error(error?.toString?.() || 'Failed to separate vocals')
    } finally {
      if (toastId != null) {
        toast.dismiss(toastId)
      }
      isSeparating.value = false
    }
  }

  const setAudioMode = mode => {
    if (mode === 'vocals' && !vocalsPath.value) {
      return
    }
    audioMode.value = mode === 'vocals' ? 'vocals' : 'song'
  }

  return {
    audioMode,
    vocalsPath,
    isSeparating,
    canSeparateVocals,
    hasVocalStem,
    playbackSource,
    spectrogramFilePath,
    refreshVocalStem,
    separateVocals,
    setAudioMode,
  }
}
