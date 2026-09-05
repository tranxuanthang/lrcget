<template>
  <div class="flex-none" @keydown.stop @keyup.stop @focusout="handleFocusOut">
    <VDropdown
      v-model:shown="shown"
      theme="lrcget-dropdown"
      placement="top-end"
      :triggers="[]"
      :container="false"
      :delay="0"
      :distance="8"
      strategy="fixed"
      no-auto-focus
      @apply-show="focusCurrentOption"
    >
      <button
        ref="trigger"
        type="button"
        class="speed-trigger"
        :class="{ 'speed-trigger-open': shown }"
        :aria-label="`Playback speed: ${formatSpeed(modelValue)}`"
        aria-haspopup="menu"
        :aria-expanded="shown"
        @click="shown = !shown"
        @keydown="handleTriggerKeydown"
      >
        <span>{{ formatSpeed(modelValue) }}</span>
        <ChevronDown
          class="text-sm transition-transform"
          :class="{ 'rotate-180': shown }"
          aria-hidden="true"
        />
      </button>

      <template #popper>
        <div
          ref="menu"
          role="menu"
          aria-label="Playback speed"
          class="speed-menu"
          @keydown="handleMenuKeydown"
        >
          <button
            v-for="(speed, index) in speedOptions"
            :key="speed"
            type="button"
            role="menuitemradio"
            :aria-checked="modelValue === speed"
            tabindex="-1"
            class="speed-option"
            :class="{ 'speed-option-selected': modelValue === speed }"
            @focus="activeIndex = index"
            @click="selectSpeed(speed)"
          >
            <span>{{ formatSpeed(speed) }}</span>
            <Check v-if="modelValue === speed" class="text-base" aria-hidden="true" />
          </button>
        </div>
      </template>
    </VDropdown>
  </div>
</template>

<script setup>
import { nextTick, ref } from 'vue'
import Check from '~icons/mdi/check'
import ChevronDown from '~icons/mdi/chevron-down'

const props = defineProps({ modelValue: { type: Number, default: 1 } })
const emit = defineEmits(['update:modelValue'])
const speedOptions = [0.5, 0.75, 1, 1.25, 1.5, 2]
const shown = ref(false)
const trigger = ref(null)
const menu = ref(null)
const activeIndex = ref(0)
const formatSpeed = speed => `${speed}x`

const focusOption = index => {
  activeIndex.value = (index + speedOptions.length) % speedOptions.length
  menu.value?.querySelectorAll('[role="menuitemradio"]')[activeIndex.value]?.focus()
}

const focusCurrentOption = async () => {
  await nextTick()
  if (shown.value) {
    // We manage menu focus; exclude Floating Vue's wrapper from the normal tab order.
    menu.value.closest('.v-popper__popper').tabIndex = -1
    focusOption(Math.max(0, speedOptions.indexOf(props.modelValue)))
  }
}

const close = () => {
  shown.value = false
  trigger.value?.focus()
}

const selectSpeed = speed => {
  emit('update:modelValue', speed)
  close()
}

const handleTriggerKeydown = event => {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (shown.value) focusCurrentOption()
    else shown.value = true
  } else if (event.key === 'Escape' && shown.value) {
    event.preventDefault()
    close()
  }
}

const handleMenuKeydown = event => {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      focusOption(activeIndex.value + 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      focusOption(activeIndex.value - 1)
      break
    case 'Home':
      event.preventDefault()
      focusOption(0)
      break
    case 'End':
      event.preventDefault()
      focusOption(speedOptions.length - 1)
      break
    case 'Escape':
      event.preventDefault()
      close()
      break
    case 'Tab':
      // Resume normal tab order from the trigger, including inside modal focus traps.
      close()
      break
  }
}

const handleFocusOut = event => {
  if (!event.currentTarget.contains(event.relatedTarget)) shown.value = false
}
</script>

<style scoped>
.speed-trigger {
  @apply inline-flex h-7 min-w-[4.5rem] items-center justify-between gap-2 rounded-full border border-neutral-300 bg-white px-2.5 text-xs font-semibold tabular-nums text-neutral-700 transition hover:bg-neutral-100 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700;
}

.speed-trigger:focus-visible,
.speed-trigger-open {
  @apply outline-none ring-2 ring-hoa-1100/60;
}

.speed-menu {
  @apply w-28 max-h-[calc(100vh-2rem)] overflow-y-auto p-1;
}

.speed-option {
  @apply flex h-8 w-full items-center justify-between rounded px-2.5 text-xs font-medium tabular-nums text-neutral-700 hover:bg-neutral-100 dark:text-neutral-200 dark:hover:bg-neutral-700;
}

.speed-option-selected {
  @apply bg-hoa-100/60 text-hoa-1500 hover:bg-hoa-100 dark:bg-hoa-1100/15 dark:text-hoa-700 dark:hover:bg-hoa-1100/25;
}

.speed-option:focus-visible {
  @apply outline-none ring-2 ring-inset ring-hoa-1100;
}
</style>
