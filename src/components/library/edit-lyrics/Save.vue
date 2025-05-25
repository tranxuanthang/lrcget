<template>
  <div>
    <div
      class="fixed top-0 left-0 h-full w-full flex items-center justify-center z-40"
      :class="{ hidden: !props.isShow }"
    >
      <div
        class="px-8 py-4 rounded-lg m-4 bg-white flex flex-col gap-2 shadow-lg"
      >
        a
      </div>
    </div>

    <div
      class="fixed top-0 left-0 h-full w-full z-30 bg-black/10"
      :class="{ hidden: !props.isShow }"
      @click="close"
    ></div>
  </div>
</template>

<script setup>
const emit = defineEmits(["close"]);
const props = defineProps(["isShow"]);

const close = () => {
  emit("close");
};
</script>
