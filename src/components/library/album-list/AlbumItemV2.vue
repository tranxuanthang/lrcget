<template>
  <div
    class="album-item cursor-pointer flex items-center p-2 hover:bg-brave-95 dark:hover:bg-brave-10 rounded"
    @click="handleClick"
  >
    <div class="album-art flex-none mr-3">
      <div
        v-if="album"
        class="w-10 h-10 rounded bg-brave-90 dark:bg-brave-20 flex items-center justify-center overflow-hidden"
      >
        <img
          v-if="album.artwork_url"
          :src="album.artwork_url"
          :alt="album.title"
          class="w-full h-full object-cover"
        />
        <MusicNote v-else class="w-6 h-6 text-brave-40" />
      </div>
      <div
        v-else
        class="w-10 h-10 rounded bg-brave-95 dark:bg-brave-15 animate-pulse"
      ></div>
    </div>

    <div class="album-info flex-1 min-w-0">
      <div v-if="album" class="album-title font-medium text-sm truncate">
        {{ album.title }}
      </div>
      <div
        v-else
        class="h-4 bg-brave-95 dark:bg-brave-15 w-32 rounded animate-pulse"
      ></div>

      <div
        v-if="album"
        class="album-artist text-xs text-brave-40 dark:text-brave-60 truncate"
      >
        {{ album.artist_name }}
      </div>
      <div
        v-else
        class="h-3 mt-1 bg-brave-95 dark:bg-brave-15 w-24 rounded animate-pulse"
      ></div>
    </div>

    <div class="flex-none ml-2">
      <div v-if="album" class="text-xs text-brave-50">
        {{ album.track_count }} tracks
      </div>
      <div
        v-else
        class="h-3 bg-brave-95 dark:bg-brave-15 w-12 rounded animate-pulse"
      ></div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { MusicNote } from "mdue";

const props = defineProps({
  albumId: {
    type: Number,
    required: true,
  },
});

const emit = defineEmits(["open-album"]);

const album = ref(null);
const loading = ref(false);

const loadAlbum = async () => {
  if (loading.value) return;

  loading.value = true;
  try {
    album.value = await invoke("get_album_details", { albumId: props.albumId });
  } catch (error) {
    console.error("Failed to load album:", error);
  } finally {
    loading.value = false;
  }
};

const handleClick = () => {
  if (album.value) {
    emit("open-album", album.value);
  }
};

onMounted(() => {
  loadAlbum();
});
</script>
