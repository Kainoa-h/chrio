import { ref, shallowRef } from 'vue';
import type { CameraProfile } from '@/lib/cameraSettings';

export function useCamera() {
  const stream = shallowRef<MediaStream | null>(null);
  const devices = ref<MediaDeviceInfo[]>([]);
  const actual = ref({ width: 0, height: 0 });
  const capabilities = ref<MediaTrackCapabilities | null>(null);
  const photoCapabilities = ref<{ imageWidth: { min: number; max: number }; imageHeight: { min: number; max: number } } | null>(null);
  const error = ref('');
  const notice = ref('');
  const busy = ref(false);
  let generation = 0;
  let queue: Promise<void> = Promise.resolve();
  let imageCapture: ImageCapture | null = null;

  async function refreshDevices() {
    if (!navigator.mediaDevices) return;
    devices.value = (await navigator.mediaDevices.enumerateDevices()).filter(d => d.kind === 'videoinput');
  }
  function stop() {
    generation++;
    stream.value?.getTracks().forEach(t => t.stop());
    stream.value = null;
    imageCapture = null;
    actual.value = { width: 0, height: 0 };
    capabilities.value = null;
    photoCapabilities.value = null;
    busy.value = false;
  }
  function start(deviceId: string, profile: CameraProfile): Promise<void> {
    stop();
    const token = generation;
    busy.value = true;
    error.value = '';
    notice.value = '';
    const resolution = profile.stream ? { width: { ideal: profile.stream.width }, height: { ideal: profile.stream.height } } : {};
    const run = async () => {
      if (token !== generation) return;
      let acquired: MediaStream | null = null;
      try {
        if (!navigator.mediaDevices?.getUserMedia) throw new Error('Camera access is unavailable in this environment.');
        try { await refreshDevices(); } catch { /* Permission may be required before enumeration. */ }
        if (token !== generation) return;
        try {
          acquired = await navigator.mediaDevices.getUserMedia({ audio: false, video: { ...resolution, ...(deviceId ? { deviceId: { exact: deviceId } } : { facingMode: 'environment' }) } });
        } catch (e) {
          if (token !== generation) return;
          if (!deviceId || !(e instanceof DOMException) || !['NotFoundError', 'OverconstrainedError'].includes(e.name)) throw e;
          acquired = await navigator.mediaDevices.getUserMedia({ audio: false, video: resolution });
          if (token === generation) notice.value = 'The preferred camera is unavailable. Using an available camera; your saved profile is retained.';
        }
        if (token !== generation) { acquired.getTracks().forEach(t => t.stop()); return; }
        stream.value = acquired;
        const track = acquired.getVideoTracks()[0];
        const settings = track.getSettings();
        actual.value = { width: settings.width || 0, height: settings.height || 0 };
        try { capabilities.value = track.getCapabilities?.() || null; } catch { capabilities.value = null; }
        if ('ImageCapture' in window) {
          try {
            const capture = new ImageCapture(track);
            const caps = await capture.getPhotoCapabilities();
            const width = caps.imageWidth;
            const height = caps.imageHeight;
            if (token === generation && width?.max && height?.max) {
              imageCapture = capture;
              photoCapabilities.value = { imageWidth: { min: width.min ?? 1, max: width.max }, imageHeight: { min: height.min ?? 1, max: height.max } };
            }
          } catch { /* Separate still capture is optional. */ }
        }
        if (token === generation) {
          track.addEventListener('ended', () => { if (token === generation) { stop(); error.value = 'The camera disconnected. Reconnect it or select another camera.'; } }, { once: true });
          try { await refreshDevices(); } catch { /* Keep the working preview if device enumeration fails. */ }
        }
      } catch (e) {
        acquired?.getTracks().forEach(t => t.stop());
        if (token === generation) { stream.value = null; error.value = `Could not access camera: ${e instanceof Error ? e.message : String(e)}`; }
      } finally { if (token === generation) busy.value = false; }
    };
    queue = queue.then(run, run);
    return queue;
  }
  async function capture(video: HTMLVideoElement, profile: CameraProfile): Promise<{ data: string; fallback: boolean }> {
    if (!stream.value || video.readyState < 2 || !video.videoWidth) throw new Error('Wait for the camera preview to be ready.');
    const token = generation;
    let source: CanvasImageSource = video;
    let width = video.videoWidth;
    let height = video.videoHeight;
    let bitmap: ImageBitmap | null = null;
    let fallback = false;
    if (imageCapture && photoCapabilities.value) {
      try {
        const caps = photoCapabilities.value;
        const blob = await imageCapture.takePhoto({ imageWidth: profile.capture?.width ?? caps.imageWidth.max, imageHeight: profile.capture?.height ?? caps.imageHeight.max });
        bitmap = await createImageBitmap(blob);
        if (Math.abs(bitmap.width / bitmap.height - width / height) > 0.000001) throw new Error('Still aspect ratio differs from preview.');
        source = bitmap;
        width = bitmap.width;
        height = bitmap.height;
      } catch { bitmap?.close(); bitmap = null; fallback = true; }
    }
    try {
      if (token !== generation || video.readyState < 2) throw new Error('The camera changed during capture. Please try again.');
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;
      const ctx = canvas.getContext('2d');
      if (!ctx) throw new Error('Image capture is unavailable.');
      ctx.drawImage(source, 0, 0, width, height);
      return { data: canvas.toDataURL('image/jpeg', 1), fallback };
    } finally { bitmap?.close(); }
  }
  return { stream, devices, actual, capabilities, photoCapabilities, error, notice, busy, refreshDevices, start, stop, capture };
}
