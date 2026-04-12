<template>
  <div class="inline-flex items-center">
    <input
      :id="id"
      type="radio"
      :name="name"
      :value="value"
      :checked="modelValue === value"
      class="hidden"
      @change="$emit('update:modelValue', value)"
    />
    <label
      :for="id"
      class="inline-flex items-center cursor-pointer select-none"
      :class="disabled ? 'opacity-50 cursor-not-allowed' : ''"
    >
      <RadioboxMarked
        v-if="modelValue === value"
        class="w-6 h-6 text-brave-30 dark:text-brave-90"
      />
      <RadioboxBlank v-else class="w-6 h-6 text-brave-30 dark:text-brave-90" />
      <span class="ml-1 child-label">
        <slot />
      </span>
    </label>
  </div>
</template>

<script setup>
import RadioboxMarked from '~icons/mdi/radiobox-marked'
import RadioboxBlank from '~icons/mdi/radiobox-blank'
defineProps({
  modelValue: {
    type: [String, Number, Boolean],
    required: true,
  },
  value: {
    type: [String, Number, Boolean],
    required: true,
  },
  name: {
    type: String,
    required: true,
  },
  id: {
    type: String,
    required: true,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
})

defineEmits(['update:modelValue'])
</script>
