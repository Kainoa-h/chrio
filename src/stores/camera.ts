import { defineStore } from 'pinia';
import { ref } from 'vue';
import { emptyPreferences, parsePreferences, settingsKey, type CameraPreferences } from '@/lib/cameraSettings';

export const useCameraSettings = defineStore('camera-settings', () => {
  let initial = emptyPreferences();
  try { initial = parsePreferences(localStorage.getItem(settingsKey)); } catch { /* Storage may be unavailable. */ }
  const preferences = ref(initial);
  function save(value: CameraPreferences) {
    localStorage.setItem(settingsKey, JSON.stringify(value));
    preferences.value = JSON.parse(JSON.stringify(value));
  }
  return { preferences, save };
});
