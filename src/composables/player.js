import { ref, onMounted, onUnmounted } from 'vue'
import { Howl, Howler } from 'howler'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { invoke } from '@tauri-apps/api/tauri'
import { platform } from '@tauri-apps/api/os';

const convertFileSrc2 = async (path) => {
  return await invoke('convert_file_src_2', { path })
}

const playingTrack = ref(null)
const status = ref('stopped')
const duration = ref(null)
const progress = ref(null)
const howlerSound = ref(null)
const id = ref(null)

export function usePlayer() {
  const setTrack = async (track, playOnLoad = true) => {
    status.value = 'stopped'
    const platformName = await platform()

    if (howlerSound.value) {
      howlerSound.value.unload()
      howlerSound.value = null
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
      src: [assetUrl],
      preload: true
    })

    howlerSound.value.once('load', () => {
      duration.value = howlerSound.value.duration(id.value)
      if (playOnLoad) {
        id.value = howlerSound.value.play()
      }
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

  const playTrack = async (track) => {
    await setTrack(track, true)
  }

  const updater = (timestamp) => {
    if (howlerSound.value && howlerSound.value.seek(null, id.value)) {
      progress.value = howlerSound.value.seek(null, id.value)
    } else {
      progress.value = 0.0
    }

    console.log(progress.value)

    if (status.value === 'playing') {
      window.requestAnimationFrame(updater)
    }
  }

  const pause = () => {
    howlerSound.value.pause(id.value)
  }

  const resume = () => {
    howlerSound.value.seek(progress.value, id.value)
    howlerSound.value.play(id.value)
    howlerSound.value.seek(progress.value, id.value)
  }

  const seek = (progress) => {
    howlerSound.value.seek(progress, id.value)
  }

  const stop = () => {
    if (howlerSound.value) {
      howlerSound.value.stop(id.value)
    }
  }

  const unload = () => {
    howlerSound.value.unload()
    status.value = 'stopped'
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
    seek
  }
}
