import { expect, test } from 'bun:test';
import { isBlankFrame } from '../src/lib/cameraPixels';

test('detects black JPEG noise, empty frames, and transparent frames', () => {
  expect(isBlankFrame([])).toBe(true);
  expect(isBlankFrame([0, 0, 0, 255, 1, 2, 3, 255])).toBe(true);
  expect(isBlankFrame([255, 255, 255, 0])).toBe(true);
});

test('accepts visible pixels, including dim frames and saturated colors', () => {
  expect(isBlankFrame([9, 9, 9, 255])).toBe(false);
  expect(isBlankFrame([0, 0, 0, 255, 0, 0, 180, 255])).toBe(false);
});
