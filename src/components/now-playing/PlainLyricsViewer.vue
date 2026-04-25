<template>
  <transition name="slide-fade" mode="out-in">
    <div v-if="plainLyrics" class="flex flex-col gap-1 relative">
      <transition name="slide-fade" mode="out-in">
        <div
          v-if="expanded"
          class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-neutral-50 dark:bg-neutral-900 border-t border-neutral-200/50 dark:border-neutral-800/50 overflow-hidden"
        >
          <div
            class="relative h-full rounded text-center text-neutral-700 whitespace-pre flex flex-col"
          >
            <div class="flex justify-center items-center h-6 w-full relative z-10">
              <button
                class="text-xl text-neutral-800 w-full flex justify-center"
                type="button"
                @click="expand"
              >
                <DragHorizontal />
              </button>
            </div>
            <div class="grow p-4 h-full overflow-y-auto">
              {{ plainLyrics }}
            </div>

            <button
              class="absolute bottom-2 right-2 flex items-center gap-1 px-3 py-1 rounded text-xs font-bold bg-neutral-100 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-700 hover:bg-neutral-200 dark:hover:bg-neutral-800 shadow"
              type="button"
              :aria-label="copied ? 'Copied' : 'Copy'"
              @click.stop="onCopy"
            >
              <ContentCopy class="w-4 h-4" />
              <span>{{ copied ? 'Copied' : 'Copy' }}</span>
            </button>
          </div>
        </div>
      </transition>

      <div
        class="mini-viewer transition cursor-pointer top-0 left-0 w-full h-12 bg-neutral-50 dark:bg-neutral-900 border-t border-neutral-200/50 dark:border-neutral-800/50 flex flex-col"
        :class="{ 'invisible opacity-0': expanded }"
        @click="expand"
      >
        <div class="flex justify-center items-center h-4 w-full">
          <button class="text-xl text-neutral-800 w-full flex justify-center" type="button">
            <DragHorizontal />
          </button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div
            v-if="!expanded"
            class="flex w-full justify-center items-center text-neutral-800 text-sm grow italic dark:text-neutral-600"
          >
            [Unsynchronized lyrics]
          </div>
        </transition>
      </div>
    </div>
  </transition>
</template>

<script setup>
import DragHorizontal from '~icons/mdi/drag-horizontal'
import ContentCopy from '~icons/mdi/content-copy'
import { ref, computed } from 'vue'
import { parseLyricsfile } from '@/utils/lyricsfile.js'

const props = defineProps(['lyricsfile'])

const expanded = ref(false)
const copied = ref(false)

const parsedLyricsfile = computed(() => {
  if (!props.lyricsfile) {
    return null
  }
  return parseLyricsfile(props.lyricsfile)
})

const plainLyrics = computed(() => {
  if (!parsedLyricsfile.value) {
    return null
  }
  return parsedLyricsfile.value.plainLyrics || null
})

const expand = () => {
  expanded.value = !expanded.value
}

const onCopy = async () => {
  try {
    const text = plainLyrics.value || ''
    if (!text) return
    await navigator.clipboard.writeText(text)
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch (e) {
    // swallow
  }
}
</script>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.1s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.1s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

.slide-fade-enter-from {
  transform: translateY(20px);
  opacity: 0;
}
</style>
