<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { Camera, X, Image as ImageIcon, Settings } from 'lucide-vue-next';
import CameraFrame from './CameraFrame.vue';
import { useCamera } from '@/composables/useCamera';
import { useCameraSettings } from '@/stores/camera';
import { clampCrop, defaultProfile, resolutions, type CameraPreferences, type CameraProfile, type Crop, type Resolution } from '@/lib/cameraSettings';

const props = defineProps<{ show: boolean; settingsOnly?: boolean }>();
const emit = defineEmits<{ (e: 'close'): void; (e: 'photo-taken', data: string, crop: Crop | null): void }>();
const store = useCameraSettings();
const camera = useCamera();
const { stream, devices, actual, capabilities, photoCapabilities, error, notice, busy } = camera;
const videoRef = ref<HTMLVideoElement | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const dialogRef = ref<HTMLElement | null>(null);
const editing = ref(false);
const draft = ref<CameraPreferences>(clone(store.preferences));
const activeDevice = ref('');
const savingPhoto = ref(false);
const ready = ref(false);
const message = ref('');
const pendingPhoto = ref<{ data: string; crop: Crop } | null>(null);
let operation = 0;
let previousFocus: HTMLElement | null = null;
function clone<T>(value: T): T { return JSON.parse(JSON.stringify(value)); }
const profile = computed<CameraProfile>(() => draft.value.profiles[activeDevice.value || draft.value.deviceId || 'default'] || defaultProfile());
const aspect = computed(() => actual.value.width && actual.value.height ? actual.value.width / actual.value.height : 16 / 9);
const crop = computed(() => clampCrop(profile.value.crop, aspect.value));
const cropDimensions = computed(() => {
  const width = Math.round(actual.value.width * crop.value.width);
  return `${width} × ${Math.round(width * 1.5)}`;
});
const streamChoices = computed(() => resolutions.filter(r => {
  const c = capabilities.value;
  return r.width >= (c?.width?.min ?? 0) && r.width <= (c?.width?.max ?? Infinity) && r.height >= (c?.height?.min ?? 0) && r.height <= (c?.height?.max ?? Infinity);
}));
const photoChoices = computed(() => {
  const c = photoCapabilities.value;
  if (!c) return [];
  const choices = resolutions.filter(r => r.width >= c.imageWidth.min && r.width <= c.imageWidth.max && r.height >= c.imageHeight.min && r.height <= c.imageHeight.max);
  const max = { width: c.imageWidth.max, height: c.imageHeight.max };
  if (max.width && max.height && !choices.some(r => r.width === max.width && r.height === max.height)) choices.push(max);
  return choices;
});
const resolutionKey = (r: Resolution | null) => r ? `${r.width}x${r.height}` : '';
function parseResolution(value: string): Resolution | null {
  if (!value) return null;
  const [width, height] = value.split('x').map(Number);
  return { width, height };
}
function updateProfile(update: Partial<CameraProfile>) {
  draft.value.profiles[activeDevice.value || draft.value.deviceId || 'default'] = { ...profile.value, ...update };
}
async function start(requestedId = draft.value.deviceId) {
  const token = ++operation;
  ready.value = false;
  await camera.start(requestedId, clone(draft.value.profiles[requestedId || 'default'] || defaultProfile()));
  if (token !== operation || !props.show) return;
  activeDevice.value = stream.value?.getVideoTracks()[0]?.getSettings().deviceId || requestedId;
  if (!draft.value.profiles[activeDevice.value || 'default']) updateProfile(defaultProfile());
  if (activeDevice.value !== requestedId && requestedId && stream.value) {
    const fallbackNotice = notice.value;
    await camera.start(activeDevice.value, clone(profile.value));
    if (token !== operation || !props.show) return;
    notice.value = fallbackNotice;
  }
  await nextTick();
  if (videoRef.value) videoRef.value.srcObject = stream.value;
}
function loaded() {
  const video = videoRef.value;
  if (video?.videoWidth && video.videoHeight) {
    actual.value = { width: video.videoWidth, height: video.videoHeight };
    ready.value = video.readyState >= 2;
  }
}
function chooseCamera(event: Event) {
  draft.value.deviceId = (event.target as HTMLSelectElement).value;
  activeDevice.value = draft.value.deviceId;
  void start();
}
function chooseStream(event: Event) {
  updateProfile({ stream: parseResolution((event.target as HTMLSelectElement).value) });
  void start(activeDevice.value);
}
async function finishSettings(save: boolean) {
  if (save) {
    try {
      updateProfile({ crop: crop.value });
      if (!draft.value.deviceId) draft.value.deviceId = activeDevice.value;
      store.save(draft.value);
    } catch { message.value = 'Could not save preferences on this device. Please try again.'; return; }
  }
  if (props.settingsOnly) { emit('close'); return; }
  draft.value = clone(store.preferences);
  editing.value = false;
  message.value = '';
  await start();
}
async function takePhoto() {
  if (!videoRef.value || !ready.value || busy.value || savingPhoto.value) return;
  savingPhoto.value = true;
  const token = operation;
  const frame = { ...crop.value };
  try {
    const result = await camera.capture(videoRef.value, clone(profile.value));
    if (token !== operation || !props.show) return;
    if (result.fallback) {
      message.value = 'Separate still capture was unavailable or did not match the preview. Captured at streaming resolution.';
      pendingPhoto.value = { data: result.data, crop: frame };
    } else { emit('photo-taken', result.data, frame); emit('close'); }
  } catch (e) { message.value = e instanceof Error ? e.message : String(e); }
  finally { savingPhoto.value = false; }
}
function acceptPhoto() {
  if (!pendingPhoto.value) return;
  emit('photo-taken', pendingPhoto.value.data, pendingPhoto.value.crop);
  emit('close');
}
function importFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const token = operation;
  const reader = new FileReader();
  reader.onload = () => {
    if (token !== operation || !props.show) return;
    emit('photo-taken', reader.result as string, null);
    emit('close');
  };
  reader.onerror = () => { message.value = 'Could not read the selected image.'; };
  reader.readAsDataURL(file);
  input.value = '';
}
async function devicesChanged() {
  try {
    await camera.refreshDevices();
    if (props.show && (!stream.value || !devices.value.some(d => d.deviceId === activeDevice.value))) await start();
  } catch { message.value = 'Could not refresh camera devices.'; }
}
function keyboard(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); if (editing.value) void finishSettings(false); else emit('close'); }
  if (event.key === 'Tab') {
    const nodes = dialogRef.value?.querySelectorAll<HTMLElement>('button:not([disabled]), select:not([disabled]), [tabindex="0"]');
    if (!nodes?.length) return;
    const first = nodes[0], last = nodes[nodes.length - 1];
    if (event.shiftKey && (document.activeElement === first || document.activeElement === dialogRef.value)) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
}
watch(() => props.show, async show => {
  if (show) {
    previousFocus = document.activeElement as HTMLElement | null;
    draft.value = clone(store.preferences);
    activeDevice.value = draft.value.deviceId;
    editing.value = !!props.settingsOnly;
    pendingPhoto.value = null;
    message.value = '';
    navigator.mediaDevices?.addEventListener('devicechange', devicesChanged);
    await nextTick();
    dialogRef.value?.focus();
    if (props.show) await start();
  } else {
    operation++;
    camera.stop();
    navigator.mediaDevices?.removeEventListener('devicechange', devicesChanged);
    previousFocus?.focus();
  }
}, { immediate: true });
onBeforeUnmount(() => { operation++; camera.stop(); navigator.mediaDevices?.removeEventListener('devicechange', devicesChanged); });
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm">
    <section ref="dialogRef" role="dialog" aria-modal="true" aria-labelledby="camera-title" tabindex="-1" class="bg-white rounded-lg shadow-xl w-full max-h-[90vh] overflow-y-auto outline-none" :class="editing ? 'max-w-2xl' : 'max-w-5xl'" @keydown="keyboard">
      <header class="p-4 border-b flex items-center justify-between gap-3">
        <h2 id="camera-title" class="font-semibold">{{ editing ? 'Camera settings' : 'Take photo' }}</h2>
        <div class="flex gap-2">
          <button v-if="!editing" title="Camera settings" aria-label="Camera settings" class="p-2 hover:bg-gray-100 rounded" :disabled="savingPhoto || !!pendingPhoto" @click="editing = true; message = ''"><Settings class="w-5 h-5" /></button>
          <button aria-label="Close" class="p-2 hover:bg-gray-100 rounded" @click="editing ? finishSettings(false) : emit('close')"><X class="w-5 h-5" /></button>
        </div>
      </header>
      <div v-if="editing" class="p-4 space-y-3">
        <label class="block text-sm font-medium">Camera
          <select :value="activeDevice" class="block w-full mt-1 border rounded p-2" :disabled="busy" @change="chooseCamera">
            <option v-if="!devices.length" value="">Default camera</option>
            <option v-for="(device, index) in devices" :key="device.deviceId" :value="device.deviceId">{{ device.label || `Camera ${index + 1}` }}</option>
          </select>
        </label>
        <div class="grid sm:grid-cols-2 gap-3">
          <label class="block text-sm font-medium">Streaming resolution
            <select :value="resolutionKey(profile.stream)" :disabled="busy || !stream" class="block w-full mt-1 border rounded p-2" @change="chooseStream">
              <option value="">Auto</option>
              <option v-if="profile.stream && !streamChoices.some(r => resolutionKey(r) === resolutionKey(profile.stream))" :value="resolutionKey(profile.stream)">{{ profile.stream.width }} × {{ profile.stream.height }} (requested)</option>
              <option v-for="r in streamChoices" :key="resolutionKey(r)" :value="resolutionKey(r)">{{ r.width }} × {{ r.height }}</option>
            </select>
          </label>
          <label class="block text-sm font-medium">Capture resolution
            <select :value="photoCapabilities ? resolutionKey(profile.capture) : ''" :disabled="busy || !photoCapabilities" class="block w-full mt-1 border rounded p-2" @change="updateProfile({ capture: parseResolution(($event.target as HTMLSelectElement).value) })">
              <option value="">{{ photoCapabilities ? 'Highest available' : 'Same as streaming resolution' }}</option>
              <option v-if="photoCapabilities && profile.capture && !photoChoices.some(r => resolutionKey(r) === resolutionKey(profile.capture))" :value="resolutionKey(profile.capture)">{{ profile.capture.width }} × {{ profile.capture.height }} (requested)</option>
              <option v-for="r in photoChoices" :key="resolutionKey(r)" :value="resolutionKey(r)">{{ r.width }} × {{ r.height }}</option>
            </select>
          </label>
        </div>
        <p v-if="!photoCapabilities" class="text-xs text-gray-600">Separate still resolution is unavailable. Photos use the streaming resolution.</p>
        <p v-else class="text-xs text-gray-600">Still resolution is a request. If capture fails or its aspect ratio differs, photos use the streaming resolution.</p>
        <p class="text-sm text-gray-600">Drag the 2:3 frame or its corner to resize. Arrow keys move; Alt + arrows resize.</p>
        <button class="text-sm text-blue-600 underline" @click="updateProfile({ crop: clampCrop(null, aspect) })">Reset framing</button>
      </div>
      <div class="bg-black flex items-start justify-center overflow-hidden">
        <!-- The wrapper defines the source bounds. Keep the video out of layout so
             its intrinsic height cannot stretch the crop coordinate system. -->
        <div class="relative flex-none overflow-hidden" :style="{ width: `min(100%, ${(editing ? 42 : 65) * aspect}vh)`, aspectRatio: String(aspect) }">
          <video ref="videoRef" autoplay playsinline muted class="absolute inset-0 block w-full h-full object-fill" @loadedmetadata="loaded" @loadeddata="loaded" @playing="loaded"></video>
          <CameraFrame v-if="ready && stream" :crop="crop" :aspect="aspect" :editable="editing && !busy" @update="updateProfile({ crop: $event })" />
        </div>
      </div>
      <div class="px-4 py-2 text-sm space-y-1" aria-live="polite">
        <p v-if="busy">Connecting to camera…</p>
        <p v-if="error" class="text-red-700">{{ error }} <button class="underline" @click="start()">Retry</button></p>
        <p v-if="notice" class="text-amber-800">{{ notice }}</p>
        <p v-if="message" class="text-amber-800">{{ message }}</p>
        <p v-if="ready && stream" class="text-gray-600">Streaming: {{ actual.width }} × {{ actual.height }} · Framed stream: {{ cropDimensions }} px</p>
      </div>
      <footer class="p-4 border-t flex items-center justify-end gap-3">
        <template v-if="editing">
          <button class="px-4 py-2 border rounded" @click="finishSettings(false)">Cancel</button>
          <button class="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50" :disabled="busy || !ready || !stream" @click="finishSettings(true)">Save settings</button>
        </template>
        <template v-else-if="pendingPhoto">
          <button class="px-4 py-2 border rounded" @click="pendingPhoto = null; message = ''">Retake</button>
          <button class="px-4 py-2 bg-blue-600 text-white rounded" @click="acceptPhoto">Use photo</button>
        </template>
        <template v-else>
          <button class="p-3 rounded hover:bg-gray-100" title="Import image" aria-label="Import image" :disabled="savingPhoto" @click="fileInput?.click()"><ImageIcon class="w-6 h-6" /></button>
          <button class="px-5 py-3 rounded bg-blue-600 text-white flex gap-2 disabled:opacity-50" :disabled="!ready || !stream || busy || savingPhoto || !!error" @click="takePhoto"><Camera class="w-6 h-6" />{{ savingPhoto ? 'Capturing…' : 'Take photo' }}</button>
        </template>
      </footer>
      <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="importFile">
    </section>
  </div>
</template>
