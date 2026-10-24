import { beforeEach, describe, expect, test } from 'bun:test';
import { useCamera } from '../src/composables/useCamera';
import { defaultProfile } from '../src/lib/cameraSettings';

function fakeStream(id = 'a', width = 1920, height = 1080) {
  let stopped = false;
  const applied: MediaTrackConstraints[] = [];
  const track = {
    stop: () => { stopped = true; },
    getSettings: () => ({ deviceId: id, width, height }), getCapabilities: () => ({ width: { min: 320, max: 1920 }, height: { min: 240, max: 1080 } }), addEventListener: () => {},
    getConstraints: () => ({ width: { ideal: width }, height: { ideal: height } }),
    applyConstraints: async (constraints: MediaTrackConstraints) => {
      applied.push(constraints);
      width = (constraints.width as ConstrainULongRange)?.ideal ?? width;
      height = (constraints.height as ConstrainULongRange)?.ideal ?? height;
    },
  };
  return { stream: { getTracks: () => [track], getVideoTracks: () => [track] } as unknown as MediaStream, stopped: () => stopped, track, applied };
}
let acquire: (constraints: MediaStreamConstraints) => Promise<MediaStream>;
let requests: MediaStreamConstraints[];
beforeEach(() => {
  requests = [];
  acquire = async () => fakeStream().stream;
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: { mediaDevices: {
    getUserMedia: async (constraints: MediaStreamConstraints) => { requests.push(constraints); return acquire(constraints); },
    enumerateDevices: async () => [{ kind: 'videoinput', deviceId: 'a', label: 'Camera A' }],
  } } });
  Object.defineProperty(globalThis, 'window', { configurable: true, value: {} });
});
const tick = () => new Promise(resolve => setTimeout(resolve, 0));

describe('camera lifecycle', () => {
  test('uses resolution requests, reports actual resolution, and stops tracks', async () => {
    const source = fakeStream('a', 1280, 720);
    acquire = async () => source.stream;
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect((requests[0].video as MediaTrackConstraints).width).toEqual({ ideal: 1920 });
    expect(camera.actual.value).toEqual({ width: 1280, height: 720 });
    expect(camera.devices.value.length).toBe(1);
    camera.stop();
    expect(source.stopped()).toBe(true);
    expect(camera.stream.value).toBeNull();
  });
  test('stops a camera that finishes acquiring after the dialog closes', async () => {
    const source = fakeStream();
    let resolve!: (stream: MediaStream) => void;
    acquire = () => new Promise(r => { resolve = r; });
    const camera = useCamera();
    const pending = camera.start('a', defaultProfile());
    await tick();
    camera.stop();
    resolve(source.stream);
    await pending;
    expect(source.stopped()).toBe(true);
    expect(camera.stream.value).toBeNull();
  });
  test('serializes switching and discards stale acquisition results', async () => {
    const first = fakeStream('a');
    const second = fakeStream('b');
    let resolve!: (stream: MediaStream) => void;
    acquire = () => requests.length === 1 ? new Promise(r => { resolve = r; }) : Promise.resolve(second.stream);
    const camera = useCamera();
    const one = camera.start('a', defaultProfile());
    await tick();
    const two = camera.start('b', defaultProfile());
    expect(requests.length).toBe(1);
    resolve(first.stream);
    await Promise.all([one, two]);
    expect(first.stopped()).toBe(true);
    expect(camera.stream.value).toBe(second.stream);
    camera.stop();
  });
  test('falls back for a missing preferred camera without retrying permission denial', async () => {
    acquire = async () => {
      if (requests.length === 1) throw new DOMException('Missing camera', 'NotFoundError');
      return fakeStream('fallback').stream;
    };
    const camera = useCamera();
    await camera.start('missing', defaultProfile());
    expect(requests.length).toBe(2);
    expect(camera.notice.value).toContain('preferred camera is unavailable');
    camera.stop();
    requests = [];
    acquire = async () => { throw new DOMException('Permission denied', 'NotAllowedError'); };
    await camera.start('a', defaultProfile());
    expect(requests.length).toBe(1);
    expect(camera.error.value).toContain('Permission denied');
    expect(camera.busy.value).toBe(false);
  });
});

describe('full-frame capture', () => {
  function environment({ blankVideo = false, width = 640, height = 480 } = {}) {
    const source = fakeStream('a', width, height);
    acquire = async () => source.stream;
    const fullDraws: unknown[][] = [];
    let frameCount = 0;
    const video = {
      readyState: 2, blank: blankVideo,
      get videoWidth() { return source.track.getSettings().width; },
      get videoHeight() { return source.track.getSettings().height; },
      requestVideoFrameCallback(callback: () => void) { frameCount++; queueMicrotask(callback); return frameCount; },
      cancelVideoFrameCallback() {}, removeEventListener() {},
    } as unknown as HTMLVideoElement;
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { createElement: () => {
      const canvas = { width: 0, height: 0, blank: false,
        getContext: () => ({
          drawImage: (image: { blank?: boolean }, ...coordinates: number[]) => {
            canvas.blank = !!image.blank;
            if (image === video) fullDraws.push(coordinates);
          },
          getImageData: () => ({ data: new Uint8ClampedArray(canvas.blank ? [0, 1, 2, 255] : [100, 150, 200, 255]) }),
        }),
        toDataURL: () => canvas.blank ? 'data:image/jpeg;base64,black' : `data:image/jpeg;base64,full-frame-${canvas.width}x${canvas.height}`,
      };
      return canvas;
    } } });
    // Native still capture must never run, even if the environment offers it.
    Object.defineProperty(globalThis, 'ImageCapture', { configurable: true, value: class {
      constructor() { throw new Error('Native still capture may change the field of view'); }
    } });
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { ImageCapture: globalThis.ImageCapture } });
    return { source, video, fullDraws, frames: () => frameCount };
  }
  test('saves the full frame and leaves the mask as independent metadata', async () => {
    const env = environment({ width: 1920, height: 1080 });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const crop = { x: 0.3, y: 0.2, width: 0.1 };
    const result = await camera.capture(env.video, { ...defaultProfile(), crop });
    expect(result).toMatchObject({ data: 'data:image/jpeg;base64,full-frame-1920x1080', crop, width: 1920, height: 1080, fallback: false });
    // Five-argument drawImage copies the entire source; no crop rectangle exists.
    expect(env.fullDraws.every(args => args.length === 4 && args[0] === 0 && args[1] === 0 && args[2] === 1920 && args[3] === 1080)).toBe(true);
    expect(env.frames()).toBeGreaterThanOrEqual(2);
    expect(env.source.applied.length).toBe(0);
    camera.stop();
  });
  test('changing the mask never changes the encoded original', async () => {
    const env = environment({ width: 1920, height: 1080 });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const first = await camera.capture(env.video, { ...defaultProfile(), crop: { x: 0.1, y: 0.1, width: 0.1 } });
    const second = await camera.capture(env.video, { ...defaultProfile(), crop: { x: 0.6, y: 0.2, width: 0.2 } });
    expect(first.data).toBe(second.data);
    expect(first.crop).not.toEqual(second.crop);
    camera.stop();
  });
  test('fits higher-resolution capture to a 4:3 preview and restores its constraints', async () => {
    const env = environment();
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const crop = { x: 0.25, y: 0.25, width: 0.25 };
    const result = await camera.capture(env.video, { ...defaultProfile(), capture: { width: 1920, height: 1080 }, crop });
    expect(result).toMatchObject({ data: 'data:image/jpeg;base64,full-frame-1440x1080', crop, fallback: false, width: 1440, height: 1080 });
    expect(env.source.applied).toEqual([
      { width: { ideal: 1440 }, height: { ideal: 1080 }, aspectRatio: { exact: 4 / 3 } },
      { width: { ideal: 640 }, height: { ideal: 480 } },
    ]);
    expect(env.video.videoWidth).toBe(640);
    expect(env.video.videoHeight).toBe(480);
    camera.stop();
  });
  test('Auto uses video capabilities while preserving the preview aspect', async () => {
    const env = environment();
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(env.video, defaultProfile())).toMatchObject({ data: 'data:image/jpeg;base64,full-frame-1440x1080', fallback: false });
    camera.stop();
  });
  test('never accepts a camera mode with a different aspect ratio', async () => {
    const env = environment();
    env.source.track.applyConstraints = async constraints => {
      env.source.applied.push(constraints);
      if (env.source.applied.length === 1) {
        env.source.track.getSettings = () => ({ deviceId: 'a', width: 1920, height: 1920 });
      }
    };
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(env.video, defaultProfile())).toMatchObject({ data: 'data:image/jpeg;base64,full-frame-640x480', width: 640, height: 480, fallback: true });
    camera.stop();
  });
  test('refuses to save blank frames and restores the preview', async () => {
    const env = environment({ blankVideo: true });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    await expect(camera.capture(env.video, defaultProfile())).rejects.toThrow('blank frames');
    expect(env.video.videoWidth).toBe(640);
    camera.stop();
  });
  test('keeps the full pre-capture frame and mask if resolution switching fails', async () => {
    const env = environment();
    env.source.track.applyConstraints = async () => { throw new Error('Unsupported resolution'); };
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const crop = { x: 0.25, y: 0.25, width: 0.25 };
    expect(await camera.capture(env.video, { ...defaultProfile(), crop })).toMatchObject({ data: 'data:image/jpeg;base64,full-frame-640x480', fallback: true, width: 640, height: 480, crop });
    camera.stop();
  });
  test('does not return a capture after the dialog closes during a resolution change', async () => {
    const env = environment();
    let resolve!: () => void;
    env.source.track.applyConstraints = () => new Promise<void>(r => { resolve = r; });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const pending = camera.capture(env.video, defaultProfile());
    camera.stop();
    resolve();
    await expect(pending).rejects.toThrow('camera changed');
  });
});
