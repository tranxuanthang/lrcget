import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const playingTrack = ref(null)
const status = ref('stopped')
const duration = ref(null)
const progress = ref(null)
const volume = ref(1.0)
const playbackSpeed = ref(1.0)

listen('player-state', async event => {
  duration.value = event.payload.duration
  progress.value = event.payload.progress
  status.value = event.payload.status
  volume.value = event.payload.volume
  playbackSpeed.value = event.payload.playback_speed ?? 1.0
})

listen('reload-track-id', async event => {
  const payload = event.payload
  if (playingTrack.value && playingTrack.value.id === payload) {
    playingTrack.value = await invoke('get_track', { trackId: playingTrack.value.id })
  }
})

export function usePlayer() {
  const playTrack = track => {
    playingTrack.value = track

    if (track.id !== undefined && track.id !== null) {
      return invoke('play_track', {
        trackId: track.id,
        filePath: null,
        title: track.title,
        albumName: track.album_name,
        artistName: track.artist_name,
        albumArtistName: track.album_artist_name,
        duration: track.duration,
      })
    } else if (track.file_path) {
      return invoke('play_track', {
        trackId: null,
        filePath: track.file_path,
        title: track.title,
        albumName: track.album_name,
        artistName: track.artist_name,
        albumArtistName: track.album_artist_name,
        duration: track.duration,
      })
    }

    return Promise.resolve()
  }

  const pause = () => {
    if (!playingTrack.value) {
      return
    }

    invoke('pause_track')
  }

  const resume = () => {
    if (!playingTrack.value) {
      return
    }

    invoke('resume_track')
  }

  const seek = async position => {
    if (!playingTrack.value) {
      return
    }

    // Rust may have cleared sound_handle on stop. Reload with FULL metadata
    // (not bare {trackId} — drops file_path for file-based tracks) and AWAIT,
    // or seek_track races against an un-loaded handle and silently no-ops.
    if (status.value === 'stopped') {
      await playTrack(playingTrack.value)
    }

    invoke('seek_track', { position })
  }

  const stop = () => {
    if (!playingTrack.value) {
      return
    }

    invoke('stop_track')
  }

  const setVolume = volume => {
    invoke('set_volume', { volume })
  }

  const setPlaybackSpeed = playbackSpeedValue => {
    invoke('set_playback_speed', { playbackSpeed: playbackSpeedValue })
  }

  return {
    playingTrack,
    status,
    duration,
    progress,
    volume,
    playbackSpeed,
    playTrack,
    pause,
    resume,
    stop,
    seek,
    setVolume,
    setPlaybackSpeed,
  }
}
