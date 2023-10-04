import { ref, onMounted, onUnmounted } from 'vue'
import { Howl, Howler } from 'howler'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { invoke } from '@tauri-apps/api/tauri'
import { platform } from '@tauri-apps/api/os'

const convertFileSrc2 = async (path) => {
  return await invoke('convert_file_src_2', { path })
}

const playingTrack = ref(null)
const status = ref('stopped')
const duration = ref(null)
const progress = ref(null)
const howlerSound = ref(null)

export function usePlayer() {
  const setTrack = async (track, playOnLoad = true, progress = 0.0) => {
    status.value = 'stopped'
    const platformName = await platform()

    if (howlerSound.value) {
      howlerSound.value.unload()
      Howler.unload()
      howlerSound.value = null
    }

    playingTrack.value = track

    let assetUrl = null

    if (platformName === 'linux') {
      assetUrl = await convertFileSrc2(playingTrack.value.file_path)
    } else {
      assetUrl = convertFileSrc(playingTrack.value.file_path)
    }

    howlerSound.value = new Howl({
      src: [assetUrl],
      html5: true
    })

    howlerSound.value.once('load', () => {
      duration.value = howlerSound.value.duration()
      if (playOnLoad) {
        howlerSound.value.play()
      }
    })

    howlerSound.value.on('play', () => {
      howlerSound.value.seek(progress)
      status.value = 'playing'
      window.requestAnimationFrame(updater)
    })

    howlerSound.value.on('pause', () => {
      status.value = 'paused'
    })

    howlerSound.value.on('stop', () => {
      status.value = 'paused'
    })

    howlerSound.value.on('end', () => {
      progress.value = duration.value
      status.value = 'ended'
    })
  }

  const playTrack = async (track, progress = 0.0) => {
    await setTrack(track, true, progress)
  }

  const updater = (timestamp) => {
    if (!howlerSound.value || status.value !== 'playing') {
      return
    }

    if (howlerSound.value.seek() > 0.0) {
      progress.value = howlerSound.value.seek()
    }

    window.requestAnimationFrame(updater)
  }

  const pause = () => {
    howlerSound.value.unload()
    Howler.unload()
    howlerSound.value = null
  }

  const resume = () => {
    playTrack(playingTrack.value, progress.value)
  }

  const seek = (progress) => {
    if (howlerSound.value) {
      howlerSound.value.unload()
      Howler.unload()
      howlerSound.value = null
    }
    playTrack(playingTrack.value, progress)
  }

  const stop = () => {
    if (howlerSound.value) {
      howlerSound.value.stop()
    }
  }

  const unload = () => {
    progress.value = 0.0
    status.value = 'stopped'
    if (howlerSound.value) {
      howlerSound.value.unload()
      Howler.unload()
      howlerSound.value = null
    }
  }

  return {
    playingTrack,
    status,
    duration,
    progress,
    setTrack,
    playTrack,
    pause,
    resume,
    stop,
    seek,
    unload
  }
}
