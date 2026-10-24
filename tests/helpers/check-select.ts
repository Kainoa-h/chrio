// Run in a separate process so camera tests' browser mocks cannot affect Vue DOM setup.
import { Window } from 'happy-dom';
import assert from 'node:assert/strict';

const browser = new Window({ url: 'http://localhost/' });
for (const name of ['window', 'document', 'navigator', 'HTMLElement', 'Element', 'Node', 'Document', 'DocumentFragment', 'SVGElement', 'Event', 'KeyboardEvent', 'PointerEvent', 'MouseEvent', 'CustomEvent', 'MutationObserver', 'ResizeObserver', 'HTMLInputElement', 'HTMLSelectElement', 'HTMLOptionElement']) {
  Object.defineProperty(globalThis, name, { configurable: true, value: (browser as any)[name] });
}
globalThis.getComputedStyle = browser.getComputedStyle.bind(browser) as typeof getComputedStyle;
globalThis.requestAnimationFrame = browser.requestAnimationFrame.bind(browser);
globalThis.cancelAnimationFrame = browser.cancelAnimationFrame.bind(browser);
const { createApp, h, ref, nextTick } = await import('vue');
const { default: AppSelect } = await import('../../src/components/ui/select/AppSelect.vue');
const value = ref<string | number | null>(null);
const disabled = ref(false);
const options = ref([
  { value: null, label: 'All years' },
  { value: 2025, label: '2025' },
  { value: '', label: 'Auto' },
  { value: 'camera-a', label: 'Camera A' },
  { value: 'disabled', label: 'Unavailable', disabled: true },
]);
const host = document.createElement('div');
document.body.append(host);
const app = createApp({ render: () => h(AppSelect, {
  modelValue: value.value, options: options.value, disabled: disabled.value,
  'aria-label': 'Test select',
  'onUpdate:modelValue': (v: string | number | null) => { value.value = v; },
}) });
app.mount(host);
const settle = async () => { await nextTick(); await new Promise(r => setTimeout(r, 50)); await nextTick(); };
const trigger = () => host.querySelector<HTMLButtonElement>('[role="combobox"]')!;
const key = async (element: Element, value: string) => {
  element.dispatchEvent(new KeyboardEvent('keydown', { key: value, bubbles: true, cancelable: true }));
  await settle();
};
const open = async () => { trigger().focus(); await key(trigger(), 'ArrowDown'); };
const pick = async (label: string) => {
  const item = [...document.querySelectorAll<HTMLElement>('[role="option"]')].find(item => item.textContent?.trim() === label)!;
  assert(item, `Missing option ${label}`);
  item.focus();
  await key(item, 'Enter');
};
await settle();
assert.match(trigger().textContent!, /All years/);
assert.equal(trigger().getAttribute('aria-label'), 'Test select');
await open();
assert(document.querySelector('[role="listbox"]'), 'Menu opens');
assert.equal(document.querySelector('[role="option"][data-disabled]')?.textContent?.trim(), 'Unavailable');
await pick('2025');
assert.equal(value.value, 2025, 'Numeric option retains its type');
assert.match(trigger().textContent!, /2025/);
await open();
await pick('Auto');
assert.equal(value.value, '', 'Empty-string option remains selectable');
await open();
await pick('All years');
assert.equal(value.value, null, 'All option resets the filter to null');
await open();
const focused = document.activeElement!;
await key(focused, 'c');
assert.match(document.activeElement?.textContent ?? '', /Camera A/, 'Typeahead finds the camera');
await key(document.activeElement!, 'Escape');
assert.equal(document.querySelector('[role="listbox"]'), null, 'Escape closes the menu');
assert.equal(document.activeElement, trigger(), 'Focus returns to trigger');
disabled.value = true;
await settle();
assert.equal(trigger().disabled, true);
disabled.value = false;
options.value = [{ value: 'camera-a', label: 'Renamed camera' }, { value: null, label: 'All dates' }];
await settle();
assert.match(trigger().textContent!, /All dates/, 'Option updates preserve selection by original value');
app.unmount();
await browser.happyDOM.abort();
console.log('Select keyboard, typed values, disabled state, dynamic options, and focus checks passed.');
