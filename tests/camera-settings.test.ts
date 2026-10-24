import { describe, expect, test } from 'bun:test';
import { setActivePinia, createPinia } from 'pinia';
import { clampCrop, frameHorizontalDelta, fitCaptureResolution, defaultProfile, emptyPreferences, parsePreferences, settingsKey } from '../src/lib/cameraSettings';
import { useCameraSettings } from '../src/stores/camera';

describe('camera framing', () => {
  test('maps screen movement and visible resize-corner direction to original coordinates', () => {
    const original = { x: 0.2, y: 0.1, width: 0.25 };
    for (const mirrored of [false, true]) {
      const dx = frameHorizontalDelta(0.05, mirrored);
      const moved = clampCrop({ ...original, x: original.x + dx }, 4 / 3);
      const screenX = (c: typeof original) => mirrored ? 1 - c.x - c.width : c.x;
      expect(screenX(moved) - screenX(original)).toBeCloseTo(0.05);
      const resized = clampCrop({ ...original, width: original.width + dx }, 4 / 3);
      const cornerX = (c: typeof original) => mirrored ? 1 - c.x - c.width : c.x + c.width;
      expect(cornerX(resized) - cornerX(original)).toBeCloseTo(0.05);
    }
    expect(original).toEqual({ x: 0.2, y: 0.1, width: 0.25 });
  });
  test('centers the largest 2:3 frame in landscape and portrait sources', () => {
    for (const aspect of [16 / 9, 4 / 3, 2 / 3, 9 / 16]) {
      const c = clampCrop(null, aspect);
      expect(c.width).toBeLessThanOrEqual(1);
      expect(c.width * 1.5 * aspect).toBeLessThanOrEqual(1);
      expect(c.x).toBeCloseTo((1 - c.width) / 2);
      expect(c.y).toBeCloseTo((1 - c.width * 1.5 * aspect) / 2);
    }
  });
  test('clamps framing after a resolution or aspect ratio change', () => {
    const saved = { x: 0.8, y: 0.9, width: 0.7 };
    for (const aspect of [16 / 9, 4 / 3, 9 / 16]) {
      const c = clampCrop(saved, aspect);
      expect(c.x).toBeGreaterThanOrEqual(0);
      expect(c.y).toBeGreaterThanOrEqual(0);
      expect(c.x + c.width).toBeLessThanOrEqual(1);
      expect(c.y + c.width * 1.5 * aspect).toBeLessThanOrEqual(1);
    }
    expect(saved).toEqual({ x: 0.8, y: 0.9, width: 0.7 });
  });
  test('fits capture dimensions to the preview aspect without exceeding the request', () => {
    expect(fitCaptureResolution({ width: 1920, height: 1080 }, 4 / 3)).toEqual({ width: 1440, height: 1080 });
    expect(fitCaptureResolution({ width: 1920, height: 1920 }, 16 / 9)).toEqual({ width: 1920, height: 1080 });
    expect(fitCaptureResolution({ width: 640, height: 480 }, 16 / 9)).toEqual({ width: 640, height: 360 });
    expect(fitCaptureResolution({ width: 1920, height: 1080 }, 9 / 16)).toEqual({ width: 607, height: 1080 });
  });
  test('keeps a usable minimum width', () => {
    expect(clampCrop({ x: -1, y: -1, width: -1 }, 16 / 9)).toEqual({ x: 0, y: 0, width: 0.05 });
  });
});

describe('camera preferences', () => {
  test('defaults mirroring off without discarding legacy camera profiles', () => {
    const legacy = { version: 1, deviceId: 'a', profiles: { a: defaultProfile() } };
    for (const mirrorPreviews of [undefined, null, 'true', 1, false]) {
      expect(parsePreferences(JSON.stringify({ ...legacy, mirrorPreviews }))).toEqual({ ...legacy, mirrorPreviews: false });
    }
    expect(parsePreferences(JSON.stringify({ ...legacy, mirrorPreviews: true }))).toEqual({ ...legacy, mirrorPreviews: true });
  });
  test('uses defaults for absent, corrupt, or unsupported data', () => {
    for (const raw of [null, '{', 'null', '{}', '{"version":2}', '{"version":1,"deviceId":"","profiles":[]}']) {
      expect(parsePreferences(raw)).toEqual(emptyPreferences());
    }
  });
  test('round-trips independent device profiles and rejects corrupt profiles', () => {
    const p = { ...emptyPreferences(), deviceId: 'a', profiles: { a: defaultProfile(), b: { ...defaultProfile(), stream: null, crop: { x: 0.2, y: 0.1, width: 0.3 } } } };
    const loaded = parsePreferences(JSON.stringify(p));
    expect(loaded).toEqual(p);
    const invalid = parsePreferences(JSON.stringify({ ...p, profiles: { ...p.profiles, bad: { stream: { width: -1, height: 1 }, capture: null, crop: null }, brokenCrop: { ...defaultProfile(), crop: { x: 0, y: 0, width: 0 } } } }));
    expect(Object.keys(invalid.profiles)).toEqual(['a', 'b']);
  });
  test('persists saved preferences without sharing the editable draft', () => {
    const memory = new Map<string, string>();
    Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: { getItem: (key: string) => memory.get(key) ?? null, setItem: (key: string, value: string) => memory.set(key, value) } });
    setActivePinia(createPinia());
    const store = useCameraSettings();
    const draft = { ...emptyPreferences(), deviceId: 'camera-a', mirrorPreviews: true, profiles: { 'camera-a': defaultProfile() } };
    store.save(draft);
    draft.deviceId = 'unsaved';
    draft.mirrorPreviews = false;
    expect(store.preferences.mirrorPreviews).toBe(true);
    expect(store.preferences.deviceId).toBe('camera-a');
    expect(parsePreferences(memory.get(settingsKey)!)).toEqual(store.preferences);
    setActivePinia(createPinia());
    expect(useCameraSettings().preferences.deviceId).toBe('camera-a');
    expect(useCameraSettings().preferences.mirrorPreviews).toBe(true);
  });
});
