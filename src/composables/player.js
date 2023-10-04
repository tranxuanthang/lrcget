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
  const setTrack = async (track, playOnLoad = true, setProgress = 0.0) => {
    status.value = 'stopped'
    const platformName = await platform()

    if (howlerSound.value) {
      howlerSound.value.unload()
      Howler.unload()
    }

    playingTrack.value = track

    let assetUrl = null

    if (platformName === 'linux') {
      assetUrl = await convertFileSrc2(playingTrack.value.file_path)
    } else {
      assetUrl = convertFileSrc(playingTrack.value.file_path)
    }

    howlerSound.value = new Howl({
      src: [assetUrl]
    })

    howlerSound.value.once('load', () => {
      duration.value = howlerSound.value.duration()
      if (playOnLoad) {
        howlerSound.value.play()
      }
    })

    howlerSound.value.on('play', () => {
      howlerSound.value.seek(setProgress)
      
      status.value = 'playing'

      if (platformName === 'linux') {
        updater2()
      } else {
        window.requestAnimationFrame(updater)
      }
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

  const playTrack = async (track, setProgress = 0.0) => {
    await setTrack(track, true, setProgress)
  }

  const updater = (timestamp) => {
    if (status.value !== 'playing') {
      return
    }

    const currentProgress = howlerSound.value.seek()

    if (currentProgress > 0.0) {
      progress.value = currentProgress
    }

    window.requestAnimationFrame(updater)
  }

  const updater2 = () => {
    if (status.value !== 'playing') {
      return
    }

    const currentProgress = howlerSound.value.seek()

    if (currentProgress > 0.0) {
      progress.value = currentProgress
    }

    setTimeout(() => {
      updater2()
    }, 70)
  }

  const pause = () => {
    howlerSound.value.unload()
    Howler.unload()
    status.value = 'paused'
  }

  const resume = () => {
    playTrack(playingTrack.value, progress.value)
  }

  const seek = (progress) => {
    if (howlerSound.value) {
      howlerSound.value.unload()
      Howler.unload()
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
