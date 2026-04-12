<template>
  <div>
    <LyricsViewer
      v-if="hasSyncedLyrics && !instrumental"
      :lyricsfile="lyricsfile"
      :duration="duration"
      :progress="progress"
      @lyrics-clicked="lyricsClicked"
    />
    <PlainLyricsViewer v-else-if="hasPlainLyrics && !instrumental" :lyricsfile="lyricsfile" />
    <div v-else class="border-b border-brave-90/50 dark:border-brave-30" />
    <div
      class="bg-brave-95 backdrop-blur px-4 py-3 flex-none flex flex-col justify-center items-center gap-3 dark:bg-brave-10"
    >
      <div class="w-full flex gap-1 justify-center items-center">
        <div class="flex-none w-12 text-xs text-brave-30 dark:text-brave-80">
          {{ humanDuration(progress) }}
        </div>
        <Seek class="grow" :duration="duration" :progress="progress" @seek="seek" />
        <div class="flex-none text-right w-12 text-xs text-brave-30 dark:text-brave-80">
          {{ humanDuration(duration) }}
        </div>
      </div>

      <div class="flex justify-between w-full">
        <div class="basis-1/3 flex-1 grow-0 flex flex-col justify-center items-start gap-0.5">
          <div v-if="playingTrack">
            <div class="text-xs font-bold text-brave-30 dark:text-brave-95 line-clamp-1">
              {{ playingTrack.title }}
            </div>
            <div class="text-xs text-brave-40 dark:text-brave-90 line-clamp-1">
              {{ playingTrack.album_name }} - {{ playingTrack.artist_name }}
            </div>
          </div>
        </div>

        <div class="basis-1/3 flex-1 flex justify-center items-center gap-2">
          <button
            class="button button-secondary p-1 m-1 rounded-full text-lg"
            @click.prevent="seek(reverse10)"
          >
            <Rewind_10 />
          </button>
          <button
            v-if="status === 'playing'"
            class="button button-primary text-white p-2 rounded-full text-xl"
            @click.prevent="pause"
          >
            <Pause />
          </button>
          <button
            v-else-if="playingTrack && status === 'stopped'"
            class="button button-primary text-white p-2 rounded-full text-xl"
            @click.prevent="playTrack(playingTrack)"
          >
            <Replay />
          </button>
          <button
            v-else
            class="button button-primary text-white p-2 rounded-full text-xl"
            @click.prevent="resume"
          >
            <Play />
          </button>
          <button
            class="button button-secondary p-1 m-1 rounded-full text-lg"
            @click.prevent="seek(forward10)"
          >
            <FastForward_10 />
          </button>
        </div>

        <div class="basis-1/3 flex-1 flex justify-end items-center">
          <VolumeSlider :volume="volume" @set-volume="setPlayerVolume" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { ref, onMounted, onUnmounted } from 'vue'
import Seek from './now-playing/Seek.vue'
import LyricsViewer from './now-playing/LyricsViewer.vue'
import PlainLyricsViewer from './now-playing/PlainLyricsViewer.vue'
import Play from '~icons/mdi/play'
import Pause from '~icons/mdi/pause'
import Replay from '~icons/mdi/replay'
import Rewind_10 from '~icons/mdi/rewind-10'
import FastForward_10 from '~icons/mdi/fast-forward-10'
import { usePlayer } from '@/composables/player.js'
import { useGlobalState } from '@/composables/global-state.js'
import VolumeSlider from './now-playing/VolumeSlider.vue'
import { humanDuration } from '@/utils/human-duration'
import { parseLyricsfile } from '@/utils/lyricsfile.js'

const { isHotkey } = useGlobalState()
const {
  playingTrack,
  status,
  duration,
  progress,
  volume,
  playTrack,
  pause,
  resume,
  seek,
  setVolume: setPlayerVolume,
} = usePlayer()
const keydownEvent = ref(null)

const instrumental = computed(() => {
  if (!playingTrack.value) {
    return null
  }

  return playingTrack.value.instrumental
})

const lyricsfile = computed(() => {
  if (!playingTrack.value) {
    return null
  }

  return playingTrack.value.lyricsfile
})

const hasSyncedLyrics = computed(() => {
  if (!lyricsfile.value) {
    return false
  }
  const parsed = parseLyricsfile(lyricsfile.value)
  return parsed.syncedLines && parsed.syncedLines.length > 0
})

const hasPlainLyrics = computed(() => {
  if (!lyricsfile.value) {
    return false
  }
  const parsed = parseLyricsfile(lyricsfile.value)
  return parsed.plainLyrics && parsed.plainLyrics.length > 0
})

const forward10 = computed(() => {
  if (!playingTrack.value) {
    return null
  }
  return Math.min(duration.value - 1, progress.value + 10)
})

const reverse10 = computed(() => {
  if (!playingTrack.value) {
    return null
  }
  return Math.max(0, progress.value - 10)
})

const setVolume = event => {
  const volume = parseFloat(event.target.value)
  setPlayerVolume(volume)
}

const lyricsClicked = line => {
  seek(line.timestamp)
}

onUnmounted(async () => {
  if (keydownEvent.value) {
    document.removeEventListener(keydownEvent.value)
  }
})

onMounted(async () => {
  keydownEvent.value = document.addEventListener('keydown', event => {
    if (!isHotkey.value) {
      // hotkey is explicitly disabled
      return
    }

    const target = event.target

    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.closest('.v-codemirror')
    ) {
      // Do nothing if the target is an input, textarea, or CodeMirror element
      return
    }

    switch (event.code) {
      case 'Space':
      case 'Enter':
        event.preventDefault()
        if (status.value === 'playing') {
          pause()
        } else if (playingTrack.value && status.value === 'stopped') {
          playTrack(playingTrack.value)
        } else {
          resume()
        }
        break
      case 'ArrowLeft':
        event.preventDefault()
        seek(reverse10.value)
        break
      case 'ArrowRight':
        event.preventDefault()
        seek(forward10.value)
        break
      default:
        break
    }
  })
})
</script>
