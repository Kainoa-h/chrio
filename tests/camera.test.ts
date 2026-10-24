import { beforeEach, describe, expect, test } from 'bun:test';
import { useCamera } from '../src/composables/useCamera';
import { defaultProfile } from '../src/lib/cameraSettings';

function fakeStream(id = 'a', width = 1920, height = 1080) {
  let stopped = false;
  const track = { stop: () => { stopped = true; }, getSettings: () => ({ deviceId: id, width, height }), getCapabilities: () => ({}), addEventListener: () => {} };
  return { stream: { getTracks: () => [track], getVideoTracks: () => [track] } as unknown as MediaStream, stopped: () => stopped };
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
  const video = { readyState: 2, videoWidth: 1920, videoHeight: 1080 } as HTMLVideoElement;
  function canvas() {
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { createElement: () => ({ width: 0, height: 0, getContext: () => ({ drawImage: () => {} }), toDataURL: () => 'data:image/jpeg;base64,photo' }) } });
  }
  test('uses the streaming frame when separate still capture is unsupported', async () => {
    canvas();
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect(await camera.capture(video, defaultProfile())).toEqual({ data: 'data:image/jpeg;base64,photo', fallback: false });
    camera.stop();
  });
  test('uses requested still dimensions and falls back on failure or aspect mismatch', async () => {
    canvas();
    let fail = false;
    let mismatch = false;
    let settings: PhotoSettings | undefined;
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { ImageCapture: true } });
    Object.defineProperty(globalThis, 'ImageCapture', { configurable: true, value: class {
      async getPhotoCapabilities() { return { imageWidth: { min: 640, max: 3840 }, imageHeight: { min: 480, max: 2160 } }; }
      async takePhoto(value: PhotoSettings) { settings = value; if (fail) throw new Error('Failed'); return new Blob(); }
    } });
    Object.defineProperty(globalThis, 'createImageBitmap', { configurable: true, value: async () => ({ width: 3840, height: mismatch ? 2880 : 2160, close: () => {} }) });
    const camera = useCamera();
    await camera.start('a', defaultProfile());
    expect((await camera.capture(video, defaultProfile())).fallback).toBe(false);
    expect(settings).toEqual({ imageWidth: 3840, imageHeight: 2160 });
    const profile = { ...defaultProfile(), capture: { width: 1920, height: 1080 } };
    await camera.capture(video, profile);
    expect(settings).toEqual({ imageWidth: 1920, imageHeight: 1080 });
    fail = true;
    expect((await camera.capture(video, profile)).fallback).toBe(true);
    fail = false;
    mismatch = true;
    expect((await camera.capture(video, profile)).fallback).toBe(true);
    camera.stop();
  });
});
