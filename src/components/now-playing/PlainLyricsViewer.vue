<template>
  <transition name="slide-fade" mode="out-in">
    <div v-if="lyrics" class="flex flex-col gap-1 relative">
      <transition name="slide-fade" mode="out-in">
        <div v-if="expanded" class="full-viewer absolute bottom-0 left-0 w-full h-[40vh] bg-brave-95 dark:bg-brave-10 border-t border-brave-90/50 dark:border-brave-10/50 overflow-hidden">
          <div class="relative h-full rounded text-center text-brave-50 whitespace-pre flex flex-col">
            <div class="flex justify-center items-center h-6 w-full relative z-10">
              <button class="text-xl text-brave-30 w-full flex justify-center" type="button" @click="expand"><DragHorizontal /></button>
            </div>
            <div class="grow p-4 h-full overflow-y-auto">
              {{ props.lyrics }}
            </div>
          </div>
        </div>
      </transition>

      <div class="mini-viewer transition cursor-pointer top-0 left-0 w-full h-12 bg-brave-95 dark:bg-brave-10 border-t border-brave-90/50 dark:border-brave-30 flex flex-col" :class="{ 'invisible opacity-0': expanded }" @click="expand">
        <div class="flex justify-center items-center h-4 w-full">
          <button class="text-xl text-brave-30 w-full flex justify-center" type="button"><DragHorizontal /></button>
        </div>

        <transition name="slide-fade" mode="out-in">
          <div class="flex w-full justify-center items-center text-brave-30 text-sm grow italic dark:text-brave-90">
            [Unsynchronized lyrics]
          </div>
        </transition>
      </div>
    </div>
  </transition>
</template>

<script setup>
import { DragHorizontal } from 'mdue'
import { ref } from 'vue'
import { computed } from '@vue/reactivity'

const props = defineProps(['lyrics'])

const expanded = ref(false)

const expand = () => {
  expanded.value = !expanded.value
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
