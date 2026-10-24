export type Crop = { x: number; y: number; width: number };
export type Resolution = { width: number; height: number };
export type CameraProfile = { stream: Resolution | null; capture: Resolution | null; crop: Crop | null };
export type CameraPreferences = { version: 1; deviceId: string; profiles: Record<string, CameraProfile> };
export const settingsKey = 'chrio.camera.v1';
export const resolutions: Resolution[] = [
  { width: 640, height: 480 }, { width: 1280, height: 720 },
  { width: 1920, height: 1080 }, { width: 2560, height: 1440 }, { width: 3840, height: 2160 },
];
export const defaultProfile = (): CameraProfile => ({ stream: { width: 1920, height: 1080 }, capture: null, crop: null });
export const emptyPreferences = (): CameraPreferences => ({ version: 1, deviceId: '', profiles: {} });
// Camera modes with different aspect ratios share a centered field of view.
// Map the preview's crop through that common field without stretching it.
export function mapCaptureCrop(crop: Crop, previewAspect: number, captureAspect: number): Crop {
  const source = clampCrop(crop, previewAspect);
  const width = Math.min(1, previewAspect / captureAspect);
  const height = Math.min(1, captureAspect / previewAspect);
  return { x: (1 - width) / 2 + source.x * width, y: (1 - height) / 2 + source.y * height, width: source.width * width };
}
export function clampCrop(crop: Crop | null, aspect: number): Crop {
  const maxWidth = Math.min(1, 1 / (1.5 * aspect));
  if (!crop) return { width: maxWidth, x: (1 - maxWidth) / 2, y: (1 - maxWidth * 1.5 * aspect) / 2 };
  const width = Math.max(Math.min(0.05, maxWidth), Math.min(maxWidth, crop.width));
  return { width, x: Math.max(0, Math.min(1 - width, crop.x)), y: Math.max(0, Math.min(1 - width * 1.5 * aspect, crop.y)) };
}
function validResolution(value: unknown): value is Resolution | null {
  if (value === null) return true;
  const r = value as Resolution | undefined;
  return !!r && Number.isInteger(r.width) && Number.isInteger(r.height) && r.width > 0 && r.height > 0 && r.width <= 65536 && r.height <= 65536;
}
export function parsePreferences(raw: string | null): CameraPreferences {
  try {
    const p = JSON.parse(raw || 'null');
    if (p?.version !== 1 || typeof p.deviceId !== 'string' || !p.profiles || typeof p.profiles !== 'object' || Array.isArray(p.profiles)) return emptyPreferences();
    const profiles: Record<string, CameraProfile> = Object.create(null);
    for (const [id, value] of Object.entries(p.profiles)) {
      const v = value as CameraProfile;
      if (!v || !validResolution(v.stream) || !validResolution(v.capture)) continue;
      const c = v.crop;
      if (c !== null && (!c || ![c.x, c.y, c.width].every(Number.isFinite) || c.x < 0 || c.y < 0 || c.width <= 0 || c.x > 1 || c.y > 1 || c.width > 1)) continue;
      profiles[id] = { stream: v.stream, capture: v.capture, crop: c };
    }
    return { version: 1, deviceId: p.deviceId, profiles };
  } catch { return emptyPreferences(); }
}
