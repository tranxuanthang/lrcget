<script setup>
const props = defineProps({
  action: {
    type: Function,
    required: true
  },
  title: {
    type: String,
    required: false
  },
  disabled: {
    type: Boolean,
    required: false
  }
})

</script>

<template>
    <li v-if="!disabled" class="dropdown-item button-normal cursor-pointer text-sm py-1.5 px-1.5 h-8 w-full whitespace-nowrap" @click="action" :title="title">
      <slot/>
    </li>
    <li v-else class="dropdown-item button-disabled text-sm py-1.5 px-1.5 h-8 w-full whitespace-nowrap" :title="title">
      <slot/>
    </li>
</template>

<script>
</script>