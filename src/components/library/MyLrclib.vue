<template>
  <div
    v-show="props.isActive"
    class="flex flex-col justify-center gap-6 items-center w-full h-full p-4"
  >
    <!--<img src="@/assets/lrclib.png" class="w-20 h-20" />-->

    <div class="font-bold text-xs text-neutral-800 dark:text-neutral-400 select-none cursor-default">
      Search with LRCLIB instance:
      <span
        class="text-neutral-800 dark:text-neutral-300 rounded-full px-2 py-1 bg-neutral-50 dark:bg-neutral-900"
        >{{ lrclibInstance }}</span
      >
    </div>

    <form
      class="flex items-center rounded-full w-full max-w-screen-sm h-auto overflow-hidden bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 transition"
      :class="{ 'ring-2 ring-hoa-1100/40': inputActive }"
      @submit.prevent="onSubmit"
    >
      <input
        v-model="keyword"
        type="text"
        class="outline-none grow h-12 px-6 bg-transparent text-neutral-800 dark:text-neutral-200 placeholder:text-neutral-400 dark:placeholder:text-neutral-500"
        placeholder="Type a song title, album, or artist to find lyrics..."
        autofocus
        @focus="inputActive = true"
        @blur="inputActive = false"
      />
      <button class="rounded-full button button-normal h-10 w-10 mr-1">
        <Magnify />
      </button>
    </form>

    <Transition name="slide-fade">
      <SearchResult
        v-if="searchingKeyword"
        :keyword="searchingKeyword"
        @back="searchingKeyword = null"
      />
    </Transition>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import Magnify from '~icons/mdi/magnify'
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
