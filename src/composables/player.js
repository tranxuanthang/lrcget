import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

const playingTrack = ref(null)
const status = ref('stopped')
const duration = ref(null)
const progress = ref(null)

listen('player-state', async (event) => {
  duration.value = event.payload.duration
  progress.value = event.payload.progress
  status.value = event.payload.status
})

listen('reload-track-id', async (event) => {
  const payload = event.payload
  if (playingTrack.value.id === payload) {
    playingTrack.value = await invoke('get_track', { trackId: playingTrack.value.id })
  }
})

export function usePlayer() {
  const playTrack = (track) => {
    playingTrack.value = track
    invoke('play_track', { trackId: track.id })
  }

  const pause = () => {
    invoke('pause_track')
  }

  const resume = () => {
    invoke('resume_track')
  }

  const seek = (position) => {
    invoke('seek_track', { position })
  }

  const stop = () => {
    invoke('stop_track')
  }

  return {
    playingTrack,
    status,
    duration,
    progress,
    playTrack,
    pause,
    resume,
    stop,
    seek
  }
}
