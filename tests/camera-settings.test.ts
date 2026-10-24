import { describe, expect, test } from 'bun:test';
import { setActivePinia, createPinia } from 'pinia';
import { clampCrop, mapCaptureCrop, defaultProfile, emptyPreferences, parsePreferences, settingsKey } from '../src/lib/cameraSettings';
import { useCameraSettings } from '../src/stores/camera';

describe('camera framing', () => {
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
  test('maps preview framing to wider and taller stills without changing its 2:3 ratio', () => {
    const frame = { x: 0.25, y: 0.2, width: 0.25 };
    for (const [preview, capture] of [[4 / 3, 16 / 9], [16 / 9, 4 / 3], [16 / 9, 16 / 9]]) {
      const mapped = mapCaptureCrop(frame, preview, capture);
      const visibleWidth = Math.min(1, preview / capture);
      const visibleHeight = Math.min(1, capture / preview);
      expect((mapped.x - (1 - visibleWidth) / 2) / visibleWidth).toBeCloseTo(frame.x);
      expect((mapped.y - (1 - visibleHeight) / 2) / visibleHeight).toBeCloseTo(frame.y);
      expect(mapped.width / visibleWidth).toBeCloseTo(frame.width);
      expect(mapped.x + mapped.width).toBeLessThanOrEqual(1);
      expect(mapped.y + mapped.width * 1.5 * capture).toBeLessThanOrEqual(1);
    }
  });
  test('keeps a usable minimum width', () => {
    expect(clampCrop({ x: -1, y: -1, width: -1 }, 16 / 9)).toEqual({ x: 0, y: 0, width: 0.05 });
  });
});

describe('camera preferences', () => {
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
    const draft = { ...emptyPreferences(), deviceId: 'camera-a', profiles: { 'camera-a': defaultProfile() } };
    store.save(draft);
    draft.deviceId = 'unsaved';
    expect(store.preferences.deviceId).toBe('camera-a');
    expect(parsePreferences(memory.get(settingsKey)!)).toEqual(store.preferences);
    setActivePinia(createPinia());
    expect(useCameraSettings().preferences.deviceId).toBe('camera-a');
  });
});
