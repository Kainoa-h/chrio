// JPEG compression can turn an empty black frame into values slightly above 0.
// Ignore alpha; do not confuse a transparent frame with visible image data.
export function isBlankFrame(pixels: ArrayLike<number>): boolean {
  for (let i = 0; i + 3 < pixels.length; i += 4) {
    if (pixels[i + 3] > 0 && Math.max(pixels[i], pixels[i + 1], pixels[i + 2]) > 8) return false;
  }
  return true;
}

export function hasCameraPixels(source: CanvasImageSource): boolean {
  const sample = document.createElement('canvas');
  sample.width = sample.height = 32;
  const ctx = sample.getContext('2d', { willReadFrequently: true });
  if (!ctx) throw new Error('Image capture is unavailable.');
  ctx.drawImage(source, 0, 0, 32, 32);
  return !isBlankFrame(ctx.getImageData(0, 0, 32, 32).data);
}

export function waitForCameraFrame(video: HTMLVideoElement, track: MediaStreamTrack): Promise<void> {
  return new Promise((resolve, reject) => {
    let callbackId: number | undefined;
    const cleanup = () => {
      clearTimeout(timeout);
      if (callbackId !== undefined) video.cancelVideoFrameCallback(callbackId);
      video.removeEventListener('timeupdate', check);
    };
    const check = () => {
      callbackId = undefined;
      const settings = track.getSettings();
      if (track.readyState === 'ended') { cleanup(); reject(new Error('The camera disconnected during capture.')); return; }
      if (!track.muted && video.readyState >= 2 && video.videoWidth && video.videoHeight &&
          (!settings.width || settings.width === video.videoWidth) && (!settings.height || settings.height === video.videoHeight)) {
        cleanup(); resolve();
      } else if (typeof video.requestVideoFrameCallback === 'function') callbackId = video.requestVideoFrameCallback(check);
    };
    const timeout = setTimeout(() => { cleanup(); reject(new Error('No fresh camera frame arrived. Please try again.')); }, 3000);
    if (typeof video.requestVideoFrameCallback === 'function') callbackId = video.requestVideoFrameCallback(check);
    else video.addEventListener('timeupdate', check);
  });
}
