<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue || isOpen" class="modal-backdrop" @click.self="handleBackdropClick">
        <div class="modal-container">
          <div :class="['modal-content', contentClass]">
            <div class="modal-header">
              <slot name="header">
                <h2 v-if="title" class="modal-title">{{ title }}</h2>
              </slot>
              <button v-if="closeButton" class="modal-close" @click="close">
                <span class="sr-only">Close</span>
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div class="modal-body">
              <slot></slot>
            </div>
            <div class="modal-footer">
              <slot name="footer"></slot>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch, onMounted, onUnmounted } from 'vue';

const isOpen = ref(false);

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: undefined
  },
  clickToClose: {
    type: Boolean,
    default: true
  },
  escToClose: {
    type: Boolean,
    default: true
  },
  closeButton: {
    type: Boolean,
    default: true
  },
  title: {
    type: String,
    default: ''
  },
  contentClass: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['update:modelValue', 'close']);

const handleBackdropClick = () => {
  if (props.clickToClose) {
    close();
  }
};

const handleEsc = (e) => {
  if (e.key === 'Escape' && props.escToClose && (props.modelValue || isOpen.value)) {
    close();
  }
};

const close = () => {
  if (props.modelValue !== undefined) {
    emit('update:modelValue', false);
  } else {
    isOpen.value = false;
  }
  emit('close');
};

watch(() => props.modelValue, (newVal) => {
  if (newVal === undefined) return;
  isOpen.value = newVal;
});

onMounted(() => {
  if (props.modelValue === undefined) {
    isOpen.value = true;
  }
  document.addEventListener('keydown', handleEsc);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleEsc);
});
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-container {
  max-width: 90%;
  max-height: 90%;
  box-sizing: border-box;
}

.modal-content {
  background-color: white;
  border-radius: 0.5rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

:global(.dark) .modal-content {
  background-color: var(--brave-background-modal-dark, #140000);
  color: white;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

:global(.dark) .modal-header {
  border-color: rgba(255, 255, 255, 0.1);
}

.modal-title {
  font-size: 1.25rem;
  font-weight: bold;
}

.modal-close {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 0.25rem;
  transition: background-color 0.2s;
}

.modal-close:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

:global(.dark) .modal-close:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  padding: 1rem;
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid rgba(0, 0, 0, 0.1);
}

:global(.dark) .modal-footer {
  border-color: rgba(255, 255, 255, 0.1);
}

.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
</style>
