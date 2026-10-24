import { Window } from 'happy-dom';
import assert from 'node:assert/strict';

const browser = new Window({ url: 'http://localhost/' });
for (const name of ['window', 'document', 'navigator', 'HTMLElement', 'Element', 'Node', 'Document', 'SVGElement', 'Event', 'KeyboardEvent', 'MouseEvent']) {
  Object.defineProperty(globalThis, name, { configurable: true, value: (browser as any)[name] });
}
const { createApp, h, nextTick } = await import('vue');
const { commands } = await import('../../src/bindings.ts');
const { dataTransferActive, registerTransferSave } = await import('../../src/lib/dataTransfer.ts');
const { default: DataTransferModal } = await import('../../src/components/DataTransferModal.vue');
const ok = (data: any) => ({ status: 'ok' as const, data });
const fail = (error: string) => ({ status: 'error' as const, error });
const counts = { clients: 2, sessions: 3, photos: 4 };
const calls: string[] = [];
let current = counts;
let exportResult: any = '/backups/clients.zip';
let validationResult: any = counts;
let exportError = '';
let closeCount = 0;
Object.assign(commands, {
  beginBackup: async () => { calls.push('begin'); return ok(current); },
  endBackup: async () => { calls.push('end'); return ok(null); },
  exportBackup: async () => { calls.push('export'); return exportError ? fail(exportError) : ok(exportResult); },
  validateBackup: async () => { calls.push('validate'); return ok(validationResult); },
  importBackup: async () => { calls.push('import'); return ok(null); },
});
const settle = async () => { await nextTick(); await new Promise(r => setTimeout(r, 0)); await nextTick(); };
const mount = async () => {
  const host = document.createElement('div');
  document.body.append(host);
  const app = createApp({ render: () => h(DataTransferModal, { onClose: () => { closeCount++; } }) });
  app.mount(host);
  await settle();
  return { host, unmount: () => { app.unmount(); host.remove(); } };
};
const button = (host: HTMLElement, text: string) => [...host.querySelectorAll('button')].find(b => b.textContent?.trim() === text)!;
const click = async (host: HTMLElement, text: string) => {
  const target = button(host, text);
  assert(target, `Missing button: ${text}`);
  assert(!target.disabled, `Disabled button: ${text}`);
  target.click();
  await settle();
};

// The backend lease is acquired only after the active editor finishes saving.
let finishSave!: () => void;
const unregister = registerTransferSave(async () => {
  calls.push('save');
  assert.equal(dataTransferActive.value, true);
  await new Promise<void>(resolve => { finishSave = resolve; });
});
let view = await mount();
assert.deepEqual(calls, ['save']);
assert.equal(button(view.host, 'Close').disabled, true);
finishSave();
await settle();
assert.deepEqual(calls, ['save', 'begin']);
unregister();
assert.equal(dataTransferActive.value, true);

// Validation cancellation leaves the ordinary export/import screen available.
validationResult = null;
await click(view.host, 'Import from ZIP…');
assert(button(view.host, 'Export all data…'));
validationResult = counts;
await click(view.host, 'Import from ZIP…');
assert.match(view.host.textContent!, /overwrite all current client data/);
assert(button(view.host, 'Export current data first…'));
assert(!button(view.host, 'Import and restart'));

// Neither cancelling nor failing the requested export proceeds to replacement.
exportResult = null;
await click(view.host, 'Export current data first…');
assert(!button(view.host, 'Import and restart'));
assert(!calls.includes('import'));
exportError = 'Disk full';
await click(view.host, 'Export current data first…');
assert.match(view.host.querySelector('[role="alert"]')!.textContent!, /Disk full/);
assert(!button(view.host, 'Import and restart'));
exportError = '';
exportResult = '/backups/clients.zip';
await click(view.host, 'Export current data first…');
assert.match(view.host.textContent!, /Backup saved to/);
assert(button(view.host, 'Import and restart'));
assert(!calls.includes('import'));
await click(view.host, 'Import and restart');
assert.equal(calls.at(-1), 'import');
await click(view.host, 'Cancel');
assert.equal(calls.at(-1), 'end');
assert.equal(dataTransferActive.value, false);
assert.equal(closeCount, 1);
view.unmount();

// Skipping export requires the same final explicit destructive confirmation.
view = await mount();
const before = calls.filter(c => c === 'export').length;
await click(view.host, 'Import from ZIP…');
await click(view.host, 'Replace without exporting');
assert.match(view.host.textContent!, /All current client data and photos will be replaced/);
assert.equal(calls.filter(c => c === 'export').length, before);
await click(view.host, 'Cancel');
view.unmount();

// An empty installation goes straight to review, with no misleading overwrite prompt.
current = { clients: 0, sessions: 0, photos: 0 };
view = await mount();
await click(view.host, 'Import from ZIP…');
assert(button(view.host, 'Import and restart'));
assert(!button(view.host, 'Replace without exporting'));
await click(view.host, 'Cancel');
view.unmount();

// Save failures stop before the backend is locked; closing permits editing again.
const rejectSave = registerTransferSave(async () => { throw new Error('Session save failed'); });
const begins = calls.filter(c => c === 'begin').length;
view = await mount();
assert.match(view.host.querySelector('[role="alert"]')!.textContent!, /Session save failed/);
assert.equal(calls.filter(c => c === 'begin').length, begins);
await click(view.host, 'Close');
assert.equal(dataTransferActive.value, false);
rejectSave();
view.unmount();
await browser.happyDOM.abort();
console.log('Import/export save barrier, validation, cancellation, export failure, overwrite, and restart checks passed.');
