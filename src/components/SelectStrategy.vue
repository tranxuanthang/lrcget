<template>
<div class="flex flex-col justify-center items-center w-full h-full p-4">
   <div class="m-4 text-hoa-1500 text-sm">Select Strategy</div>
   <div class="flex mb-4 gap-4 text-gray-200">
      <button @click="chooseCreateLRC" class="text-left flex-1 border-4 rounded p-3 transition" :class="{ 'border-hoa-1400 bg-hoa-800/70': isCreateLRC, 'border-transparent bg-hoa-800/30': !isCreateLRC }">
         <div class="font-bold text-hoa-1400 mb-3">Create separate LRC files</div>
         <div class="text-sm text-hoa-1300">LRC lyrics files will be created in the same location with the music files and have the same name.</div>
      </button>

      <button @click="chooseEmbed" class="text-left flex-1 border-4 rounded p-3 transition" :class="{ 'border-hoa-1400 bg-hoa-800/70': isEmbed, 'border-transparent bg-hoa-800/30': !isEmbed }">
         <div class="font-bold text-hoa-1400 mb-3">Embed lyrics to music files</div>
         <div class="text-sm text-hoa-1300">Lyrics will be embedded to music files through SYLT tag (mp3) or LYRICS tag (flac). Limited players support.</div>
      </button>
   </div>

   <div class="">
    <div class="flex items-center">
      <input id="skip-tracks" type="checkbox" v-model="skipTracksHaveExistingLyrics" class="w-4 h-4 text-hoa-600 bg-hoa-1100 accent-hoa-1100 rounded border-hoa-300 focus:ring-hoa-500 focus:ring-2">
      <label for="skip-tracks" class="ml-2 text-sm font-medium text-hoa-1500">Skip tracks that already have lyrics</label>
    </div>
   </div>

   <div class="p-3">
      <button type="button" @click="progressStep" class="px-6 py-2 bg-hoa-1100 rounded-full uppercase font-bold text-white hover:bg-hoa-1200 active:bg-hoa-1300 transition">
         Confirm
      </button>
   </div>
</div>
</template>

<script setup>
import { ref, defineEmits } from 'vue'

const emit = defineEmits(['progressStep'])

const isCreateLRC = ref(true)
const isEmbed = ref(false)
const skipTracksHaveExistingLyrics = ref(true)

const progressStep = () => {
  emit('progressStep', isCreateLRC.value, isEmbed.value, skipTracksHaveExistingLyrics.value)
}

const chooseCreateLRC = () => {
   isCreateLRC.value = true
   isEmbed.value = false
}

const chooseEmbed = () => {
   isCreateLRC.value = false
   isEmbed.value = true
}
</script>
