import { ref } from 'vue'

const pendingNavigation = ref(null)
// { type: 'album' | 'artist', id: number }

export function useLibraryNavigation() {
  const navigateToAlbum = id => {
    pendingNavigation.value = { type: 'album', id }
  }

  const navigateToArtist = id => {
    pendingNavigation.value = { type: 'artist', id }
  }

  const clearNavigation = () => {
    pendingNavigation.value = null
  }

  return {
    pendingNavigation,
    navigateToAlbum,
    navigateToArtist,
    clearNavigation,
  }
}
