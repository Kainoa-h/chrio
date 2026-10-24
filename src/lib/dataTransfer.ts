import { ref } from 'vue';

// A single barrier shared by the global dialog and the active session editor.
export const dataTransferActive = ref(false);
const saveHandlers = new Set<() => Promise<void>>();

export function registerTransferSave(handler: () => Promise<void>) {
  saveHandlers.add(handler);
  return () => { saveHandlers.delete(handler); };
}

export async function flushTransferSaves() {
  dataTransferActive.value = true;
  for (const save of saveHandlers) await save();
}
