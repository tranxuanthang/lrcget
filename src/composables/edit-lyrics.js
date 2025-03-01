import { ref } from "vue";
import { useModal } from "vue-final-modal";
import EditLyrics from "@/components/library/EditLyrics.vue";

const editingTrack = ref(null);

export function useEditLyrics() {
  const editLyrics = (track) => {
    editingTrack.value = track;
    openModal();
  };

  const { open: openModal, close: closeModal } = useModal({
    component: EditLyrics,
    attrs: {
      onClose() {
        closeModal();
      },
      onClosed() {
        editingTrack.value = null;
      },
    },
  });

  return {
    editingTrack,
    editLyrics,
  };
}
