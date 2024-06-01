<template>
	<div ref="parentRef" class="p-4 overflow-y-auto h-full" v-show="props.isActive">
		<div :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }">
			<div class="w-full">
				<div class="w-full flex">
					<div class="text-xs text-brave-30/70 font-bold flex w-full">
						<div class="text-left flex-none w-[65%] p-1">Track</div> <!-- Adjusted width percentage -->
						<div class="text-right flex-none w-[10%] p-1">Duration</div>
						<div class="text-center flex-none w-[10%] p-1">Lyrics</div>
						<div class="text-right flex-none w-[15%] p-1"></div>
					</div>
				</div>
				<div class="w-full flex flex-col">
					<div v-for="virtualRow in virtualRows" :key="virtualRow.index" class="group flex flex-col w-full"
						:style="{
							position: 'absolute',
							top: 0,
							left: 0,
							width: '100%',
							height: `${virtualRow.size}px`,
							transform: `translateY(${virtualRow.start}px)`,
						}">
						<TrackItem :trackId="virtualRow.key" @play-track="playTrack"
							@download-lyrics="downloadLyrics" />
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import TrackItem from './track-list/TrackItem.vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useSearchLibrary } from '../../composables/search';

const props = defineProps(['isActive'])
const emit = defineEmits(['playTrack', 'downloadLyrics'])

const unfilteredTrackIds = ref([])
const trackIds = ref([])

const searchPass = async () => {
	trackIds.value = []
	const res = await useSearchLibrary().filter(unfilteredTrackIds.value)
	trackIds.value = res
}

const parentRef = ref(null)

const rowVirtualizer = useVirtualizer(
	computed(() => ({
		count: trackIds.value.length,
		getScrollElement: () => parentRef.value,
		estimateSize: () => 52,
		overscan: 5,
		paddingStart: 32,
		getItemKey: (index) => trackIds.value[index]
	}))
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems());

const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

const playTrack = (track) => {
	emit('playTrack', track)
}

const downloadLyrics = (track) => {
	emit('downloadLyrics', track)
}

const getTrackIds = async () => {
	unfilteredTrackIds.value = await invoke('get_track_ids')
	await searchPass()
}


onMounted(async () => {
	if (props.isActive) {
		await getTrackIds()
	}
})

watch(() => props.isActive, async () => {
	if (props.isActive) {
		await getTrackIds()
	}
})

watch(() => useSearchLibrary().searchValue.value, async () => {
	try {
		await getTrackIds()
	} catch (error) {
		console.error(error)
	}
})
</script>
