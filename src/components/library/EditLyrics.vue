<template>
  <div>
    <div class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-30" :class="{ 'hidden': !props.isShow }">
      <div class="w-full h-[95vh] max-w-screen-sm rounded-lg m-4 bg-white flex flex-col gap-2">
        <div class="flex-none flex justify-between items-center px-6 py-2">
          <div class="text-thin text-xl text-brave-15">Edit Lyrics</div>
          <button class="text-brave-20 hover:text-brave-15 hover:bg-brave-95 active:text-white active:bg-brave-25 transition rounded-full p-4" @click="close"><Close /></button>
        </div>

        <div class="px-6 grow overflow-hidden flex flex-col gap-4">

        </div>
      </div>
    </div>

    <div class="fixed top-0 left-0 h-full w-full z-20 bg-black/30" :class="{ 'hidden': !props.isShow }" @click="close">
    </div>
  </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'
import { ref, onMounted } from 'vue'
import { Close, Loading } from 'mdue'
import { useToast } from 'vue-toastification'
import { useEditLyrics } from '@/composables/edit-lyrics.js'

const toast = useToast()
const props = defineProps(['isShow'])
const { editingTrack } = useEditLyrics()
const loading = ref(true)

const close = () => {
  searchingTrack.value = null
}

onMounted(async () => {
})
</script>
