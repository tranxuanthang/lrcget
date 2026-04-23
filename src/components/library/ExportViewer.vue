<template>
  <BaseModal
    content-class="w-full h-[80vh] max-w-screen-md"
    body-class="flex flex-col h-full min-h-0 justify-between gap-6"
    :title="isFinished ? 'Export Complete' : 'Exporting Lyrics'"
    @close="checkAndClose"
  >
    <div class="flex flex-col items-center justify-center gap-1">
      <div class="w-full bg-brave-95 h-1 rounded">
        <div class="bg-brave-30 h-1" :style="{ width: progressWidth }" />
      </div>
      <div class="text-[0.7rem] text-brave-30/60 dark:text-brave-95/60 flex gap-3">
        <span>{{ exportedCount }} EXPORTED</span>
        <span>{{ skippedCount }} SKIPPED</span>
        <span>{{ errorCount }} ERRORS</span>
        <span>{{ totalCount }} TOTAL</span>
      </div>
    </div>

    <div class="rounded-lg p-3 bg-brave-98 dark:bg-brave-1 w-full text-xs grow overflow-auto">
      <div v-for="(logItem, index) in log" :key="index" class="mb-1">
        <div
          :class="{
            'text-green-800 dark:text-green-400': logItem.status === 'exported',
            'text-yellow-800 dark:text-yellow-400': logItem.status === 'skipped',
            'text-red-800 dark:text-red-400': logItem.status === 'error',
          }"
        >
          <strong>{{ logItem.title }} - {{ logItem.artistName }}</strong
          >:
          <span>{{ logItem.message }}</span>
        </div>
        <!-- Show per-format details if available -->
        <div
          v-if="logItem.details && logItem.details.length > 0"
          class="pl-4 mt-0.5 text-brave-30/70 dark:text-brave-90/70"
        >
          <div
            v-for="detail in logItem.details"
            :key="detail.format"
            :class="getDetailStatusClass(detail)"
          >
            • {{ detail.format }}: {{ getDetailMessage(detail) }}
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex-none flex justify-center">
        <button
          v-if="isFinished"
          class="button button-primary px-8 py-2 rounded-full"
          @click="checkAndClose"
        >
          Finish
        </button>
        <button v-else class="button button-normal px-8 py-2 rounded-full" @click="handleStop">
          Stop
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { onUnmounted, computed } from 'vue'
import { useExporter } from '@/composables/export.js'

const {
  isExporting,
  exportProgress,
  exportedCount,
  skippedCount,
  errorCount,
  totalCount,
  log,
  startOver,
  stopExporting,
} = useExporter()

const getDetailStatusClass = detail => {
  const statusType = detail.status?.type
  if (statusType === 'success') {
    return 'text-green-700 dark:text-green-500'
  } else if (statusType === 'skipped') {
    return 'text-yellow-700 dark:text-yellow-500'
  } else if (statusType === 'error') {
    return 'text-red-700 dark:text-red-500'
  }
  return 'text-brave-30/70 dark:text-brave-90/70'
}

const getDetailMessage = detail => {
  const statusType = detail.status?.type
  if (statusType === 'success') {
    return 'exported successfully'
  } else if (statusType === 'skipped' || statusType === 'error') {
    return detail.status?.message || 'unknown'
  }
  return 'unknown status'
}

const emit = defineEmits(['close'])

const progressWidth = computed(() => {
  if (!isExporting.value) {
    return '100%'
  }

  if (exportProgress.value >= 1.0) {
    return '100%'
  }

  return `${exportProgress.value * 100}%`
})

const isFinished = computed(() => {
  if (!isExporting.value) return true
  if (totalCount.value === 0) return false
  return exportedCount.value + skippedCount.value + errorCount.value >= totalCount.value
})

const handleStop = () => {
  stopExporting()
  emit('close')
}

const checkAndClose = () => {
  if (isFinished.value) {
    startOver()
    emit('close')
  } else {
    emit('close')
  }
}

onUnmounted(() => {
  checkAndClose()
})
</script>
