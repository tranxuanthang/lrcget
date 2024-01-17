<script setup>
  import { MenuDown } from 'mdue';
  import { ref } from 'vue';

const props = defineProps({
  text: {
    type: String,
    required: true
  },
  mainAction: {
    type: Function,
    required: true
  },
  mainDisabled: {
    type: Boolean,
    required: false
  },
  title: {
    type: String,
    required: false
  },
  popupDisabled: {
    type: Boolean,
    required: false
  }
})

</script>

<template>
  <div class="flex w-fit relative">
    <button 
      v-if="!mainDisabled"
      @click="mainAction" 
      :title="title" 
      class="button button-normal text-sm pl-5 pr-2.5 py-1.5 h-8 w-fit rounded-full rounded-r-none whitespace-nowrap"
    >
      {{ text }}
    </button>
    <button 
      v-else
      :title="title" 
      class="button button-disabled !border-r-0 text-sm pl-5 pr-2.5 py-1.5 h-8 w-fit rounded-full rounded-r-none whitespace-nowrap"
    >
      {{ text }}
    </button>
    <button 
      v-if="!popupDisabled"
      ref="popupToggleButton" 
      @click="isDropdownOpen = !isDropdownOpen" 
      :class="{'dropdown-arrow-button-active': isDropdownOpen}" 
      class="button dropdown-arrow-button border-normal flex align-middle items-center text-sm h-8 border-l pr-1 w-fit rounded-full rounded-l-none whitespace-nowrap"
    >
      <MenuDown class="text-2xl" />
    </button>
    <button 
      v-else
      ref="popupToggleButton" 
      :class="{'dropdown-arrow-button-active': isDropdownOpen}" 
      class="button dropdown-arrow-button-disabled border-normal flex align-middle items-center text-sm h-8 border-l pr-1 w-fit rounded-full rounded-l-none whitespace-nowrap"
    >
      <MenuDown class="text-2xl" />
    </button>
    <div 
      v-click-outside="clickOutsideHandler"
      v-show="isDropdownOpen"
      class="dropdown-popup">
        <ul class="flex flex-col gap-0.5 w-full">
          <slot/>
      </ul>
    </div>
  </div>
</template>

<script>
const popupToggleButton = ref(null)
const isDropdownOpen = ref(false)

const clickOutsideHandler = (event) => {
  // if clicked on the toggle button, do nothing
  if (popupToggleButton.value.contains(event.target)) return;
  
  // if clicked outside the dropdown, close it
  isDropdownOpen.value = false
}

</script>