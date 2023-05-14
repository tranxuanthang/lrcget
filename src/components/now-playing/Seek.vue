<template>
  <div class="bg-brave-90 w-full h-2 relative origin-center hover:scale-y-150 transition" @mouseover="onMouseOver" @mousemove="onMouseOver" @mouseleave="onMouseLeave" @click="chooseProgress">
    <div class="bg-brave-30 h-full" :style="{ width: progressPercent }"></div>
    <div class="bg-brave-60 h-full absolute top-0 left-0 opacity-40" :style="{ width: choosingProgressPercent }"></div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'

const props = defineProps(['duration', 'progress'])
const emit = defineEmits(['seek'])

const progressPercent = computed(() => {
  if (!props.progress || !props.duration) {
    return '0%'
  }
  return `${(props.progress / props.duration) * 100}%` 
})

const choosingProgress = ref(0.0)

const choosingProgressPercent = computed(() => `${choosingProgress.value * 100}%` )

const onMouseOver = (event) => {
  const totalWidth = event.currentTarget.clientWidth
  const seekWidth = event.offsetX
  choosingProgress.value = seekWidth / totalWidth
}

const onMouseLeave = (event) => {
  choosingProgress.value = 0.0
}

const chooseProgress = () => {
  emit('seek', choosingProgress.value * props.duration)
}
</script>
