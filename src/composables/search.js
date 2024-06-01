import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { distance, closest } from 'fastest-levenshtein'

const searchValue = ref("")

const DEFAULT_MAX_LEV_DIST = 1

export function useSearchLibrary() {
	const setSearch = (text) => {
		searchValue.value = text
	}

	const matches = (title, maxdist = DEFAULT_MAX_LEV_DIST) => {
		const search = searchValue.value
		if (!search) {
			return true
		}
		if (!title) {
			return false
		}

		const titleLower = title.toLowerCase()
		const searchLower = search.toLowerCase()

		const globalContains = titleLower.includes(searchLower)
		if (globalContains) {
			return true
		}
		const dist = distance(titleLower, searchLower)
		return dist <= maxdist
	}

	const filter = async (trackIdList, maxLevDist = DEFAULT_MAX_LEV_DIST) => {
		const search = searchValue.value
		if (!search) {
			return trackIdList
		}

		const promises = trackIdList.map(
			async (id) => {
				const track = await invoke('get_track', { trackId: id })
				if (!track) {
					return false
				}
				if ((track.title ?? "").includes(search)) {
					return true
				}
				// Probably more performant than an OR
				const titleMatch = matches(track.title ?? "", maxLevDist)
				if (titleMatch) {
					return true
				}
				const artistMatch = matches(track.artist_name ?? "", maxLevDist)
				if (artistMatch) {
					return true
				}
				const albumMatch = matches(track.album_name ?? "", maxLevDist)
				if (albumMatch) {
					return true
				}
				return false
			}
		)
		const filtered = await Promise.all(promises).then((results) => {
			return trackIdList.filter((_, i) => results[i])
		})
		return filtered
	}

	return {
		searchValue,
		setSearch,
		matches,
		filter
	}
}
