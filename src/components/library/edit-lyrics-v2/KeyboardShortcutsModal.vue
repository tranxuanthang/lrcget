<template>
  <BaseModal
    title="Keyboard Shortcuts"
    :esc-to-close="true"
    :click-to-close="true"
    content-class="w-full max-w-md max-h-[80vh]"
    body-class="overflow-auto"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-5 mx-3">
      <div class="flex items-center justify-between gap-2">
        <span class="text-xs text-neutral-500 dark:text-neutral-400">
          {{ isConfigMode ? 'Press Change, then press the new shortcut' : 'View all shortcuts' }}
        </span>
        <div class="flex items-center gap-2">
          <button
            class="button button-normal p-1.5 rounded h-7 w-7 inline-flex items-center justify-center"
            :title="isConfigMode ? 'Done configuring shortcuts' : 'Configure shortcuts'"
            :aria-label="isConfigMode ? 'Done configuring shortcuts' : 'Configure shortcuts'"
            @click="isConfigMode = !isConfigMode"
          >
            <Check v-if="isConfigMode" class="w-4 h-4" />
            <Tune v-else class="w-4 h-4" />
          </button>
          <button
            v-if="isConfigMode"
            class="button button-normal p-1.5 rounded h-7 w-7 inline-flex items-center justify-center"
            :class="resetAllButtonClass"
            title="Reset all shortcuts"
            aria-label="Reset all shortcuts"
            @click="handleResetAll"
          >
            <Restore class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div
        v-if="isConfigMode && duplicateGroups.length > 0"
        class="rounded-md border border-amber-300/70 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/30 px-3 py-2"
      >
        <p class="text-xs font-medium text-amber-800 dark:text-amber-200">
          Duplicate shortcuts detected ({{ duplicateGroups.length }}). Conflicting shortcuts are highlighted below.
        </p>
      </div>

      <div
        v-for="category in shortcutCategories"
        :key="category.id"
        class="flex flex-col gap-2"
      >
        <h3
          class="text-xs font-bold uppercase tracking-wider"
          :class="
            isCategoryActive(category.id)
              ? 'text-hoa-800 dark:text-hoa-200'
              : 'text-neutral-500 dark:text-neutral-400'
          "
        >
          {{ category.label }}
          <span
            v-if="isCategoryActive(category.id)"
            class="ml-1.5 inline-block w-1.5 h-1.5 rounded-full bg-hoa-500"
          />
        </h3>

        <div class="flex flex-col gap-1.5">
          <div
            v-for="(shortcut, index) in category.shortcuts"
            :key="index"
            class="flex items-start justify-between gap-4"
          >
            <div class="flex flex-col gap-0.5">
              <span class="text-sm text-neutral-700 dark:text-neutral-300">{{ shortcut.description }}</span>
              <span
                v-if="isConfigMode && shortcut.id && hasDuplicateShortcut(shortcut.id)"
                class="text-xs text-amber-700 dark:text-amber-300"
              >
                Conflicts with: {{ getDuplicateConflictText(shortcut.id) }}
              </span>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <div class="flex items-center gap-1">
                <span
                  v-for="(key, keyIndex) in shortcut.keys"
                  :key="keyIndex"
                  class="inline-flex items-center justify-center px-1.5 py-0.5 rounded border text-neutral-700 dark:text-neutral-300 font-mono text-xs leading-none min-w-[1.5rem]"
                  :class="
                    isConfigMode && shortcut.id && hasDuplicateShortcut(shortcut.id)
                      ? 'bg-amber-100 dark:bg-amber-950/40 border-amber-300 dark:border-amber-700 text-amber-800 dark:text-amber-200'
                      : shortcut.id && isShortcutModified(shortcut)
                        ? 'bg-hoa-100 dark:bg-hoa-1200/40 border-hoa-300 dark:border-hoa-800 text-hoa-900 dark:text-hoa-200'
                      : 'bg-neutral-100 dark:bg-neutral-700 border-neutral-200 dark:border-neutral-600'
                  "
                >
                  {{ key }}
                </span>
              </div>

              <template v-if="isConfigMode">
                <template v-if="shortcut.id">
                  <button
                    class="button button-normal p-1 rounded h-6 w-6 inline-flex items-center justify-center"
                    :class="changeButtonClass(shortcut.id)"
                    :title="
                      captureShortcutId === shortcut.id
                        ? 'Press keys for new shortcut'
                        : `Change shortcut for ${shortcut.description}`
                    "
                    :aria-label="
                      captureShortcutId === shortcut.id
                        ? 'Press keys for new shortcut'
                        : `Change shortcut for ${shortcut.description}`
                    "
                    @click="startCapture(shortcut.id)"
                  >
                    <Loading v-if="isListeningShortcut(shortcut.id)" class="w-4 h-4 animate-spin" />
                    <Keyboard v-else class="w-4 h-4" />
                  </button>
                  <button
                    class="button button-normal p-1 rounded h-6 w-6 inline-flex items-center justify-center"
                    :class="resetButtonClass(shortcut.id, shortcut)"
                    :title="`Reset shortcut for ${shortcut.description}`"
                    :aria-label="`Reset shortcut for ${shortcut.description}`"
                    @click="handleResetShortcut(shortcut.id)"
                  >
                    <Restore class="w-4 h-4" />
                  </button>
                </template>
                <template v-else>
                  <span class="h-6 w-6 inline-block" aria-hidden="true" />
                  <span class="h-6 w-6 inline-block" aria-hidden="true" />
                </template>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Tune from '~icons/mdi/tune-variant'
import Check from '~icons/mdi/check'
import Restore from '~icons/mdi/restore'
import Keyboard from '~icons/mdi/keyboard-settings-outline'
import Loading from '~icons/mdi/loading'
import { getKeyboardShortcutCategories } from '@/composables/edit-lyrics-v2/keyboardShortcuts.js'
import {
  resetAllShortcutOverrides,
  resetShortcutOverride,
  setShortcutOverride,
  shortcutKeysFromKeyboardEvent,
} from '@/composables/edit-lyrics-v2/shortcutRegistry.js'

const props = defineProps({
  activeTab: {
    type: String,
    required: true,
  },
})

const emit = defineEmits(['close'])

const isConfigMode = ref(false)
const captureShortcutId = ref('')
const refreshVersion = ref(0)
const shortcutButtonStates = ref({})
const resetAllButtonState = ref('idle')
const buttonStateTimeouts = new Map()

const shortcutCategories = computed(() => {
  refreshVersion.value
  return getKeyboardShortcutCategories()
})

const configurableShortcuts = computed(() => {
  return shortcutCategories.value.flatMap(category =>
    category.shortcuts
      .filter(shortcut => typeof shortcut.id === 'string' && shortcut.id.length > 0)
      .map(shortcut => ({
        id: shortcut.id,
        description: shortcut.description,
        keys: shortcut.keys,
      }))
  )
})

const duplicateGroups = computed(() => {
  const byCombo = new Map()

  for (const shortcut of configurableShortcuts.value) {
    const keys = Array.isArray(shortcut.keys) ? shortcut.keys : []
    if (keys.length === 0) {
      continue
    }

    const combo = keys.join('+')
    const group = byCombo.get(combo) || []
    group.push(shortcut)
    byCombo.set(combo, group)
  }

  return Array.from(byCombo.entries())
    .filter(([, shortcuts]) => shortcuts.length > 1)
    .map(([combo, shortcuts]) => ({ combo, shortcuts }))
})

const duplicateConflictMap = computed(() => {
  const conflictMap = {}

  for (const group of duplicateGroups.value) {
    for (const shortcut of group.shortcuts) {
      conflictMap[shortcut.id] = group.shortcuts
        .filter(other => other.id !== shortcut.id)
        .map(other => other.description)
    }
  }

  return conflictMap
})

const hasDuplicateShortcut = shortcutId => {
  return Array.isArray(duplicateConflictMap.value[shortcutId])
}

const getDuplicateConflictText = shortcutId => {
  const conflicts = duplicateConflictMap.value[shortcutId] || []
  return conflicts.join(', ')
}

const areShortcutKeysEqual = (firstKeys, secondKeys) => {
  if (!Array.isArray(firstKeys) || !Array.isArray(secondKeys)) {
    return false
  }

  if (firstKeys.length !== secondKeys.length) {
    return false
  }

  return firstKeys.every((key, index) => key === secondKeys[index])
}

const isShortcutModified = shortcut => {
  if (!shortcut?.id || !Array.isArray(shortcut.keys) || !Array.isArray(shortcut.defaultKeys)) {
    return false
  }

  return !areShortcutKeysEqual(shortcut.keys, shortcut.defaultKeys)
}

const setShortcutButtonState = (shortcutId, buttonType, state, timeoutMs = 0) => {
  const nextState = {
    ...(shortcutButtonStates.value[shortcutId] || {}),
    [buttonType]: state,
  }
  shortcutButtonStates.value = {
    ...shortcutButtonStates.value,
    [shortcutId]: nextState,
  }

  const timeoutKey = `${shortcutId}:${buttonType}`
  const existingTimeout = buttonStateTimeouts.get(timeoutKey)
  if (existingTimeout) {
    clearTimeout(existingTimeout)
    buttonStateTimeouts.delete(timeoutKey)
  }

  if (timeoutMs > 0) {
    const timeoutId = setTimeout(() => {
      const current = shortcutButtonStates.value[shortcutId] || {}
      shortcutButtonStates.value = {
        ...shortcutButtonStates.value,
        [shortcutId]: {
          ...current,
          [buttonType]: 'idle',
        },
      }
      buttonStateTimeouts.delete(timeoutKey)
    }, timeoutMs)
    buttonStateTimeouts.set(timeoutKey, timeoutId)
  }
}

const setResetAllButtonState = (state, timeoutMs = 0) => {
  resetAllButtonState.value = state

  const timeoutKey = '__reset_all__'
  const existingTimeout = buttonStateTimeouts.get(timeoutKey)
  if (existingTimeout) {
    clearTimeout(existingTimeout)
    buttonStateTimeouts.delete(timeoutKey)
  }

  if (timeoutMs > 0) {
    const timeoutId = setTimeout(() => {
      resetAllButtonState.value = 'idle'
      buttonStateTimeouts.delete(timeoutKey)
    }, timeoutMs)
    buttonStateTimeouts.set(timeoutKey, timeoutId)
  }
}

const changeButtonClass = shortcutId => {
  const state = shortcutButtonStates.value[shortcutId]?.change || 'idle'

  if (state === 'listening') {
    return '!bg-hoa-600 !text-white hover:!bg-hoa-700 dark:!bg-hoa-500 hover:dark:!bg-hoa-400 ring-2 ring-hoa-300/80 dark:ring-hoa-500/70'
  }

  if (state === 'success') {
    return '!bg-emerald-200 !text-emerald-900 hover:!bg-emerald-300 dark:!bg-emerald-800 dark:!text-emerald-100 hover:dark:!bg-emerald-700'
  }

  if (state === 'warning') {
    return '!bg-amber-200 !text-amber-900 hover:!bg-amber-300 dark:!bg-amber-800 dark:!text-amber-100 hover:dark:!bg-amber-700'
  }

  return ''
}

const isListeningShortcut = shortcutId => {
  return (shortcutButtonStates.value[shortcutId]?.change || 'idle') === 'listening'
}

const resetButtonClass = (shortcutId, shortcut) => {
  const state = shortcutButtonStates.value[shortcutId]?.reset || 'idle'

  if (state === 'success') {
    return '!bg-emerald-200 !text-emerald-900 hover:!bg-emerald-300 dark:!bg-emerald-800 dark:!text-emerald-100 hover:dark:!bg-emerald-700'
  }

  if (isShortcutModified(shortcut)) {
    return '!bg-hoa-100 !text-hoa-900 hover:!bg-hoa-200 dark:!bg-hoa-1200/40 dark:!text-hoa-200 hover:dark:!bg-hoa-1100/50'
  }

  return ''
}

const resetAllButtonClass = computed(() => {
  if (resetAllButtonState.value === 'success') {
    return '!bg-emerald-200 !text-emerald-900 hover:!bg-emerald-300 dark:!bg-emerald-800 dark:!text-emerald-100 hover:dark:!bg-emerald-700'
  }

  return ''
})

const startCapture = shortcutId => {
  if (!shortcutId) {
    return
  }

  if (captureShortcutId.value === shortcutId) {
    captureShortcutId.value = ''
    setShortcutButtonState(shortcutId, 'change', 'warning', 1200)
    return
  }

  if (captureShortcutId.value) {
    setShortcutButtonState(captureShortcutId.value, 'change', 'idle')
  }

  captureShortcutId.value = shortcutId
  setShortcutButtonState(shortcutId, 'change', 'listening')
}

const handleResetShortcut = shortcutId => {
  if (!shortcutId) {
    return
  }

  resetShortcutOverride(shortcutId)
  if (captureShortcutId.value === shortcutId) {
    captureShortcutId.value = ''
  }
  setShortcutButtonState(shortcutId, 'change', 'idle')
  setShortcutButtonState(shortcutId, 'reset', 'success', 1400)
  refreshVersion.value += 1
}

const handleResetAll = () => {
  resetAllShortcutOverrides()
  captureShortcutId.value = ''
  shortcutButtonStates.value = {}
  setResetAllButtonState('success', 1400)
  refreshVersion.value += 1
}

const handleCaptureKeydown = event => {
  if (!captureShortcutId.value) {
    return
  }

  event.preventDefault()
  event.stopPropagation()

  if (event.key === 'Escape') {
    const canceledShortcutId = captureShortcutId.value
    captureShortcutId.value = ''
    if (canceledShortcutId) {
      setShortcutButtonState(canceledShortcutId, 'change', 'warning', 1200)
    }
    return
  }

  const keys = shortcutKeysFromKeyboardEvent(event)
  if (keys.length === 0) {
    return
  }

  const currentShortcutId = captureShortcutId.value
  setShortcutOverride(currentShortcutId, keys)
  setShortcutButtonState(currentShortcutId, 'change', 'success', 1500)
  captureShortcutId.value = ''
  refreshVersion.value += 1
}

onMounted(() => {
  document.addEventListener('keydown', handleCaptureKeydown, true)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleCaptureKeydown, true)

  for (const timeoutId of buttonStateTimeouts.values()) {
    clearTimeout(timeoutId)
  }
  buttonStateTimeouts.clear()
})

const isCategoryActive = categoryId => {
  if (categoryId === 'global') return true
  if (props.activeTab === 'plain') return categoryId === 'plain'
  if (props.activeTab === 'synced') return categoryId === 'synced' || categoryId === 'wordTiming'
  return false
}
</script>
