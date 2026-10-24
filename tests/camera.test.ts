import { beforeEach, describe, expect, test } from 'bun:test';
import { useCamera } from '../src/composables/useCamera';
import { defaultProfile } from '../src/lib/cameraSettings';

function fakeStream(id = 'a', width = 1920, height = 1080) {
  let stopped = false;
  const applied: MediaTrackConstraints[] = [];
  const track = {
    stop: () => { stopped = true; },
    getSettings: () => ({ deviceId: id, width, height }), getCapabilities: () => ({}), addEventListener: () => {},
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
    expect(camera.photoCapabilities.value).toBeNull();
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

describe('capture', () => {
  function environment({ blankStill = false, blankVideo = false, failStill = false, width = 640, height = 480 } = {}) {
    const source = fakeStream('a', width, height);
    acquire = async () => source.stream;
    let settings: PhotoSettings | undefined;
    let drawingNative = 0;
    const video = {
      readyState: 2, blank: blankVideo,
      get videoWidth() { return source.track.getSettings().width; },
      get videoHeight() { return source.track.getSettings().height; },
      requestVideoFrameCallback(callback: () => void) { queueMicrotask(callback); return 1; },
      cancelVideoFrameCallback() {}, removeEventListener() {},
    } as unknown as HTMLVideoElement;
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { createElement: () => {
      const canvas = { width: 0, height: 0, blank: false,
        getContext: () => ({
          drawImage: (image: { blank?: boolean; native?: boolean }) => { canvas.blank = !!image.blank; if (image.native) drawingNative++; },
          getImageData: () => ({ data: new Uint8ClampedArray(canvas.blank ? [0, 1, 2, 255] : [100, 150, 200, 255]) }),
        }),
        toDataURL: () => canvas.blank ? 'data:image/jpeg;base64,black' : `data:image/jpeg;base64,frame-${canvas.width}x${canvas.height}`,
      };
      return canvas;
    } } });
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { ImageCapture: true } });
    Object.defineProperty(globalThis, 'ImageCapture', { configurable: true, value: class {
      async getPhotoCapabilities() { return { imageWidth: { min: 640, max: 1920 }, imageHeight: { min: 480, max: 1080 } }; }
      async takePhoto(value: PhotoSettings) { settings = value; if (failStill) throw new Error('Failed'); return new Blob(['native JPEG pixels'], { type: 'image/jpeg' }); }
    } });
    Object.defineProperty(globalThis, 'FileReader', { configurable: true, value: class {
      result = 'data:image/jpeg;base64,nativePhoto';
      onload: (() => void) | null = null;
      readAsDataURL() { this.onload?.(); }
    } });
    Object.defineProperty(globalThis, 'Image', { configurable: true, value: class {
      native = true;
      blank = blankStill;
      naturalWidth = 1920;
      naturalHeight = 1080;
      src = '';
      async decode() {}
    } });
    return { source, video, settings: () => settings, drawingNative: () => drawingNative };
  }
  test('uses visible streaming pixels when separate still capture is unsupported', async () => {
    const env = environment({ width: 1920, height: 1080 });
    Object.defineProperty(globalThis, 'window', { configurable: true, value: {} });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(env.video, defaultProfile())).toMatchObject({ data: 'data:image/jpeg;base64,frame-1920x1080', fallback: false });
    camera.stop();
  });
  test('preserves a nonblank native JPEG and maps its crop across differing aspect ratios', async () => {
    const env = environment();
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const profile = { ...defaultProfile(), capture: { width: 1920, height: 1080 }, crop: { x: 0.25, y: 0.25, width: 0.25 } };
    const result = await camera.capture(env.video, profile);
    expect(result).toMatchObject({ data: 'data:image/jpeg;base64,nativePhoto', fallback: false, width: 1920, height: 1080, crop: { x: 0.3125, y: 0.25, width: 0.1875 } });
    expect(env.settings()).toEqual({ imageWidth: 1920, imageHeight: 1080 });
    expect(env.drawingNative()).toBe(1); // Validate pixels, but retain the original JPEG.
    expect(env.source.applied.length).toBe(0);
    camera.stop();
  });
  test('rejects a decodable black JPEG and recovers full resolution from fresh video', async () => {
    const env = environment({ blankStill: true });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    const result = await camera.capture(env.video, { ...defaultProfile(), capture: { width: 1920, height: 1080 } });
    expect(result).toMatchObject({ data: 'data:image/jpeg;base64,frame-1920x1080', fallback: false, width: 1920, height: 1080 });
    expect(env.source.applied).toEqual([
      { width: { ideal: 1920 }, height: { ideal: 1080 } },
      { width: { ideal: 640 }, height: { ideal: 480 } },
    ]);
    expect(env.video.videoWidth).toBe(640);
    expect(env.video.videoHeight).toBe(480);
    camera.stop();
  });
  test('also recovers when takePhoto rejects', async () => {
    const env = environment({ failStill: true });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(env.video, defaultProfile())).toMatchObject({ data: 'data:image/jpeg;base64,frame-1920x1080', fallback: false });
    camera.stop();
  });
  test('refuses to save an image when still and video captures are both blank', async () => {
    const env = environment({ blankStill: true, blankVideo: true });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    await expect(camera.capture(env.video, defaultProfile())).rejects.toThrow('blank frames');
    expect(env.video.videoWidth).toBe(640);
    camera.stop();
  });
  test('keeps the visible pre-capture frame if resolution switching fails', async () => {
    const env = environment({ blankStill: true });
    env.source.track.applyConstraints = async () => { throw new Error('Unsupported resolution'); };
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(env.video, defaultProfile())).toMatchObject({ data: 'data:image/jpeg;base64,frame-640x480', fallback: true, width: 640, height: 480 });
    camera.stop();
  });
});
