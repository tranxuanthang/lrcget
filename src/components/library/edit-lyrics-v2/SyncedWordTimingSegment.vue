<template>
  <div
    ref="segmentElement"
    class="word-segment absolute flex items-center justify-center px-1 py-1 text-sm select-none h-full overflow-visible"
    :class="segmentClass"
    :style="segmentStyle"
    :title="`${word.text} (${formatTimestampMs(startMs)} - ${formatTimestampMs(endMs)})`"
    @mouseenter="handleSegmentHover"
    @mousemove="handleSegmentHover"
    @mouseleave="handleSegmentLeave"
    @dblclick.stop="handleSegmentDoubleClick"
  >
    <div
      ref="previewContainerElement"
      class="relative flex items-center justify-center w-full h-full min-w-0 overflow-hidden"
    >
      <span
        ref="textElement"
        class="relative z-10 block max-w-full overflow-hidden whitespace-nowrap text-clip"
      >
        {{ word.text }}
      </span>

      <div
        v-if="hoverPreview"
        class="absolute inset-y-0 pointer-events-none z-20"
        :style="{ left: `${hoverPreview.splitX}px` }"
      >
        <div
          class="absolute top-0 bottom-0 w-[2px] -translate-x-1/2 bg-neutral-500/80 dark:bg-neutral-200/90 ring-1 ring-neutral-500/20"
        />
      </div>
    </div>

    <div
      v-if="showNextWordHint"
      class="absolute left-1/2 top-full z-40 mt-1 -translate-x-1/2 px-2 py-0.5 rounded-md border border-hoa-1100/40 bg-hoa-1100 text-xs font-medium leading-4 text-white shadow-sm whitespace-nowrap pointer-events-none"
      :style="{ left: `${nextSegmentCenterOffsetPx}px` }"
      :title="`Next word: ${nextWordHintText}`"
    >
      {{ nextWordHintText }}
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { formatTimestampMs } from '@/utils/lyricsfile.js'

const emit = defineEmits(['split-at'])

const props = defineProps({
  word: {
    type: Object,
    required: true,
  },
  wordIndex: {
    type: Number,
    required: true,
  },
  startMs: {
    type: Number,
    default: 0,
  },
  endMs: {
    type: Number,
    default: 0,
  },
  lineStartMs: {
    type: Number,
    default: 0,
  },
  lineEndMs: {
    type: Number,
    default: 0,
  },
  timelineWidth: {
    type: Number,
    default: 0,
  },
  progressMs: {
    type: Number,
    default: 0,
  },
  nextWordText: {
    type: String,
    default: '',
  },
  nextWordStartMs: {
    type: Number,
    default: null,
  },
  nextWordEndMs: {
    type: Number,
    default: null,
  },
  selectedBoundaryIndex: {
    type: Number,
    default: -1,
  },
  selectedBoundaryIndices: {
    type: Array,
    default: () => [],
  },
})

const segmentElement = ref(null)
const previewContainerElement = ref(null)
const textElement = ref(null)
const hoverPreview = ref(null)

const nextWordHintText = computed(() => (props.nextWordText || '').trim())

const nextSegmentWidthPx = computed(() => {
  if (!Number.isFinite(props.timelineWidth) || props.timelineWidth <= 0) {
    return 0
  }

  if (
    !Number.isFinite(props.lineStartMs) ||
    !Number.isFinite(props.lineEndMs) ||
    props.lineEndMs <= props.lineStartMs
  ) {
    return 0
  }

  if (!Number.isFinite(props.nextWordStartMs) || !Number.isFinite(props.nextWordEndMs)) {
    return 0
  }

  const lineDuration = props.lineEndMs - props.lineStartMs
  const nextDuration = Math.max(0, props.nextWordEndMs - props.nextWordStartMs)
  return (nextDuration / lineDuration) * props.timelineWidth
})

const currentSegmentWidthPx = computed(() => {
  if (!Number.isFinite(props.timelineWidth) || props.timelineWidth <= 0) {
    return 0
  }

  if (
    !Number.isFinite(props.lineStartMs) ||
    !Number.isFinite(props.lineEndMs) ||
    props.lineEndMs <= props.lineStartMs
  ) {
    return 0
  }

  if (!Number.isFinite(props.startMs) || !Number.isFinite(props.endMs)) {
    return 0
  }

  const lineDuration = props.lineEndMs - props.lineStartMs
  const currentDuration = Math.max(0, props.endMs - props.startMs)
  return (currentDuration / lineDuration) * props.timelineWidth
})

const nextSegmentCenterOffsetPx = computed(() => {
  // Hint element is positioned relative to current segment.
  // Move to center of the next segment: current width + half next width.
  return currentSegmentWidthPx.value + nextSegmentWidthPx.value / 2
})

const doesNextWordOverflow = computed(() => {
  if (!nextWordHintText.value || nextSegmentWidthPx.value <= 0) {
    return false
  }

  const font = textElement.value ? getComputedStyle(textElement.value).font : '14px sans-serif'
  const textWidth = measureTextWidth(nextWordHintText.value, font)

  // Segment uses horizontal padding (px-1), so reserve a small visual margin.
  const availableWidth = Math.max(0, nextSegmentWidthPx.value - 8)
  return textWidth > availableWidth + 1
})

const isSelectedDividerForNextSegment = computed(() => {
  if (!Number.isFinite(props.nextWordStartMs)) {
    return false
  }

  const nextBoundaryIndex = props.wordIndex + 1

  if (props.selectedBoundaryIndex === nextBoundaryIndex) {
    return true
  }

  return Array.isArray(props.selectedBoundaryIndices)
    ? props.selectedBoundaryIndices.includes(nextBoundaryIndex)
    : false
})

const showNextWordHint = computed(() => {
  return (
    isSelectedDividerForNextSegment.value &&
    doesNextWordOverflow.value &&
    nextWordHintText.value.length > 0
  )
})

const splitTextByGrapheme = text => {
  if (!text || typeof text !== 'string') {
    return []
  }

  if (typeof Intl !== 'undefined' && typeof Intl.Segmenter === 'function') {
    const segmenter = new Intl.Segmenter('und', { granularity: 'grapheme' })
    return Array.from(segmenter.segment(text), item => item.segment)
  }

  return Array.from(text)
}

const measureTextWidth = (text, font) => {
  if (typeof document === 'undefined') {
    return 0
  }

  const canvas = document.createElement('canvas')
  const context = canvas.getContext('2d')

  if (!context) {
    return 0
  }

  context.font = font
  return context.measureText(text).width
}

const getSplitPreview = clientX => {
  if (!segmentElement.value || !previewContainerElement.value || !textElement.value) {
    return null
  }

  const text = props.word.text || ''
  const graphemes = splitTextByGrapheme(text)
  if (graphemes.length <= 1) {
    return null
  }

  const segmentRect = segmentElement.value.getBoundingClientRect()
  const previewContainerRect = previewContainerElement.value.getBoundingClientRect()
  const textRect = textElement.value.getBoundingClientRect()
  if (segmentRect.width <= 0 || previewContainerRect.width <= 0 || textRect.width <= 0) {
    return null
  }

  const textNode = textElement.value.firstChild
  if (textNode && textNode.nodeType === Node.TEXT_NODE) {
    const pointerClientX = Math.max(textRect.left, Math.min(textRect.right, clientX))
    const range = document.createRange()
    const boundaries = []

    for (let index = 1; index < graphemes.length; index++) {
      const offset = graphemes.slice(0, index).join('').length

      range.setStart(textNode, 0)
      range.setEnd(textNode, offset)

      const boundaryRect = range.getBoundingClientRect()
      const boundaryClientX = boundaryRect.right

      if (!Number.isFinite(boundaryClientX)) {
        continue
      }

      boundaries.push({
        splitIndex: index,
        measuredWidth: Math.max(0, boundaryClientX - textRect.left),
        splitX: boundaryClientX - previewContainerRect.left,
      })
    }

    if (boundaries.length > 0) {
      let nearestBoundary = boundaries[0]
      let nearestDistance = Math.abs(pointerClientX - (previewContainerRect.left + nearestBoundary.splitX))

      for (const boundary of boundaries.slice(1)) {
        const distance = Math.abs(pointerClientX - (previewContainerRect.left + boundary.splitX))
        if (distance < nearestDistance) {
          nearestBoundary = boundary
          nearestDistance = distance
        }
      }

      const splitRatio = Math.max(0, Math.min(1, nearestBoundary.measuredWidth / textRect.width))

      return {
        splitIndex: nearestBoundary.splitIndex,
        splitRatio,
        splitX: nearestBoundary.splitX,
      }
    }
  }

  const font = getComputedStyle(textElement.value).font
  const graphemeWidths = graphemes.map(grapheme => measureTextWidth(grapheme, font))
  const totalMeasuredWidth = graphemeWidths.reduce((sum, width) => sum + width, 0)

  if (totalMeasuredWidth <= 0) {
    return null
  }

  const cumulativeBoundaries = []
  let runningWidth = 0
  for (let index = 0; index < graphemeWidths.length - 1; index++) {
    runningWidth += graphemeWidths[index]
    cumulativeBoundaries.push({
      splitIndex: index + 1,
      measuredWidth: runningWidth,
    })
  }

  const pointerOffset = Math.max(0, Math.min(textRect.width, clientX - textRect.left))
  const scaledOffset = (pointerOffset / textRect.width) * totalMeasuredWidth

  let nearestBoundary = cumulativeBoundaries[0]
  let nearestDistance = Math.abs(scaledOffset - nearestBoundary.measuredWidth)

  for (const boundary of cumulativeBoundaries.slice(1)) {
    const distance = Math.abs(scaledOffset - boundary.measuredWidth)
    if (distance < nearestDistance) {
      nearestBoundary = boundary
      nearestDistance = distance
    }
  }

  const splitX =
    (nearestBoundary.measuredWidth / totalMeasuredWidth) * textRect.width +
    (textRect.left - previewContainerRect.left)

  return {
    splitIndex: nearestBoundary.splitIndex,
    splitRatio: nearestBoundary.measuredWidth / totalMeasuredWidth,
    splitX,
  }
}

const isPlaying = computed(() => {
  // The word is playing when: startMs <= currentTime < endMs
  return props.progressMs >= props.startMs && props.progressMs < props.endMs
})

const segmentClass = computed(() => {
  const baseClasses = [
    'bg-neutral-200 dark:bg-neutral-700',
    'text-neutral-800 dark:text-neutral-300',
    'border-r',
    'border-neutral-300',
    'dark:border-neutral-600',
  ]

  if (isPlaying.value) {
    baseClasses.push(
      'bg-hoa-1100',
      'dark:bg-hoa-1100',
      'text-white',
      'dark:text-white',
      'font-bold',
      'border-r',
      'border-hoa-1100',
      'dark:border-hoa-1100'
    )
  }

  return baseClasses
})

const segmentStyle = computed(() => {
  if (!props.timelineWidth || props.lineEndMs <= props.lineStartMs) {
    return {}
  }

  const duration = props.lineEndMs - props.lineStartMs
  const leftPercent = ((props.startMs - props.lineStartMs) / duration) * 100
  const wordDuration = Math.max(0, props.endMs - props.startMs)
  const widthPercent = (wordDuration / duration) * 100

  return {
    left: `${leftPercent}%`,
    width: `${widthPercent}%`,
    transition: 'none',
  }
})

const handleSegmentDoubleClick = event => {
  const splitPreview = getSplitPreview(event.clientX)
  if (!splitPreview) {
    return
  }

  emit('split-at', {
    wordIndex: props.wordIndex,
    splitIndex: splitPreview.splitIndex,
    splitRatio: splitPreview.splitRatio,
  })
}

const handleSegmentHover = event => {
  hoverPreview.value = getSplitPreview(event.clientX)
}

const handleSegmentLeave = () => {
  hoverPreview.value = null
}

</script>

<style scoped>
.word-segment {
  user-select: none;
  touch-action: none;
}
</style>
