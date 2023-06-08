import { ref, onMounted, onUnmounted } from 'vue'
import { Howl, Howler } from 'howler'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { invoke } from '@tauri-apps/api/tauri'
import { platform } from '@tauri-apps/api/os';

const convertFileSrc2 = async (path) => {
  return await invoke('convert_file_src_2', { path })
}

export function usePlayer() {
  const playingTrack = ref(null)
  const status = ref(null)
  const duration = ref(null)
  const progress = ref(null)
  const howlerSound = ref(null)

  const playTrack = async (track) => {
    const platformName = await platform()

    if (howlerSound.value) {
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
      howlerSound.value.seek(0.0)
      howlerSound.value.play()
    })

    howlerSound.value.on('play', () => {
      status.value = 'playing'
      window.requestAnimationFrame(updater)
    })

    howlerSound.value.on('pause', () => {
      status.value = 'paused'
    })

    howlerSound.value.on('stop', () => {
      status.value = 'stopped'
    })

    howlerSound.value.on('end', () => {
      progress.value = duration.value
      status.value = 'ended'
    })
  }

  const updater = (timestamp) => {
    progress.value = howlerSound.value.seek()

    if (status.value === 'playing') {
      window.requestAnimationFrame(updater)
    }
  }

  const pause = (progress) => {
    howlerSound.value.pause()
  }

  const resume = (progress) => {
    howlerSound.value.play()
  }

  const seek = (progress) => {
    howlerSound.value.seek(progress)
  }

  onUnmounted(() => {
    if (howlerSound.value) {
      howlerSound.value.unload()
    }
  })

  return {
    playingTrack,
    status,
    duration,
    progress,
    playTrack,
    pause,
    resume,
    seek
  }
}
