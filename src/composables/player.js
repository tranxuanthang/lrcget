import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const playingTrack = ref(null);
const status = ref("stopped");
const duration = ref(null);
const progress = ref(null);
const volume = ref(1.0);

listen("player-state", async (event) => {
  duration.value = event.payload.duration;
  progress.value = event.payload.progress;
  status.value = event.payload.status;
  volume.value = event.payload.volume;
});

listen("reload-track-id", async (event) => {
  const payload = event.payload;
  if (playingTrack.value && playingTrack.value.id === payload) {
    playingTrack.value = await invoke("get_track", {
      trackId: playingTrack.value.id,
    });
  }
});

export function usePlayer() {
  const playTrack = (track) => {
    playingTrack.value = track;
    invoke("play_track", { trackId: track.id });
  };

  const pause = () => {
    if (!playingTrack.value) {
      return;
    }

    invoke("pause_track");
  };

  const resume = () => {
    if (!playingTrack.value) {
      return;
    }

    invoke("resume_track");
  };

  const seek = (position) => {
    if (!playingTrack.value) {
      return;
    }

    if (status.value === "stopped") {
      invoke("play_track", { trackId: playingTrack.value.id });
    }

    invoke("seek_track", { position });
  };

  const stop = () => {
    if (!playingTrack.value) {
      return;
    }

    invoke("stop_track");
  };

  const setVolume = (volume) => {
    invoke("set_volume", { volume });
  };

  return {
    playingTrack,
    status,
    duration,
    progress,
    volume,
    playTrack,
    pause,
    resume,
    stop,
    seek,
    setVolume,
  };
}
