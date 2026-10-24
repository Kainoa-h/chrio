import { ref, shallowRef } from 'vue';
import { hasCameraPixels, waitForCameraFrame } from '@/lib/cameraPixels';
import { clampCrop, mapCaptureCrop, type CameraProfile } from '@/lib/cameraSettings';

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
  async function capture(video: HTMLVideoElement, profile: CameraProfile) {
    if (!stream.value || video.readyState < 2 || !video.videoWidth || !video.videoHeight) throw new Error('Wait for the camera preview to be ready.');
    const token = generation;
    const width = video.videoWidth;
    const height = video.videoHeight;
    const crop = clampCrop(profile.crop, width / height);
    // Freeze the fallback before takePhoto: still capture can interrupt or mute
    // the live track, making a video draw performed afterwards black.
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Image capture is unavailable.');
    ctx.drawImage(video, 0, 0, width, height);
    const fallbackData = canvas.toDataURL('image/jpeg', 1);
    const fallbackHasPixels = hasCameraPixels(canvas);
    const track = stream.value.getVideoTracks()[0];
    const caps = photoCapabilities.value;
    const target = profile.capture ?? (caps ? { width: caps.imageWidth.max, height: caps.imageHeight.max } : { width, height });
    const ensureCurrent = () => {
      if (token !== generation) throw new Error('The camera changed during capture. Please try again.');
    };
    if (imageCapture && photoCapabilities.value) {
      try {
        const caps = photoCapabilities.value;
        const blob = await imageCapture.takePhoto({ imageWidth: profile.capture?.width ?? caps.imageWidth.max, imageHeight: profile.capture?.height ?? caps.imageHeight.max });
        ensureCurrent();
        if (!blob.size) throw new Error('The camera returned an empty photo.');
        const data = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(reader.result as string);
          reader.onerror = () => reject(new Error('Could not read the captured photo.'));
          reader.readAsDataURL(blob);
        });
        const image = new Image();
        image.src = data;
        await image.decode();
        ensureCurrent();
        if (!image.naturalWidth || !image.naturalHeight) throw new Error('The camera returned an invalid photo.');
        if (!hasCameraPixels(image)) throw new Error('The camera returned a blank still photo.');
        // Preserve a native JPEG directly. Avoid routing still photos through
        // ImageBitmap/canvas, which can lose the camera's decoded pixels.
        let photoData = data;
        if (blob.type !== 'image/jpeg') {
          canvas.width = image.naturalWidth;
          canvas.height = image.naturalHeight;
          ctx.drawImage(image, 0, 0);
          photoData = canvas.toDataURL('image/jpeg', 1);
        }
        return { data: photoData, fallback: false, crop: mapCaptureCrop(crop, width / height, image.naturalWidth / image.naturalHeight), width: image.naturalWidth, height: image.naturalHeight };
      } catch {
        ensureCurrent();
        // Some cameras advertise still capture but return a valid, black JPEG.
        // Recover using fresh video frames, requesting the capture resolution.
      }
    } else if (!profile.capture && fallbackHasPixels) {
      ensureCurrent();
      return { data: fallbackData, fallback: false, crop, width, height };
    }
    const originalConstraints = track.getConstraints();
    let changed = false;
    try {
      if (target.width !== width || target.height !== height) {
        changed = true;
        await track.applyConstraints({ ...originalConstraints, width: { ideal: target.width }, height: { ideal: target.height } });
        ensureCurrent();
      }
      // Skip the first frame after reconfiguration/still capture, which can be blank.
      await waitForCameraFrame(video, track);
      for (let attempt = 0; attempt < 3; attempt++) {
        await waitForCameraFrame(video, track);
        ensureCurrent();
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
        if (!hasCameraPixels(canvas)) continue;
        return {
          data: canvas.toDataURL('image/jpeg', 1),
          fallback: canvas.width < target.width || canvas.height < target.height,
          crop: mapCaptureCrop(crop, width / height, canvas.width / canvas.height),
          width: canvas.width, height: canvas.height,
        };
      }
      throw new Error('The camera returned blank frames. Wait for the preview to recover and try again.');
    } catch (e) {
      ensureCurrent();
      if (!fallbackHasPixels) throw e;
      return { data: fallbackData, fallback: true, crop, width, height };
    } finally {
      if (changed && token === generation && stream.value) {
        try {
          await track.applyConstraints({ ...originalConstraints, width: originalConstraints.width ?? { ideal: width }, height: originalConstraints.height ?? { ideal: height } });
        } catch {
          if (token === generation) notice.value = 'Photo captured, but the preview resolution could not be restored. Reopen the camera to restore it.';
        }
      }
    }
  }
  return { stream, devices, actual, capabilities, photoCapabilities, error, notice, busy, refreshDevices, start, stop, capture };
}
