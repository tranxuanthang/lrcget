import { ref, computed } from 'vue'

const isHotkeyState = ref(true)

export function useGlobalState() {
  const disableHotkey = () => {
    console.log('disabled hotkey!')
    isHotkeyState.value = false
  }
  const enableHotkey = () => {
    console.log('enabled hotkey!')
    isHotkeyState.value = true
  }
  const isHotkey = computed(() => isHotkeyState.value)

  return {
    isHotkey,
    disableHotkey,
    enableHotkey
  }
}
