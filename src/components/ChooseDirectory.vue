<template>
  <div class="flex flex-col w-full h-full justify-center items-center">
    <div class="px-4 py-2 flex flex-col gap-4 flex-none">
      <div class="text-thin text-xl text-brave-5 dark:text-brave-95">
        Select directories
      </div>
    </div>

    <div
      class="grow flex flex-col items-center justify-center gap-8 w-full max-w-screen-sm"
    >
      <div class="flex flex-col gap-2 w-full justify-center items-center">
        <div
          v-for="(directory, index) in directories"
          :key="directory"
          class="w-full bg-brave-90 dark:bg-brave-10 text-brave-5 dark:text-brave-95 font-bold p-4 rounded-lg flex items-center"
        >
          <div class="grow">
            {{ directory }}
          </div>
          <button
            class="flex-none button button-normal p-2 rounded-full"
            @click.prevent="removeDirectory(index)"
          >
            <Close />
          </button>
        </div>

        <button
          class="w-full bg-brave-95 dark:bg-brave-5 hover:bg-brave-90 hover:dark:bg-brave-10 active:bg-brave-80/50 transition text-brave-5 dark:text-brave-95 font-bold p-4 rounded-lg flex items-center border border-dashed border-brave-70 dark:border-brave-30"
          @click.prevent="chooseDirectory"
        >
          <Plus />
          <div>Add new directory</div>
        </button>
      </div>

      <button
        class="button button-primary w-full p-4 rounded-lg"
        @click.prevent="progressStep"
      >
        Continue
      </button>
    </div>
  </div>
</template>

<script setup>
import { open } from "@tauri-apps/plugin-dialog";
import { audioDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted } from "vue";
import { Close, Plus } from "mdue";

const emit = defineEmits(["progressStep"]);

const directories = ref([]);

const progressStep = async () => {
  await invoke("set_directories", { directories: directories.value });
  emit("progressStep");
};

onMounted(async () => {
  const init = await invoke("get_init");
  if (init) {
    emit("progressStep");
  }

  const directoriesFromDB = await invoke("get_directories");
  if (directoriesFromDB) {
    directories.value = directoriesFromDB;
  } else {
    const dirPath = await audioDir();
    directories.value.push(dirPath);
  }
});

const chooseDirectory = async () => {
  const selected = await open({
    directory: true,
    recursive: true,
  });

  if (selected && !directories.value.includes(selected)) {
    directories.value.push(selected);
  }
};

const removeDirectory = (index) => {
  if (index < 0) {
    return;
  }
  directories.value.splice(index, 1);
};
</script>
