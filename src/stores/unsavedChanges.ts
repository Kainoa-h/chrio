import { defineStore } from "pinia";
import { ref } from "vue";

export const useUnsavedChangesStore = defineStore("unsavedChanges", () => {
  const hasUnsavedChanges = ref(false);
  const closeRequested = ref(false);

  function setDirty(value: boolean) {
    hasUnsavedChanges.value = value;
  }

  function requestClose() {
    closeRequested.value = true;
  }

  function clearCloseRequest() {
    closeRequested.value = false;
  }

  return {
    hasUnsavedChanges,
    closeRequested,
    setDirty,
    requestClose,
    clearCloseRequest,
  };
});
