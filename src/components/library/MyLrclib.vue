<template>
  <div class="flex flex-col justify-center gap-6 items-center w-full h-full p-4" v-show="props.isActive">
    <!--<img src="@/assets/lrclib.png" class="w-20 h-20" />-->

    <div class="font-bold text-xs text-brave-30 dark:text-brave-90 select-none cursor-default">
      Search with LRCLIB instance:
      <span class="text-brave-30 dark:text-brave-90 rounded-full px-2 py-1 bg-brave-95 dark:bg-brave-5"
      >{{ lrclibInstance }}</span>
    </div>

    <form class="flex items-center rounded-full w-full max-w-screen-sm h-auto overflow-hidden
    bg-brave-98 dark:bg-brave-5 transition"
      :class="{ 'ring ring-brave-30/30': inputActive }" @submit.prevent="onSubmit">
      <input
        type="text"
        v-model="keyword"
        class="outline-none grow h-12 px-6 bg-brave-98 dark:bg-brave-5 placeholder:text-brave-30/30
         text-brave-20 dark:text-brave-95 dark:placeholder:text-brave-70/30"
        placeholder="Type a song title, album, or artist to find lyrics..."
        @focus="inputActive = true"
        @blur="inputActive = false"
        autofocus
      >
      <button
        class="rounded-full button button-normal h-12 w-12 m-1">
        <Magnify />
      </button>
    </form>

    <Transition name="slide-fade">
      <SearchResult v-if="searchingKeyword" :keyword="searchingKeyword" @back="searchingKeyword = null" />
    </Transition>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Magnify } from 'mdue'
import SearchResult from './my-lrclib/SearchResult.vue'
import { invoke } from '@tauri-apps/api/core'
import { useGlobalState } from '../../composables/global-state'

const { lrclibInstance } = useGlobalState()

const props = defineProps({
  isActive: {
    type: Boolean,
    default: false,
  },
})

const searchingKeyword = ref('')
const keyword = ref('')
const inputActive = ref(false)

const onSubmit = () => {
  searchingKeyword.value = keyword.value
}
</script>
