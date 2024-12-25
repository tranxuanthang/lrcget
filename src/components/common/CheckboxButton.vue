<template>
  <div class="inline-flex items-center">
    <input
      type="checkbox"
      :id="id"
      :name="name"
      :checked="modelValue"
      class="hidden"
      @change="$emit('update:modelValue', !modelValue)"
    >
    <label
      :for="id"
      class="inline-flex items-center cursor-pointer select-none"
      :class="disabled ? 'opacity-50 cursor-not-allowed' : ''"
    >
      <CheckboxMarked
        v-if="modelValue"
        class="w-6 h-6 text-brave-30 dark:text-brave-90"
      />
      <CheckboxBlankOutline
        v-else
        class="w-6 h-6 text-brave-30 dark:text-brave-90"
      />
      <span class="ml-1 child-label">
        <slot></slot>
      </span>
    </label>
  </div>
</template>

<script setup>
import { CheckboxMarked, CheckboxBlankOutline } from 'mdue'

defineProps({
  modelValue: {
    type: Boolean,
    required: true
  },
  name: {
    type: String,
    required: true
  },
  id: {
    type: String,
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

defineEmits(['update:modelValue'])
</script>
